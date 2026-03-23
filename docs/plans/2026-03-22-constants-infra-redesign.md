# Constants Infrastructure Redesign — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace guessed-and-hashed constants with gate-constructed constants derived from `u` (inserted via public logup sum), eliminating the Blake hash commitment and reducing circuit overhead.

**Architecture:** During building, `context.constant(val)` still reserves a Var via `guess()`. In finalize, instead of adding trivial `x+0=x` gates and Blake-hashing all constants, we construct every constant from `u` using arithmetic gates: a `+1` chain for consecutive M31 integers, power-of-2 base decomposition for larger M31 values, and QM31 basis combination (`i`, `u`, `iu`) for extension field constants. The preprocessed trace commits to the circuit structure, implicitly verifying constants.

**Tech Stack:** Rust nightly, stwo circuit DSL (`circuits`, `circuit_common`, `circuit_air`, `stark_verifier` crates)

---

## Background

### Current Flow
1. `context.constant(val)` calls `guess()` — allocates a Var, stores value, adds idx to `guessed_vars`
2. `finalize_guessed_vars()` adds trivial `Add { in0: idx, in1: 0, out: idx }` for every guessed var (including constants)
3. `hash_constants()` Blake-hashes all constants and adds the hash as public outputs
4. `generate_column_indices()` builds 0..N column indices with inline `+1` Add gates

