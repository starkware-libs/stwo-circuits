use std::collections::HashMap;

use indexmap::IndexMap;
use num_traits::Zero;
use stwo::core::fields::qm31::QM31;

use crate::circuit::{Add, Mul};
use crate::context::{Context, Var};
use crate::eval;
use crate::ivalue::{IValue, qm31_from_u32s};
use crate::ops::{eq, output};

#[cfg(test)]
#[path = "finalize_constants_test.rs"]
mod test;

/// Constructs all constants via arithmetic gates, replacing the old guess+hash approach.
///
/// Instead of guessing constants and Blake-hashing them, we derive every constant from `u`
/// (the QM31 extension element `(0,0,1,0)`) using arithmetic gates:
/// - A `+1` chain for consecutive M31 integers
/// - Base decomposition for larger M31 values: K = (a*base + b)*base + c
/// - Broadcast optimization for (x,x,x,x) constants: x * (1,1,1,1)
/// - QM31 basis combination (`i`, `u`, `iu`) for general extension field constants
///
/// The preprocessed trace commits to the circuit structure, implicitly verifying constants.
pub fn finalize_constants(context: &mut Context<impl IValue>) {
    let constant_idxs: IndexMap<QM31, usize> =
        context.constants().iter().map(|(val, var)| (*val, var.idx)).collect();

    // 1. Zero: x + x = x → 2x = x → x = 0.
    let zero_idx = context.zero().idx;
    context.circuit.add.push(Add { in0: zero_idx, in1: zero_idx, out: zero_idx });

    // 2. u: trivial yield gate + output gate. The output gate increases u's use multiplicity. The
    //    public logup sum must add a corresponding logup_use_term with the hardcoded value
    //    (0,0,1,0) to constrain u's value.
    let u_idx = context.u().idx;
    context.circuit.add.push(Add { in0: u_idx, in1: zero_idx, out: u_idx });
    output(context, context.u());

    // 3. One: trivial yield gate, plus u * one = u constraint to enforce one = 1.
    let one = context.one();
    context.circuit.add.push(Add { in0: one.idx, in1: zero_idx, out: one.idx });
    let u = context.u();
    let u_times_one = eval!(context, (u) * (one));
    eq(context, u_times_one, u);

    // 4. Build the +1 chain for consecutive M31 integer constants.
    let mut chain = build_plus_one_chain(context, &constant_idxs);

    // 5. Extend the chain to at least MIN_BASE, and further if decomposition requires it.
    const MIN_BASE: u32 = 256;
    let min_chain_len = compute_min_chain_length(&constant_idxs, &chain).max(MIN_BASE);
    extend_chain(context, &mut chain, min_chain_len, &constant_idxs);

    // 6. Use the chain length as the base for decomposition.
    let base = *chain.keys().max().unwrap();

    // 7. Build a cache of known M31 var indices, seeded from the chain. Decomposition results are
    //    cached here for reuse across constants.
    let mut m31_cache: HashMap<u32, usize> = chain.clone();

    // 8. Decompose M31 constants not in chain.
    decompose_m31_constants(context, &constant_idxs, &mut m31_cache, base);

    // 9. Handle broadcast constants: (x, x, x, x) = x * (1,1,1,1).
    let has_broadcast = constant_idxs.keys().any(|val| is_broadcast(val) && val != &QM31::zero());
    let has_general_qm31 = constant_idxs.keys().any(|val| {
        !is_base_field_element(val) && !is_broadcast(val) && val != &qm31_from_u32s(0, 0, 1, 0)
    });

    if has_broadcast || has_general_qm31 {
        let u = context.u();
        let (i_var, iu_var) = build_qm31_bases(context, &m31_cache);

        if has_broadcast {
            let ones = build_ones_vector(context, i_var, u, iu_var);
            decompose_broadcast_constants(context, &constant_idxs, &mut m31_cache, base, ones);
        }

        if has_general_qm31 {
            decompose_qm31_constants(
                context,
                &constant_idxs,
                &mut m31_cache,
                base,
                u,
                i_var,
                iu_var,
            );
        }
    }
}

