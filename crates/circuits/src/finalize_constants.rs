use std::collections::HashMap;

use crate::circuit::{Add, Eq, Mul, Sub};
use crate::context::{Context, Var};
use crate::ivalue::{IValue, qm31_from_u32s};
use indexmap::IndexMap;
use stwo::core::fields::qm31::QM31;

#[cfg(test)]
#[path = "finalize_constants_test.rs"]
mod test;

/// Constructs all constants via arithmetic gates, replacing the old guess+hash approach.
///
/// Instead of guessing constants and Blake-hashing them, we derive every constant from `u`
/// (the QM31 extension element `(0,0,1,0)`) using arithmetic gates:
/// - A `+1` chain for consecutive M31 integers
/// - Power-of-2 base decomposition for larger M31 values
/// - QM31 basis combination (`i`, `u`, `iu`) for extension field constants
///
/// The preprocessed trace commits to the circuit structure, implicitly verifying constants.
pub fn finalize_constants(context: &mut Context<impl IValue>) {
    let constant_idxs: IndexMap<QM31, usize> =
        context.constants().iter().map(|(val, var)| (*val, var.idx)).collect();

    // 1. Zero: x + x = x → 2x = x → x = 0.
    let zero_idx = context.zero().idx;
    context.circuit.add.push(Add { in0: zero_idx, in1: zero_idx, out: zero_idx });

    // 2. u: temporary trivial yield (will be replaced by public logup sum).
    let u_idx = context.u().idx;
    context.circuit.add.push(Add { in0: u_idx, in1: zero_idx, out: u_idx });

    // 3. One: trivial yield gate, plus u * one = u constraint to enforce one = 1.
    let one_idx = context.one().idx;
    context.circuit.add.push(Add { in0: one_idx, in1: zero_idx, out: one_idx });
    // Constraint: u * one must equal u.
    let u_times_one_val = context.get(Var { idx: u_idx }) * context.get(Var { idx: one_idx });
    let u_times_one = context.new_var(u_times_one_val);
    context.circuit.mul.push(Mul { in0: u_idx, in1: one_idx, out: u_times_one.idx });
    context.circuit.eq.push(Eq { in0: u_times_one.idx, in1: u_idx });

    // 4. Build the +1 chain for consecutive M31 integer constants.
    let mut chain = build_plus_one_chain(context, &constant_idxs);

    // 5. Extend the chain if needed to support all constants that require decomposition.
    let min_chain_len = compute_min_chain_length(&constant_idxs, &chain);
    extend_chain(context, &mut chain, min_chain_len, &constant_idxs);

    // 6. Use the chain length as the base for decomposition.
    let base = *chain.keys().max().unwrap_or(&1);

    // 7. Decompose M31 constants not in chain.
    if base > 1 {
        decompose_m31_constants(context, &constant_idxs, &chain, base);
    }

    // 8. Build QM31 bases and decompose QM31 constants.
    let has_qm31 = constant_idxs
        .keys()
        .any(|val| !is_m31_broadcast(val) && val != &qm31_from_u32s(0, 0, 1, 0));
    if has_qm31 {
        let (i_idx, iu_idx) = build_qm31_bases(context, &chain);
        decompose_qm31_constants(context, &constant_idxs, &chain, base, u_idx, i_idx, iu_idx);
    }
}

/// Returns true if a QM31 value is a "broadcast" M31 value (only first coordinate nonzero).
fn is_m31_broadcast(val: &QM31) -> bool {
    val.0.1 == 0.into() && val.1.0 == 0.into() && val.1.1 == 0.into()
}

/// Computes the minimum chain length needed to support decomposition of all constants.
///
/// For a base B, 2-level decomposition can represent values up to B^3 - 1.
/// The chain must be at least B long, and B must be a power of 2.
/// Additionally, the chain must include 2 if QM31 constants exist (for i = u^2 - 2).
fn compute_min_chain_length(
    constant_idxs: &IndexMap<QM31, usize>,
    chain: &HashMap<u32, usize>,
) -> u32 {
    let current_max = *chain.keys().max().unwrap_or(&1);

    // Collect all M31 values that need decomposition (not already in chain).
    let mut max_needed: u32 = 0;
    let has_qm31 = constant_idxs
        .keys()
        .any(|val| !is_m31_broadcast(val) && val != &qm31_from_u32s(0, 0, 1, 0));

    for qm31_val in constant_idxs.keys() {
        if is_m31_broadcast(qm31_val) {
            let v = qm31_val.0.0.0;
            if !chain.contains_key(&v) {
                max_needed = max_needed.max(v);
            }
        } else if qm31_val != &qm31_from_u32s(0, 0, 1, 0) {
            // QM31 constant: all 4 limbs need to be available.
            for limb in [qm31_val.0.0.0, qm31_val.0.1.0, qm31_val.1.0.0, qm31_val.1.1.0] {
                let v = limb;
                if !chain.contains_key(&v) {
                    max_needed = max_needed.max(v);
                }
            }
        }
    }

    if max_needed == 0 && !has_qm31 {
        return current_max;
    }

    // Need base^3 > max_needed (so that a = val/base^2 < base for any val <= max_needed).
    let min_base = if max_needed > 0 {
        // Cube root, rounded up.
        let cbrt = (max_needed as f64).cbrt().ceil() as u32 + 1;
        cbrt
    } else {
        2 // Minimum for QM31 (need 2 in chain for i = u^2 - 2).
    };

    // Chain must be at least min_base.
    // Also must be at least 2 for QM31 bases.
    let needed = if has_qm31 { min_base.max(2) } else { min_base };
    current_max.max(needed)
}