### New Flow
1. `context.constant(val)` still calls `guess()` — same API, reserves Var
2. `Context::default()` reserves addresses 0 (zero), 1 (one), 2 (u)
3. In finalize:
   - Zero: `Add { in0: 0, in1: 0, out: 0 }` (enforces `2x=x → x=0`)
   - `u` at address 2: inserted via public logup sum (no gate needed from finalize — it's yielded by external logup)
   - One at address 1: `Mul { in0: 2, in1: 1, out: 2 }` (enforces `u*x=u → x=1`, note: the Mul gate yields a fresh var that equals u, and we Eq it to address 2... actually see Task 3 for the exact gate construction)
   - Build `+1` chain from `one` up to `max_consecutive_constant`
   - Pick `base = largest power of 2 ≤ max_consecutive`
   - Decompose larger M31 constants: `K = (a*base + b)*base + c` where `a,b,c < base`
   - Build QM31 bases: `i = u*u - 2`, `iu = i*u`
   - Decompose QM31 constants into M31 limbs, combine: `a + b*i + c*u + d*iu`
   - Non-constant guessed vars: trivial `x+0=x` as before
4. `hash_constants()` removed entirely
5. `generate_column_indices()` replaced with `context.constant(i.into())`

### Key Invariant
Every Var must appear exactly once as a yield. For constants, the construction gate's output directly uses the reserved Var idx — satisfying the yield invariant without extra Eq gates.

---

## Task 1: Reserve `u` in `Context::default()` and add `finalize_constants` skeleton

**Files:**
- Modify: `crates/circuits/src/context.rs:148-165` (Context::default, add u reservation)
- Create: `crates/circuit_common/src/finalize_constants.rs` (new module, skeleton)
- Modify: `crates/circuit_common/src/lib.rs` (add module)
- Test: `crates/circuits/src/context_test.rs`

**Step 1: Update `Context::default()` to reserve zero (idx 0), one (idx 1), u (idx 2)**

Currently:
```rust
fn default() -> Self {
    // ...
    res.constant(QM31::zero());   // idx 0
    res.constant(QM31::one());    // idx 1
    res
}
```

Change to:
```rust
fn default() -> Self {
    // ...
    res.constant(QM31::zero());                     // idx 0
    res.constant(QM31::one());                      // idx 1
    res.constant(qm31_from_u32s(0, 0, 1, 0));      // idx 2 (u)
    res
}
```

Add a `u()` accessor:
```rust
pub fn u(&self) -> Var {
    Var { idx: 2 }
}
```

**Step 2: Write a test verifying the address layout**

```rust
#[test]
fn test_default_constants_layout() {
    let context = TraceContext::default();
    assert_eq!(context.zero().idx, 0);
    assert_eq!(context.one().idx, 1);
    assert_eq!(context.u().idx, 2);
    assert_eq!(context.constants().len(), 3);
}
```

**Step 3: Run tests**

Run: `cargo test -p circuits --release`
Expected: PASS

**Step 4: Create `finalize_constants.rs` skeleton**

Create `crates/circuit_common/src/finalize_constants.rs` with:
```rust
use circuits::context::Context;
use circuits::ivalue::IValue;

/// Constructs all constants via arithmetic gates, replacing the old guess+hash approach.
/// Called during finalize_context, before padding.
pub fn finalize_constants(context: &mut Context<impl IValue>) {
    todo!("implement constant construction")
}
```

Register in `crates/circuit_common/src/lib.rs`.

**Step 5: Commit**

```
feat: reserve u at address 2 in Context::default, add finalize_constants skeleton
```

---

## Task 2: Implement the `+1` chain builder

**Files:**
- Modify: `crates/circuit_common/src/finalize_constants.rs`
- Test: `crates/circuit_common/src/finalize_constants.rs` (inline test module)

**Step 1: Write a failing test**

```rust
#[cfg(test)]
mod test {
    use super::*;
    use circuits::context::TraceContext;
    use circuits::ivalue::qm31_from_u32s;

    #[test]
    fn test_chain_builds_consecutive_m31_constants() {
        let mut context = TraceContext::default();
        // Request constants 0..5
        for i in 0u32..5 {
            context.constant(i.into());
        }
        // Build chain. After this, the chain vars should map values 1..5.
        let chain = build_plus_one_chain(&mut context);
        // Chain should have entries for 1..=4 (0 is zero, handled separately)
        assert_eq!(chain.len(), 5); // indices for values 0,1,2,3,4
        context.validate_circuit();
    }
}
```

**Step 2: Implement `build_plus_one_chain`**

This function:
1. Scans all constants to find the maximum consecutive M31 integer (starting from 0)
2. Builds Add gates: `chain[n] + one = chain[n+1]` for n in 1..max_consecutive
3. For each chain value, if a constant with that M31 value was requested, the Add gate outputs to the reserved Var idx; otherwise, outputs to a fresh Var

```rust
use circuits::circuit::Add;
use indexmap::IndexMap;
use stwo::core::fields::qm31::QM31;
use circuits::context::Var;
use std::collections::HashMap;

/// Returns a map from M31-as-u32 value → Var idx for all values in the +1 chain.
/// Also adds the appropriate Add gates to the circuit.
fn build_plus_one_chain(context: &mut Context<impl IValue>) -> HashMap<u32, usize> {
    let constants = context.constants().clone();

    // Find max consecutive M31 integer constant (value is QM31 with only first limb nonzero).
    let max_consecutive = find_max_consecutive(&constants);
    if max_consecutive == 0 {
        let mut map = HashMap::new();
        map.insert(0, context.zero().idx);
        return map;
    }

    let one_idx = context.one().idx;
    let mut chain: HashMap<u32, usize> = HashMap::new();
    chain.insert(0, context.zero().idx);
    chain.insert(1, one_idx);

    let mut prev_idx = one_idx;
    for val in 2..=max_consecutive {
        let qm31_val = QM31::from(val);
        // If this value was requested as a constant, use its reserved idx
        let out_idx = if let Some(var) = constants.get(&qm31_val) {
            var.idx
        } else {
            let var = context.new_var(IValue::from_qm31(qm31_val));
            var.idx
        };
        context.circuit.add.push(Add { in0: prev_idx, in1: one_idx, out: out_idx });
        chain.insert(val, out_idx);
        prev_idx = out_idx;
    }

    chain
}

/// Finds the largest N such that all M31 integers 0..=N are present as constants
/// (embedded as QM31 with only the first coordinate nonzero).
fn find_max_consecutive(constants: &IndexMap<QM31, Var>) -> u32 {
    let mut n = 0u32;
    loop {
        let next = n + 1;
        if constants.contains_key(&QM31::from(next)) {
            n = next;
        } else {
            break;
        }
    }
    n
}
```

Note: `find_max_consecutive` finds the largest N where 1..=N are all present. It doesn't require that ALL constants in that range were requested — it stops at the first gap. However, since the column indices use 0..N_COLUMNS which is consecutive, this will naturally find the right bound.

**Wait — re-reading the user's notes**: "Remove the current +1 gates for the column numbers (and everywhere else it's used), which will create 4000 consecutive constants for Cairo." This means after we convert `generate_column_indices` to use `context.constant()`, those become consecutive M31 constants. So `find_max_consecutive` will find them. But there may also be non-consecutive M31 constants beyond the chain that still need decomposition.