/// Returns true if a QM31 value is a base field (M31) element embedded in QM31,
/// i.e. of the form (x, 0, 0, 0).
fn is_base_field_element(val: &QM31) -> bool {
    val.0.1 == 0.into() && val.1.0 == 0.into() && val.1.1 == 0.into()
}

/// Returns true if a QM31 value is a broadcast: (x, x, x, x).
fn is_broadcast(val: &QM31) -> bool {
    val.0.0 == val.0.1 && val.0.0 == val.1.0 && val.0.0 == val.1.1
}

/// Builds the (1, 1, 1, 1) QM31 value as (1 + i) + (u + iu).
/// Returns the Var idx.
fn build_ones_vector(
    context: &mut Context<impl IValue>,
    i_var: Var,
    u_var: Var,
    iu_var: Var,
) -> Var {
    let one = context.one();
    eval!(context, ((one) + (i_var)) + ((u_var) + (iu_var)))
}

/// Constructs broadcast constants (x, x, x, x) as x * (1, 1, 1, 1).
fn decompose_broadcast_constants(
    context: &mut Context<impl IValue>,
    constant_idxs: &IndexMap<QM31, usize>,
    m31_cache: &mut HashMap<u32, usize>,
    base: u32,
    ones: Var,
) {
    let base_idx = *m31_cache.get(&base).expect("base must be in cache");

    for (qm31_val, &reserved_idx) in constant_idxs {
        if !is_broadcast(qm31_val) || qm31_val == &QM31::zero() {
            continue;
        }

        let x = qm31_val.0.0.0;
        let x_idx = get_or_build_m31_var(context, m31_cache, constant_idxs, base, base_idx, x);

        // x * (1, 1, 1, 1) → output to reserved idx
        context.circuit.mul.push(Mul { in0: x_idx, in1: ones.idx, out: reserved_idx });
    }
}

/// Computes the minimum chain length needed to support non-base-field constants.
/// The chain must include 2 if QM31/broadcast constants exist (for i = u^2 - 2).
/// With dynamic-limb decomposition, any base works for any value, so no minimum
/// base computation is needed beyond this.
fn compute_min_chain_length(
    constant_idxs: &IndexMap<QM31, usize>,
    chain: &HashMap<u32, usize>,
) -> u32 {
    let current_max = *chain.keys().max().unwrap();
    let has_non_base = constant_idxs
        .keys()
        .any(|val| !is_base_field_element(val) && val != &qm31_from_u32s(0, 0, 1, 0));
    if has_non_base { current_max.max(2) } else { current_max }
}

/// Finds the largest N such that all M31 integers 1..=N are present as constants.
fn find_max_consecutive(constant_idxs: &IndexMap<QM31, usize>) -> u32 {
    let mut m31_values: Vec<u32> =
        constant_idxs.keys().filter(|v| is_base_field_element(v)).map(|v| v.0.0.0).collect();
    m31_values.sort_unstable();

    // After sorting, a consecutive run from 0 satisfies m31_values[i] == i.
    let n =
        (0..m31_values.len()).position(|i| m31_values[i] != i as u32).unwrap_or(m31_values.len());
    if n <= 1 { 0 } else { (n - 1) as u32 }
}

/// Builds the +1 chain: Add gates for 1+1=2, 2+1=3, ..., up to the max consecutive M31
/// constant present. Returns a map from M31 value → Var idx for all values 0..=max_consecutive.
///
/// For each value, if a constant with that M31 value was requested, the Add gate outputs
/// directly to the reserved Var idx. Otherwise a fresh Var is allocated.
fn build_plus_one_chain(
    context: &mut Context<impl IValue>,
    constant_idxs: &IndexMap<QM31, usize>,
) -> HashMap<u32, usize> {
    let max_consecutive = find_max_consecutive(constant_idxs);

    let mut chain: HashMap<u32, usize> = HashMap::new();
    chain.insert(0, context.zero().idx);
    chain.insert(1, context.one().idx);

    extend_chain(context, &mut chain, max_consecutive, constant_idxs);
    chain
}