/// Finds the largest N such that all M31 integers 1..=N are present as constants.
fn find_max_consecutive(constant_idxs: &IndexMap<QM31, usize>) -> u32 {
    let mut n = 0u32;
    loop {
        let next = n + 1;
        if constant_idxs.contains_key(&QM31::from(next)) {
            n = next;
        } else {
            break;
        }
    }
    n
}

/// Builds the +1 chain: Add gates for 1+1=2, 2+1=3, ..., (N-1)+1=N.
///
/// For each value in the chain, if a constant with that M31 value was requested, the Add gate
/// outputs directly to the reserved Var idx. Otherwise a fresh Var is allocated.
///
/// Returns a map from M31-as-u32 value → Var idx for all values 0..=max_consecutive.
fn build_plus_one_chain(
    context: &mut Context<impl IValue>,
    constant_idxs: &IndexMap<QM31, usize>,
) -> HashMap<u32, usize> {
    let max_consecutive = find_max_consecutive(constant_idxs);

    let mut chain: HashMap<u32, usize> = HashMap::new();
    chain.insert(0, context.zero().idx);
    chain.insert(1, context.one().idx);

    let one_idx = context.one().idx;
    let mut prev_idx = one_idx;

    for val in 2..=max_consecutive {
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

    chain
}

/// Extends the chain with +1 gates from the current max to the target length.
fn extend_chain(
    context: &mut Context<impl IValue>,
    chain: &mut HashMap<u32, usize>,
    target: u32,
    constant_idxs: &IndexMap<QM31, usize>,
) {
    let current_max = *chain.keys().max().unwrap_or(&1);
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
    chain: &HashMap<u32, usize>,
    base: u32,
) {
    let base_idx = *chain.get(&base).expect("base must be in chain");

    for (qm31_val, &reserved_idx) in constant_idxs {
        if !is_m31_broadcast(qm31_val) {
            continue;
        }
        let m31_val = qm31_val.0.0.0;
        if chain.contains_key(&m31_val) {
            continue;
        }

        build_m31_from_base(context, chain, base, base_idx, m31_val, reserved_idx);
    }
}

/// Builds a single M31 value via decomposition: K = (a * base + b) * base + c.
/// The final Add gate outputs to `out_idx`.
fn build_m31_from_base(
    context: &mut Context<impl IValue>,
    chain: &HashMap<u32, usize>,
    base: u32,
    base_idx: usize,
    val: u32,
    out_idx: usize,
) {
    let c = val % base;
    let remainder = val / base;
    let b = remainder % base;
    let a = remainder / base;
    assert!(
        a < base,
        "constant {val} requires more than 2 levels of decomposition (a={a}, base={base})"
    );

    let a_idx = *chain.get(&a).expect("a must be in chain");
    let b_idx = *chain.get(&b).expect("b must be in chain");
    let c_idx = *chain.get(&c).expect("c must be in chain");

    // temp1 = a * base
    let temp1 = context.new_var(IValue::from_qm31(QM31::from(a * base)));
    context.circuit.mul.push(Mul { in0: a_idx, in1: base_idx, out: temp1.idx });

    // temp2 = a * base + b
    let temp2 = context.new_var(IValue::from_qm31(QM31::from(a * base + b)));
    context.circuit.add.push(Add { in0: temp1.idx, in1: b_idx, out: temp2.idx });

    // temp3 = (a * base + b) * base
    let temp3 = context.new_var(IValue::from_qm31(QM31::from((a * base + b) * base)));
    context.circuit.mul.push(Mul { in0: temp2.idx, in1: base_idx, out: temp3.idx });

    // result = (a * base + b) * base + c → output to reserved idx
    context.circuit.add.push(Add { in0: temp3.idx, in1: c_idx, out: out_idx });
}

/// Builds the QM31 basis elements from u.
/// i = u^2 - 2, iu = i * u.
/// Returns (i_idx, iu_idx).
fn build_qm31_bases(
    context: &mut Context<impl IValue>,
    chain: &HashMap<u32, usize>,
) -> (usize, usize) {
    let u_idx = context.u().idx;
    let two_idx = *chain.get(&2).expect("2 must be in chain for QM31 base construction");

    // i = u * u - 2
    let u_sq_val = context.get(Var { idx: u_idx }) * context.get(Var { idx: u_idx });
    let u_squared = context.new_var(u_sq_val);
    context.circuit.mul.push(Mul { in0: u_idx, in1: u_idx, out: u_squared.idx });

    let i_val = u_sq_val - context.get(Var { idx: two_idx });
    let i_var = context.new_var(i_val);
    context.circuit.sub.push(Sub { in0: u_squared.idx, in1: two_idx, out: i_var.idx });

    // iu = i * u
    let iu_val = i_val * context.get(Var { idx: u_idx });
    let iu_var = context.new_var(iu_val);
    context.circuit.mul.push(Mul { in0: i_var.idx, in1: u_idx, out: iu_var.idx });

    (i_var.idx, iu_var.idx)
}

/// Gets or builds a Var for an M31 value. Returns the Var idx.
/// If the value is in the chain, returns it directly. Otherwise decomposes it.
fn get_or_build_m31_var(
    context: &mut Context<impl IValue>,
    chain: &HashMap<u32, usize>,
    base: u32,
    base_idx: usize,
    val: u32,
) -> usize {
    if let Some(&idx) = chain.get(&val) {
        return idx;
    }
    // Decompose into a fresh var.
    let fresh = context.new_var(IValue::from_qm31(QM31::from(val)));
    build_m31_from_base(context, chain, base, base_idx, val, fresh.idx);
    fresh.idx
}

/// Constructs QM31 constants as a + b*i + c*u + d*iu.
fn decompose_qm31_constants(
    context: &mut Context<impl IValue>,
    constant_idxs: &IndexMap<QM31, usize>,
    chain: &HashMap<u32, usize>,
    base: u32,
    u_idx: usize,
    i_idx: usize,
    iu_idx: usize,
) {
    let base_idx = *chain.get(&base).expect("base must be in chain");

    for (qm31_val, &reserved_idx) in constant_idxs {
        if is_m31_broadcast(qm31_val) {
            continue;
        }
        if reserved_idx == u_idx {
            continue;
        }

        let limbs = [
            qm31_val.0.0.0,
            qm31_val.0.1.0,
            qm31_val.1.0.0,
            qm31_val.1.1.0,
        ];

        let limb_idxs: [usize; 4] =
            std::array::from_fn(|j| get_or_build_m31_var(context, chain, base, base_idx, limbs[j]));

        // b * i
        let bi_val = context.get(Var { idx: limb_idxs[1] }) * context.get(Var { idx: i_idx });
        let bi = context.new_var(bi_val);
        context.circuit.mul.push(Mul { in0: limb_idxs[1], in1: i_idx, out: bi.idx });

        // a + b*i
        let a_plus_bi_val = context.get(Var { idx: limb_idxs[0] }) + bi_val;
        let a_plus_bi = context.new_var(a_plus_bi_val);
        context.circuit.add.push(Add { in0: limb_idxs[0], in1: bi.idx, out: a_plus_bi.idx });

        // c * u
        let cu_val = context.get(Var { idx: limb_idxs[2] }) * context.get(Var { idx: u_idx });
        let cu = context.new_var(cu_val);
        context.circuit.mul.push(Mul { in0: limb_idxs[2], in1: u_idx, out: cu.idx });

        // d * iu
        let diu_val = context.get(Var { idx: limb_idxs[3] }) * context.get(Var { idx: iu_idx });
        let diu = context.new_var(diu_val);
        context.circuit.mul.push(Mul { in0: limb_idxs[3], in1: iu_idx, out: diu.idx });

        // c*u + d*iu
        let cu_plus_diu_val = cu_val + diu_val;
        let cu_plus_diu = context.new_var(cu_plus_diu_val);
        context.circuit.add.push(Add { in0: cu.idx, in1: diu.idx, out: cu_plus_diu.idx });

        // (a + b*i) + (c*u + d*iu) → output to reserved idx
        context.circuit.add.push(Add {
            in0: a_plus_bi.idx,
            in1: cu_plus_diu.idx,
            out: reserved_idx,
        });
    }
}