Actually, re-reading more carefully: "create all the +1 consecutive gates until the last consecutive gate. even if it's 3000. But then take the max power of 2 in that range to be that base." So the chain length is determined by how many consecutive M31 values exist, and the base is the largest power of 2 within that range.

**Step 3: Run test**

Run: `cargo test -p circuit-common --release`
Expected: PASS

**Step 4: Commit**

```
feat: implement +1 chain builder for consecutive M31 constants
```

---

## Task 3: Implement zero, one, and u gate construction

**Files:**
- Modify: `crates/circuit_common/src/finalize_constants.rs`

**Step 1: Write a failing test**

```rust
#[test]
fn test_zero_one_u_gates() {
    let mut context = TraceContext::default();
    finalize_builtin_constants(&mut context);
    context.validate_circuit();

    // Zero is constrained by x+x=x (2x=x → x=0)
    // One is constrained by u*x=u (x must be 1)
    // u is inserted externally — needs a yield gate (x+0=x for now, later public logup)
}
```

**Step 2: Implement `finalize_builtin_constants`**

```rust
/// Adds gates for zero, one, and u.
/// - Zero (idx 0): Add { in0: 0, in1: 0, out: 0 } → enforces 2x=x → x=0
/// - One (idx 1): constrained by Mul gate u*one = fresh_var, then Eq fresh_var to u
///   This enforces that one=1 because u*1=u.
///   The Mul gate yields fresh_var. One's yield comes from... we need to think about this.
/// - u (idx 2): inserted via public logup sum. For now, use trivial x+0=x yield gate
///   (to be replaced by public logup sum insertion in a later task).
fn finalize_builtin_constants(context: &mut Context<impl IValue>) {
    let zero_idx = context.zero().idx;
    let one_idx = context.one().idx;
    let u_idx = context.u().idx;

    // Zero: x + x = x → 2x = x → x = 0
    context.circuit.add.push(Add { in0: zero_idx, in1: zero_idx, out: zero_idx });

    // u: for now, trivial yield gate. Will be replaced by public logup sum.
    context.circuit.add.push(Add { in0: u_idx, in1: zero_idx, out: u_idx });

    // One: u * one = u_copy, then eq(u_copy, u) constrains one=1.
    // The Mul gate yields u_copy (a fresh var). One still needs a yield — it will get
    // its yield from the +1 chain: the first chain step is `one + one = two`, which
    // uses one as input but doesn't yield it. One's yield comes from being the output
    // of a construction gate... but one is a guessed var, so we need a yield for it.
    //
    // Solution: one's yield gate is `Add { in0: one_idx, in1: zero_idx, out: one_idx }`
    // (trivial, same as before). The Mul constraint `u * one = u` is an additional
    // soundness constraint, not the yield gate.
    context.circuit.add.push(Add { in0: one_idx, in1: zero_idx, out: one_idx });

    // Add the constraint: u * one = u (constrains one to be 1).
    let u_times_one = context.new_var(context.get(Var { idx: u_idx }) * context.get(Var { idx: one_idx }));
    context.circuit.mul.push(Mul { in0: u_idx, in1: one_idx, out: u_times_one.idx });
    context.circuit.eq.push(Eq { in0: u_times_one.idx, in1: u_idx });
}
```

**Important note on yield invariant**: Each var needs exactly one yield. For `one`, the yield is the trivial `one+0=one` Add gate. The `u*one=u_copy` Mul gate yields `u_copy`, and `eq(u_copy, u)` is an Eq gate that yields nothing. This is sound — `one` is yielded once, `u_copy` is yielded once, `u` is yielded once (by its own `u+0=u` gate).

**Revisit**: Actually, once the `+1` chain is built, `one` will be used as input to Add gates (as `in1`), not as output. So `one` still needs its own yield. The trivial `one+0=one` serves as that yield. The `u*one=u` constraint is separate.

**Step 3: Run test**

Run: `cargo test -p circuit-common --release`
Expected: PASS

**Step 4: Commit**

```
feat: add zero/one/u gate construction in finalize
```

---

## Task 4: Implement M31 constant decomposition using base

**Files:**
- Modify: `crates/circuit_common/src/finalize_constants.rs`