/// Extends the chain with +1 gates from the current max to the target length.
fn extend_chain(
    context: &mut Context<impl IValue>,
    chain: &mut HashMap<u32, usize>,
    target: u32,
    constant_idxs: &IndexMap<QM31, usize>,
) {
    let current_max = *chain.keys().max().unwrap();
    if target <= current_max {
        return;
    }

    let one_idx = context.one().idx;
    let mut prev_idx = *chain.get(&current_max).unwrap();

    for val in (current_max + 1)..=target {
        let qm31_val = QM31::from(val);
        let out_idx = if let Some(&idx) = constant_idxs.get(&qm31_val) {
            idx
        } else {
            context.new_var(IValue::from_qm31(qm31_val)).idx
        };
        context.circuit.add.push(Add { in0: prev_idx, in1: one_idx, out: out_idx });
        chain.insert(val, out_idx);
        prev_idx = out_idx;
    }
}

/// Decomposes M31 constants that aren't in the +1 chain using base decomposition:
/// K = (a * base + b) * base + c, where a, b, c < base.
fn decompose_m31_constants(
    context: &mut Context<impl IValue>,
    constant_idxs: &IndexMap<QM31, usize>,
    m31_cache: &mut HashMap<u32, usize>,
    base: u32,
) {
    let base_idx = *m31_cache.get(&base).expect("base must be in cache");

    for (qm31_val, &reserved_idx) in constant_idxs {
        if !is_base_field_element(qm31_val) {
            continue;
        }
        let m31_val = qm31_val.0.0.0;
        if m31_cache.contains_key(&m31_val) {
            continue;
        }

        build_m31_from_base(
            context,
            m31_cache,
            constant_idxs,
            base,
            base_idx,
            m31_val,
            reserved_idx,
        );
        m31_cache.insert(m31_val, reserved_idx);
    }
}

/// Decomposes a value into base-B limbs and builds it via repeated mul-add:
/// val = (...((limbs[n] * B + limbs[n-1]) * B + limbs[n-2]) * B + ...) + limbs[0].
/// The final gate outputs to `out_idx`. Intermediates are cached in m31_cache for reuse.
/// If an intermediate's value matches a reserved constant, the reserved Var idx is used as
/// the gate output (ensuring it gets its yield gate).
fn build_m31_from_base(
    context: &mut Context<impl IValue>,
    m31_cache: &mut HashMap<u32, usize>,
    constant_idxs: &IndexMap<QM31, usize>,
    base: u32,
    base_idx: usize,
    val: u32,
    out_idx: usize,
) {
    // Decompose val into base-B limbs (least significant first).
    let mut limbs = Vec::new();
    let mut remaining = val;
    while remaining > 0 {
        limbs.push(remaining % base);
        remaining /= base;
    }
    assert!(!limbs.is_empty());
    assert!(*limbs.last().unwrap() < base, "most significant limb must be < base");

    // Build from the most significant limb down: acc = (...((limbs[n] * B + limbs[n-1]) * B)...)
    let mut acc_val = *limbs.last().unwrap();
    let mut acc_idx = *m31_cache.get(&acc_val).expect("limb must be in cache");

    for &limb in limbs.iter().rev().skip(1) {
        let limb_idx = *m31_cache.get(&limb).expect("limb must be in cache");

        // acc = acc * base
        let mul_val = acc_val * base;
        let mul_idx = match m31_cache.get(&mul_val) {
            Some(&idx) => idx,
            None => {
                let idx = var_idx_for_m31(context, constant_idxs, mul_val);
                context.circuit.mul.push(Mul { in0: acc_idx, in1: base_idx, out: idx });
                m31_cache.insert(mul_val, idx);
                idx
            }
        };

        // acc = acc * base + limb
        let add_val = mul_val + limb;
        let add_idx = match m31_cache.get(&add_val) {
            Some(&idx) => idx,
            None => {
                let idx = var_idx_for_m31(context, constant_idxs, add_val);
                context.circuit.add.push(Add { in0: mul_idx, in1: limb_idx, out: idx });
                m31_cache.insert(add_val, idx);
                idx
            }
        };

        acc_val = add_val;
        acc_idx = add_idx;
    }

    // The final value should equal val. Output to out_idx if not already there.
    if m31_cache.get(&val) != Some(&out_idx) {
        let zero_idx = *m31_cache.get(&0).expect("0 must be in cache");
        context.circuit.add.push(Add { in0: acc_idx, in1: zero_idx, out: out_idx });
    }
}