**Step 1: Write a failing test**

```rust
#[test]
fn test_m31_decomposition() {
    let mut context = TraceContext::default();
    // Request some constants beyond the chain
    for i in 0u32..100 {
        context.constant(i.into());
    }
    // Request a large M31 constant
    context.constant(5000u32.into());
    context.constant(100000u32.into());

    finalize_constants(&mut context);
    context.validate_circuit();
}
```

**Step 2: Implement decomposition**

After building the `+1` chain and determining the base (e.g., if chain goes to 100, base=64):

For an M31 constant K not in the chain:
1. Decompose: `c = K % base`, `remainder = K / base`, `b = remainder % base`, `a = remainder / base`
2. Look up `a`, `b`, `c` in the chain (they must be < base, so they're in the chain)
3. Build: `temp = a * base_var + b`, then `result = temp * base_var + c`
4. The final Add gate outputs to the reserved Var idx

```rust
/// Decomposes M31 constants not in the +1 chain using the base.
fn decompose_m31_constants(
    context: &mut Context<impl IValue>,
    chain: &HashMap<u32, usize>,
    base: u32,
) {
    let constants = context.constants().clone();
    let base_idx = *chain.get(&base).expect("base must be in chain");

    for (qm31_val, var) in &constants {
        // Skip non-M31-broadcast constants (handled by QM31 decomposition)
        if !is_m31_broadcast(qm31_val) {
            continue;
        }
        let m31_val = qm31_val.0.0.0 as u32;

        // Skip if already in chain (including zero, one, u-as-integer-if-applicable)
        if chain.contains_key(&m31_val) {
            continue;
        }

        // Decompose: K = (a * base + b) * base + c
        let c = m31_val % base;
        let remainder = m31_val / base;
        let b = remainder % base;
        let a = remainder / base;
        assert!(a < base, "constant {m31_val} requires more than 2 levels of decomposition");

        let a_idx = *chain.get(&a).expect("a must be in chain");
        let b_idx = *chain.get(&b).expect("b must be in chain");
        let c_idx = *chain.get(&c).expect("c must be in chain");

        // temp = a * base + b
        let temp_mul = context.new_var(IValue::from_qm31(QM31::from(a * base)));
        context.circuit.mul.push(Mul { in0: a_idx, in1: base_idx, out: temp_mul.idx });
        let temp_add = context.new_var(IValue::from_qm31(QM31::from(a * base + b)));
        context.circuit.add.push(Add { in0: temp_mul.idx, in1: b_idx, out: temp_add.idx });

        // result = temp * base + c → output to reserved var idx
        let temp_mul2 = context.new_var(IValue::from_qm31(QM31::from((a * base + b) * base)));
        context.circuit.mul.push(Mul { in0: temp_add.idx, in1: base_idx, out: temp_mul2.idx });
        // Final add outputs to the reserved constant's Var idx
        context.circuit.add.push(Add { in0: temp_mul2.idx, in1: c_idx, out: var.idx });
    }
}

/// Returns true if a QM31 value is a "broadcast" M31 value (only first limb nonzero).
fn is_m31_broadcast(val: &QM31) -> bool {
    val.0.1 == 0.into() && val.1.0 == 0.into() && val.1.1 == 0.into()
}
```

**Step 3: Run test**

Run: `cargo test -p circuit-common --release`
Expected: PASS

**Step 4: Commit**

```
feat: implement M31 constant decomposition via power-of-2 base
```

---

## Task 5: Implement QM31 constant construction

**Files:**
- Modify: `crates/circuit_common/src/finalize_constants.rs`

**Step 1: Write a failing test**

```rust
#[test]
fn test_qm31_constant_construction() {
    let mut context = TraceContext::default();
    // Ensure chain has enough range for limbs
    for i in 0u32..20 {
        context.constant(i.into());
    }
    // Request a QM31 constant
    let val = qm31_from_u32s(3, 7, 11, 5);
    context.constant(val);

    finalize_constants(&mut context);
    context.validate_circuit();
}
```

**Step 2: Build QM31 bases from u**

```rust
/// Builds the QM31 basis elements from u.
/// Returns (i_idx, iu_idx) — variable indices for i and iu.
/// i = u^2 - 2 (in the QM31 extension tower)
/// iu = i * u
fn build_qm31_bases(context: &mut Context<impl IValue>, chain: &HashMap<u32, usize>) -> (usize, usize) {
    let u_idx = context.u().idx;
    let two_idx = *chain.get(&2).expect("2 must be in chain");

    // i = u * u - 2
    let u_squared = context.new_var(
        context.get(Var { idx: u_idx }) * context.get(Var { idx: u_idx })
    );
    context.circuit.mul.push(Mul { in0: u_idx, in1: u_idx, out: u_squared.idx });
    let i_var = context.new_var(
        context.get(u_squared) - context.get(Var { idx: two_idx })
    );
    context.circuit.sub.push(Sub { in0: u_squared.idx, in1: two_idx, out: i_var.idx });

    // iu = i * u
    let iu_var = context.new_var(
        context.get(i_var) * context.get(Var { idx: u_idx })
    );
    context.circuit.mul.push(Mul { in0: i_var.idx, in1: u_idx, out: iu_var.idx });

    (i_var.idx, iu_var.idx)
}
```

**Step 3: Decompose general QM31 constants**

```rust
/// Constructs QM31 constants that aren't pure M31 broadcasts.
/// Each QM31 (a, b, c, d) is built as: a_var + b_var*i + c_var*u + d_var*iu
/// where a_var, b_var, c_var, d_var are M31 values from the chain or decomposed.
fn decompose_qm31_constants(
    context: &mut Context<impl IValue>,
    chain: &HashMap<u32, usize>,
    base: u32,
    i_idx: usize,
    iu_idx: usize,
) {
    let constants = context.constants().clone();
    let u_idx = context.u().idx;

    for (qm31_val, var) in &constants {
        if is_m31_broadcast(qm31_val) {
            continue; // Already handled
        }
        // Skip u itself (already has a yield gate)
        if var.idx == context.u().idx {
            continue;
        }

        let limbs = [
            qm31_val.0.0.0 as u32,  // a
            qm31_val.0.1.0 as u32,  // b
            qm31_val.1.0.0 as u32,  // c
            qm31_val.1.1.0 as u32,  // d
        ];

        // Get or build each limb's Var idx
        let limb_idxs: Vec<usize> = limbs.iter().map(|&limb_val| {
            get_or_build_m31_var(context, chain, base, limb_val)
        }).collect();

        // Build: a + b*i + c*u + d*iu
        // b*i
        let bi = context.new_var(IValue::from_qm31(QM31::from(limbs[1]) * qm31_from_u32s(0, 1, 0, 0)));
        context.circuit.mul.push(Mul { in0: limb_idxs[1], in1: i_idx, out: bi.idx });
        // a + b*i
        let a_plus_bi = context.new_var(IValue::from_qm31(
            QM31::from(limbs[0]) + QM31::from(limbs[1]) * qm31_from_u32s(0, 1, 0, 0)
        ));
        context.circuit.add.push(Add { in0: limb_idxs[0], in1: bi.idx, out: a_plus_bi.idx });
        // c*u
        let cu = context.new_var(IValue::from_qm31(QM31::from(limbs[2]) * qm31_from_u32s(0, 0, 1, 0)));
        context.circuit.mul.push(Mul { in0: limb_idxs[2], in1: u_idx, out: cu.idx });
        // d*iu
        let diu = context.new_var(IValue::from_qm31(QM31::from(limbs[3]) * qm31_from_u32s(0, 0, 0, 1)));
        context.circuit.mul.push(Mul { in0: limb_idxs[3], in1: iu_idx, out: diu.idx });
        // c*u + d*iu
        let cu_plus_diu = context.new_var(context.get(cu) + context.get(diu));
        context.circuit.add.push(Add { in0: cu.idx, in1: diu.idx, out: cu_plus_diu.idx });
        // final: (a + b*i) + (c*u + d*iu) → output to reserved var idx
        context.circuit.add.push(Add { in0: a_plus_bi.idx, in1: cu_plus_diu.idx, out: var.idx });
    }
}

/// Gets the chain Var for an M31 value, or decomposes it if not in chain.
/// Returns the Var idx.
fn get_or_build_m31_var(
    context: &mut Context<impl IValue>,
    chain: &HashMap<u32, usize>,
    base: u32,
    val: u32,
) -> usize {
    if let Some(&idx) = chain.get(&val) {
        return idx;
    }
    // Decompose: val = (a * base + b) * base + c
    let c = val % base;
    let remainder = val / base;
    let b = remainder % base;
    let a = remainder / base;

    let base_idx = *chain.get(&base).unwrap();
    let a_idx = *chain.get(&a).expect("a must be in chain");
    let b_idx = *chain.get(&b).expect("b must be in chain");
    let c_idx = *chain.get(&c).expect("c must be in chain");

    let temp_mul = context.new_var(IValue::from_qm31(QM31::from(a * base)));
    context.circuit.mul.push(Mul { in0: a_idx, in1: base_idx, out: temp_mul.idx });
    let temp_add = context.new_var(IValue::from_qm31(QM31::from(a * base + b)));
    context.circuit.add.push(Add { in0: temp_mul.idx, in1: b_idx, out: temp_add.idx });
    let temp_mul2 = context.new_var(IValue::from_qm31(QM31::from((a * base + b) * base)));
    context.circuit.mul.push(Mul { in0: temp_add.idx, in1: base_idx, out: temp_mul2.idx });
    let result = context.new_var(IValue::from_qm31(QM31::from(val)));
    context.circuit.add.push(Add { in0: temp_mul2.idx, in1: c_idx, out: result.idx });

    result.idx
}
```

**Step 4: Run test**

Run: `cargo test -p circuit-common --release`
Expected: PASS

**Step 5: Commit**

```
feat: implement QM31 constant construction from bases
```

---

## Task 6: Wire up `finalize_constants` and update `finalize_context`

**Files:**
- Modify: `crates/circuit_common/src/finalize_constants.rs` (compose the full function)
- Modify: `crates/circuit_common/src/finalize.rs:56-79` (remove hash_constants, call finalize_constants)
- Modify: `crates/circuits/src/context.rs:141-145` (update finalize_guessed_vars to skip constants)

**Step 1: Compose `finalize_constants`**

```rust
pub fn finalize_constants(context: &mut Context<impl IValue>) {
    // 1. Handle zero, one, u
    finalize_builtin_constants(context);

    // 2. Build the +1 chain
    let chain = build_plus_one_chain(context);

    // 3. Determine base
    let max_consecutive = *chain.keys().max().unwrap_or(&1);
    let base = if max_consecutive >= 2 {
        1u32 << (31 - (max_consecutive).leading_zeros() - 1)  // largest power of 2 <= max_consecutive
    } else {
        1
    };

    // 4. Decompose M31 constants not in chain
    decompose_m31_constants(context, &chain, base);

    // 5. Build QM31 bases and decompose QM31 constants
    if has_non_m31_constants(context) {
        let (i_idx, iu_idx) = build_qm31_bases(context, &chain);
        decompose_qm31_constants(context, &chain, base, i_idx, iu_idx);
    }
}
```

**Step 2: Update `finalize_context` in `crates/circuit_common/src/finalize.rs`**

Remove `hash_constants` function and its call. Remove the `output()` calls for hash. Add `finalize_constants()` call:

```rust
pub fn finalize_context(context: &mut Context<impl IValue>) {
    // No more hash_constants — constants are now gate-constructed.
    // Padding the components to a power of two.
    pad_eq(context);
    pad_qm31_ops(context);
    pad_blake(context);
}
```

Note: `finalize_constants` will be called separately, similar to how `finalize_guessed_vars` is called today.

**Step 3: Update `finalize_guessed_vars` to skip constants**

In `context.rs`, modify `finalize_guessed_vars` to skip Vars that are in the constants map (they get their yield gates from `finalize_constants`):

```rust
pub fn finalize_guessed_vars(&mut self) {
    let constant_idxs: HashSet<usize> = self.constants.values().map(|v| v.idx).collect();
    for idx in self.guessed_vars.take().unwrap().iter() {
        if constant_idxs.contains(idx) {
            continue; // Constants get yield gates from finalize_constants
        }
        self.circuit.add.push(Add { in0: *idx, in1: self.zero().idx, out: *idx });
    }
}
```

**Step 4: Run all tests**

Run: `cargo test --release`
Expected: Many tests will fail because callers of `finalize_guessed_vars` don't call `finalize_constants` yet. This is expected — Task 7 fixes the call sites.

**Step 5: Commit**

```
feat: wire up finalize_constants, remove hash_constants from finalize_context
```

---

## Task 7: Update all call sites

**Files:**
- Modify: `crates/circuit_air/src/verify.rs:44` — add finalize_constants call before finalize_guessed_vars
- Modify: `crates/cairo_air/src/verify.rs:117,153` — same
- Modify: `crates/cairo_air/src/statement_test.rs:265` — same
- Modify: `crates/stark_verifier_examples/src/verify_test.rs:45,107` — same
- Modify: `crates/circuit_prover/src/prover_test.rs:82,151,217,286` — same
- Modify: `crates/circuits/src/context_test.rs:16` — same
- Modify: `crates/circuits/src/blake_test.rs:45` — same
- Modify: `crates/circuits/src/ops_test.rs:73` — same
- Modify: `crates/cairo_air/src/test.rs:137` — same
- Modify: `crates/circuit_common/src/preprocessed.rs:463` — finalize_context already calls, but finalize_constants is separate

**Step 1: At every call site of `finalize_guessed_vars`, add `finalize_constants` before it**

Pattern: every `context.finalize_guessed_vars()` becomes:
```rust
circuit_common::finalize_constants::finalize_constants(&mut context);
context.finalize_guessed_vars();
```

Or alternatively, fold `finalize_constants` into `finalize_guessed_vars` — call `finalize_constants` from within `finalize_guessed_vars` itself. This reduces call-site churn. The trade-off is coupling the two, but since `finalize_guessed_vars` already handles the yield invariant for guessed vars and constants are a subset, this is natural.

**Recommended approach**: Make `finalize_guessed_vars` call `finalize_constants` internally:

```rust
pub fn finalize_guessed_vars(&mut self) {
    finalize_constants(self);  // Construct constant gates first
    let constant_idxs: HashSet<usize> = self.constants.values().map(|v| v.idx).collect();
    for idx in self.guessed_vars.take().unwrap().iter() {
        if constant_idxs.contains(idx) {
            continue;
        }
        self.circuit.add.push(Add { in0: *idx, in1: self.zero().idx, out: *idx });
    }
}
```

This way, no call site changes are needed. The only issue is the dependency from `circuits` crate on `circuit_common` — check if this creates a circular dependency. If it does, keep `finalize_constants` as a separate call and update all sites.

**Alternative if circular dependency**: Move `finalize_constants` logic into `circuits` crate itself (into `context.rs`), since it only uses types from `circuits`.

**Step 2: Run all tests**

Run: `cargo test --release`
Expected: PASS (or identify remaining failures)

**Step 3: Commit**

```
feat: update all call sites to use new constant finalization
```

---

## Task 8: Replace `generate_column_indices` with `context.constant()`

**Files:**
- Modify: `crates/stark_verifier/src/sort_queries.rs:23-34`
- Modify: `crates/stark_verifier/src/merkle.rs:145` (call site)

**Step 1: Write test (the existing tests should cover this)**

The existing sort_queries and merkle tests should pass after the change.

**Step 2: Replace `generate_column_indices`**

```rust
pub fn generate_column_indices<Value: IValue>(
    context: &mut Context<Value>,
    n_columns: usize,
) -> Vec<Var> {
    (0..n_columns).map(|i| context.constant((i as u32).into())).collect()
}
```

**Step 3: Run tests**

Run: `cargo test --release`
Expected: PASS

**Step 4: Commit**

```
refactor: replace +1 loop in generate_column_indices with context.constant()
```

---

## Task 9: Remove `hash_constants` and clean up unused Blake imports

**Files:**
- Modify: `crates/circuit_common/src/finalize.rs` — remove `hash_constants` fn, remove blake imports, remove output imports
- Verify no other code calls `hash_constants`

**Step 1: Clean up finalize.rs**

Remove:
- `use circuits::blake::{HashValue, blake};`
- `use circuits::ops::{eq, output};` (if no longer used)
- The `hash_constants` function entirely
- The hash-related lines in `finalize_context`

The `finalize_context` function should now just do padding:
```rust
pub fn finalize_context(context: &mut Context<impl IValue>) {
    pad_eq(context);
    pad_qm31_ops(context);
    pad_blake(context);
}
```

**Step 2: Run tests**

Run: `cargo test --release`
Expected: PASS

**Step 3: Run clippy**

Run: `scripts/clippy.sh`
Expected: PASS (no unused imports)

**Step 4: Commit**

```
refactor: remove hash_constants and unused Blake imports from finalize
```

---

## Task 10: Update existing tests and add comprehensive tests

**Files:**
- Modify: `crates/circuits/src/context_test.rs` — update `test_constants` to expect new gate pattern
- Add tests in `crates/circuit_common/src/finalize_constants.rs`

**Step 1: Update `test_constants` in context_test.rs**

The current test expects:
```
[0] = [0] + [0]
[1] = [1] + [0]
[2] = [2] + [0]
[3] = [3] + [0]
```

After the change, the pattern will be different — constants are constructed via arithmetic gates, not trivial `x+0=x`. Update the assertion to match the new gate pattern.

**Step 2: Add comprehensive tests**

```rust
#[test]
fn test_large_m31_decomposition() {
    let mut context = TraceContext::default();
    for i in 0u32..100 {
        context.constant(i.into());
    }
    context.constant(50000u32.into());
    context.constant(1000000u32.into());
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_qm31_constants() {
    let mut context = TraceContext::default();
    for i in 0u32..20 {
        context.constant(i.into());
    }
    context.constant(qm31_from_u32s(5, 3, 7, 2));
    context.constant(qm31_from_u32s(0, 0, 1, 0)); // u itself
    context.constant(qm31_from_u32s(0, 1, 0, 0)); // i
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_no_constants_beyond_defaults() {
    let mut context = TraceContext::default();
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_mixed_m31_and_qm31_constants() {
    let mut context = TraceContext::default();
    for i in 0u32..500 {
        context.constant(i.into());
    }
    context.constant(qm31_from_u32s(100, 200, 300, 400));
    context.constant(10000u32.into());
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}
```

**Step 3: Run all tests**

Run: `cargo test --release`
Expected: PASS

**Step 4: Commit**

```
test: add comprehensive tests for new constant construction
```

---

## Task 11: End-to-end verification — run full CI suite

**Step 1: Run format check**

Run: `scripts/rust_fmt.sh --check`

**Step 2: Run clippy**

Run: `scripts/clippy.sh`

**Step 3: Run full test suite**

Run: `cargo test --release`

**Step 4: Run machete (unused deps)**

Run: `cargo machete`

**Step 5: Fix any issues found**

**Step 6: Final commit if needed**

```
chore: fix lint/format issues from constants infra redesign
```

---

## Key Risks and Edge Cases

1. **Circular dependency**: `finalize_constants` needs types from `circuits` crate. If placed in `circuit_common`, it can depend on `circuits` (which it already does). If we want to call it from within `context.rs` (`circuits` crate), we'd need the logic there. Recommendation: keep it in `circuit_common` and call it explicitly.

2. **Large constants**: M31 values up to 2^31-1. With base=2^11, decomposition is `(a*2^11+b)*2^11+c`. Max `a` = 2^31 / 2^22 ≈ 512, which fits in the chain if chain ≥ 512. If the chain is smaller, we need a 3-level decomposition or a larger chain. For Cairo's ~4000 consecutive constants, base=2^11=2048, and max `a` for any M31 < 2^31 is ~1024, which fits in the 4000-length chain.

3. **`u` as constant vs external**: Currently `u` = `qm31(0,0,1,0)` is registered as a constant via `context.constant()`. After the change, it's still registered as a constant but its yield gate is temporary (`u+0=u`). In a follow-up, this will be replaced by the public logup sum mechanism. The public logup sum for `u` needs to be added in the `Statement::public_logup_sum` implementation.

4. **Yield invariant**: Every constant Var must be yielded exactly once. The `+1` chain and decomposition gates handle this by outputting directly to reserved Var idxs. Fresh intermediate Vars are yielded by their construction gates. Non-constant guessed Vars get the trivial `x+0=x`. Zero and one have their own explicit yield gates.

5. **Constants that are QM31 basis elements**: Constants like `qm31(0,1,0,0)` (i), `qm31(0,0,1,0)` (u), `qm31(0,0,0,1)` (iu) may be requested by user code (e.g., `from_partial_evals`). These need special handling: `u` already has its yield gate; `i` and `iu` are built by `build_qm31_bases` and if they were requested as constants, their construction gate should output to the reserved idx.