/// Returns the Var idx to use for an M31 value: the reserved constant idx if one exists,
/// otherwise a fresh Var.
fn var_idx_for_m31(
    context: &mut Context<impl IValue>,
    constant_idxs: &IndexMap<QM31, usize>,
    val: u32,
) -> usize {
    if let Some(&idx) = constant_idxs.get(&QM31::from(val)) {
        idx
    } else {
        context.new_var(IValue::from_qm31(QM31::from(val))).idx
    }
}

/// Builds the QM31 basis elements from u.
/// i = u^2 - 2, iu = i * u.
/// Returns (i_idx, iu_idx).
fn build_qm31_bases(
    context: &mut Context<impl IValue>,
    m31_cache: &HashMap<u32, usize>,
) -> (Var, Var) {
    let u = context.u();
    let two =
        Var { idx: *m31_cache.get(&2).expect("2 must be in cache for QM31 base construction") };

    // i = u * u - 2
    let i_var = eval!(context, ((u) * (u)) - (two));
    // iu = i * u
    let iu_var = eval!(context, (i_var) * (u));

    (i_var, iu_var)
}

/// Gets or builds a Var for an M31 value. Returns the Var idx.
/// If the value is in the chain, returns it directly. Otherwise decomposes it and caches the
/// result (and intermediates) in the chain for reuse.
fn get_or_build_m31_var(
    context: &mut Context<impl IValue>,
    m31_cache: &mut HashMap<u32, usize>,
    constant_idxs: &IndexMap<QM31, usize>,
    base: u32,
    base_idx: usize,
    val: u32,
) -> usize {
    if let Some(&idx) = m31_cache.get(&val) {
        return idx;
    }
    let out_idx = var_idx_for_m31(context, constant_idxs, val);
    build_m31_from_base(context, m31_cache, constant_idxs, base, base_idx, val, out_idx);
    m31_cache.insert(val, out_idx);
    out_idx
}

/// Constructs QM31 constants as a + b*i + c*u + d*iu.
fn decompose_qm31_constants(
    context: &mut Context<impl IValue>,
    constant_idxs: &IndexMap<QM31, usize>,
    m31_cache: &mut HashMap<u32, usize>,
    base: u32,
    u_var: Var,
    i_var: Var,
    iu_var: Var,
) {
    let base_idx = *m31_cache.get(&base).expect("base must be in cache");

    for (qm31_val, &reserved_idx) in constant_idxs {
        if is_base_field_element(qm31_val) || is_broadcast(qm31_val) {
            continue;
        }
        if reserved_idx == u_var.idx {
            continue;
        }

        let limbs = [qm31_val.0.0.0, qm31_val.0.1.0, qm31_val.1.0.0, qm31_val.1.1.0];
        let limb_vars: [Var; 4] = std::array::from_fn(|j| Var {
            idx: get_or_build_m31_var(context, m31_cache, constant_idxs, base, base_idx, limbs[j]),
        });
        let [a, b, c, d] = limb_vars;

        // a + b*i + c*u + d*iu → output to reserved idx
        let first_half = eval!(context, (a) + ((b) * (i_var)));
        let second_half = eval!(context, ((c) * (u_var)) + ((d) * (iu_var)));
        context.circuit.add.push(Add {
            in0: first_half.idx,
            in1: second_half.idx,
            out: reserved_idx,
        });
    }
}
