use std::collections::HashMap;

use indexmap::IndexMap;
use num_traits::Zero;
use stwo::core::fields::qm31::QM31;

use crate::circuit::{Add, Eq, Mul, Sub};
use crate::context::{Context, Var};
use crate::ivalue::{IValue, qm31_from_u32s};

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

    // 5. Extend the chain to at least MIN_BASE, and further if decomposition requires it.
    const MIN_BASE: u32 = 16;
    let min_chain_len = compute_min_chain_length(&constant_idxs, &chain).max(MIN_BASE);
    extend_chain(context, &mut chain, min_chain_len, &constant_idxs);

    // 6. Use the chain length as the base for decomposition.
    let base = *chain.keys().max().unwrap_or(&1);

    // 7. Build a cache of known M31 var indices, seeded from the chain. Decomposition results are
    //    cached here for reuse across constants.
    let mut m31_cache: HashMap<u32, usize> = chain.clone();

    // 8. Decompose M31 constants not in chain.
    if base > 1 {
        decompose_m31_constants(context, &constant_idxs, &mut m31_cache, base);
    }

    // 9. Handle broadcast constants: (x, x, x, x) = x * (1,1,1,1).
    let has_broadcast = constant_idxs.keys().any(|val| is_broadcast(val) && val != &QM31::zero());
    let has_general_qm31 = constant_idxs.keys().any(|val| {
        !is_base_field_element(val) && !is_broadcast(val) && val != &qm31_from_u32s(0, 0, 1, 0)
    });

    if has_broadcast || has_general_qm31 {
        let (i_idx, iu_idx) = build_qm31_bases(context, &m31_cache);

        if has_broadcast {
            let ones_idx = build_ones_vector(context, i_idx, u_idx, iu_idx);
            decompose_broadcast_constants(context, &constant_idxs, &mut m31_cache, base, ones_idx);
        }

        if has_general_qm31 {
            decompose_qm31_constants(
                context,
                &constant_idxs,
                &mut m31_cache,
                base,
                u_idx,
                i_idx,
                iu_idx,
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
    i_idx: usize,
    u_idx: usize,
    iu_idx: usize,
) -> usize {
    let one_idx = context.one().idx;

    // 1 + i
    let one_plus_i_val = context.get(Var { idx: one_idx }) + context.get(Var { idx: i_idx });
    let one_plus_i = context.new_var(one_plus_i_val);
    context.circuit.add.push(Add { in0: one_idx, in1: i_idx, out: one_plus_i.idx });

    // u + iu
    let u_plus_iu_val = context.get(Var { idx: u_idx }) + context.get(Var { idx: iu_idx });
    let u_plus_iu = context.new_var(u_plus_iu_val);
    context.circuit.add.push(Add { in0: u_idx, in1: iu_idx, out: u_plus_iu.idx });

    // (1 + i) + (u + iu)
    let ones_val = one_plus_i_val + u_plus_iu_val;
    let ones = context.new_var(ones_val);
    context.circuit.add.push(Add { in0: one_plus_i.idx, in1: u_plus_iu.idx, out: ones.idx });

    ones.idx
}

/// Constructs broadcast constants (x, x, x, x) as x * (1, 1, 1, 1).
fn decompose_broadcast_constants(
    context: &mut Context<impl IValue>,
    constant_idxs: &IndexMap<QM31, usize>,
    m31_cache: &mut HashMap<u32, usize>,
    base: u32,
    ones_idx: usize,
) {
    let base_idx = *m31_cache.get(&base).expect("base must be in cache");

    for (qm31_val, &reserved_idx) in constant_idxs {
        if !is_broadcast(qm31_val) || qm31_val == &QM31::zero() {
            continue;
        }

        let x = qm31_val.0.0.0;
        let x_idx = get_or_build_m31_var(context, m31_cache, base, base_idx, x);

        // x * (1, 1, 1, 1) → output to reserved idx
        context.circuit.mul.push(Mul { in0: x_idx, in1: ones_idx, out: reserved_idx });
    }
}

/// Computes the minimum chain length needed to support decomposition of all constants.
///
/// For a base B (= chain length), 2-level decomposition can represent values up to B^3 - 1.
/// Additionally, the chain must include 2 if non-base-field constants exist (for i = u^2 - 2).
fn compute_min_chain_length(
    constant_idxs: &IndexMap<QM31, usize>,
    chain: &HashMap<u32, usize>,
) -> u32 {
    let current_max = *chain.keys().max().unwrap_or(&1);

    // Collect all M31 values that need decomposition (not already in chain).
    let mut max_needed: u32 = 0;
    let has_non_base = constant_idxs
        .keys()
        .any(|val| !is_base_field_element(val) && val != &qm31_from_u32s(0, 0, 1, 0));

    for qm31_val in constant_idxs.keys() {
        if is_base_field_element(qm31_val) || qm31_val == &qm31_from_u32s(0, 0, 1, 0) {
            let v = qm31_val.0.0.0;
            if !chain.contains_key(&v) {
                max_needed = max_needed.max(v);
            }
        } else if is_broadcast(qm31_val) {
            // Broadcast (x, x, x, x): only x needs to be available.
            let v = qm31_val.0.0.0;
            if !chain.contains_key(&v) {
                max_needed = max_needed.max(v);
            }
        } else {
            // General QM31: all 4 limbs need to be available.
            for limb in [qm31_val.0.0.0, qm31_val.0.1.0, qm31_val.1.0.0, qm31_val.1.1.0] {
                if !chain.contains_key(&limb) {
                    max_needed = max_needed.max(limb);
                }
            }
        }
    }

    if max_needed == 0 && !has_non_base {
        return current_max;
    }

    // Need base^3 > max_needed (so that a = val/base^2 < base for any val <= max_needed).
    let min_base = if max_needed > 0 {
        // Cube root, rounded up.
        (max_needed as f64).cbrt().ceil() as u32 + 1
    } else {
        2 // Minimum for non-base-field constants (need 2 in chain for i = u^2 - 2).
    };

    // Chain must be at least min_base.
    // Also must be at least 2 for QM31/broadcast bases.
    let needed = if has_non_base { min_base.max(2) } else { min_base };
    current_max.max(needed)
}

/// Finds the largest N such that all M31 integers 1..=N are present as constants.
fn find_max_consecutive(constant_idxs: &IndexMap<QM31, usize>) -> u32 {
    let mut m31_values: Vec<u32> =
        constant_idxs.keys().filter(|v| is_base_field_element(v)).map(|v| v.0.0.0).collect();
    m31_values.sort_unstable();

    // After sorting, a consecutive run from 0 satisfies m31_values[i] == i.
    // (No dedup needed — constants are already deduplicated by the IndexMap.)
    // Binary search for the first index where this property breaks.
    let mut lo = 0usize;
    let mut hi = m31_values.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if m31_values[mid] == mid as u32 {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    // lo is the first index where m31_values[i] != i. The consecutive run is 0..lo.
    if lo <= 1 { 0 } else { (lo - 1) as u32 }
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

        build_m31_from_base(context, m31_cache, base, base_idx, m31_val, reserved_idx);
        m31_cache.insert(m31_val, reserved_idx);
    }
}

/// Builds a single M31 value via decomposition: K = (a * base + b) * base + c.
/// The final Add gate outputs to `out_idx`. Intermediates are cached in the chain for reuse.
fn build_m31_from_base(
    context: &mut Context<impl IValue>,
    m31_cache: &mut HashMap<u32, usize>,
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

    let a_idx = *m31_cache.get(&a).expect("a must be in cache");
    let b_idx = *m31_cache.get(&b).expect("b must be in cache");
    let c_idx = *m31_cache.get(&c).expect("c must be in cache");

    // temp1 = a * base
    let ab = a * base;
    let temp1 = match m31_cache.get(&ab) {
        Some(&idx) => idx,
        None => {
            let var = context.new_var(IValue::from_qm31(QM31::from(ab)));
            context.circuit.mul.push(Mul { in0: a_idx, in1: base_idx, out: var.idx });
            m31_cache.insert(ab, var.idx);
            var.idx
        }
    };

    // temp2 = a * base + b
    let ab_b = ab + b;
    let temp2 = match m31_cache.get(&ab_b) {
        Some(&idx) => idx,
        None => {
            let var = context.new_var(IValue::from_qm31(QM31::from(ab_b)));
            context.circuit.add.push(Add { in0: temp1, in1: b_idx, out: var.idx });
            m31_cache.insert(ab_b, var.idx);
            var.idx
        }
    };

    // temp3 = (a * base + b) * base
    let ab_b_base = ab_b * base;
    let temp3 = match m31_cache.get(&ab_b_base) {
        Some(&idx) => idx,
        None => {
            let var = context.new_var(IValue::from_qm31(QM31::from(ab_b_base)));
            context.circuit.mul.push(Mul { in0: temp2, in1: base_idx, out: var.idx });
            m31_cache.insert(ab_b_base, var.idx);
            var.idx
        }
    };

    // result = (a * base + b) * base + c → output to out_idx
    context.circuit.add.push(Add { in0: temp3, in1: c_idx, out: out_idx });
}

/// Builds the QM31 basis elements from u.
/// i = u^2 - 2, iu = i * u.
/// Returns (i_idx, iu_idx).
fn build_qm31_bases(
    context: &mut Context<impl IValue>,
    m31_cache: &HashMap<u32, usize>,
) -> (usize, usize) {
    let u_idx = context.u().idx;
    let two_idx = *m31_cache.get(&2).expect("2 must be in cache for QM31 base construction");

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
/// If the value is in the chain, returns it directly. Otherwise decomposes it and caches the
/// result (and intermediates) in the chain for reuse.
fn get_or_build_m31_var(
    context: &mut Context<impl IValue>,
    m31_cache: &mut HashMap<u32, usize>,
    base: u32,
    base_idx: usize,
    val: u32,
) -> usize {
    if let Some(&idx) = m31_cache.get(&val) {
        return idx;
    }
    let fresh = context.new_var(IValue::from_qm31(QM31::from(val)));
    build_m31_from_base(context, m31_cache, base, base_idx, val, fresh.idx);
    m31_cache.insert(val, fresh.idx);
    fresh.idx
}

/// Constructs QM31 constants as a + b*i + c*u + d*iu.
fn decompose_qm31_constants(
    context: &mut Context<impl IValue>,
    constant_idxs: &IndexMap<QM31, usize>,
    m31_cache: &mut HashMap<u32, usize>,
    base: u32,
    u_idx: usize,
    i_idx: usize,
    iu_idx: usize,
) {
    let base_idx = *m31_cache.get(&base).expect("base must be in cache");

    for (qm31_val, &reserved_idx) in constant_idxs {
        if is_base_field_element(qm31_val) || is_broadcast(qm31_val) {
            continue;
        }
        if reserved_idx == u_idx {
            continue;
        }

        let limbs = [qm31_val.0.0.0, qm31_val.0.1.0, qm31_val.1.0.0, qm31_val.1.1.0];

        let limb_idxs: [usize; 4] = std::array::from_fn(|j| {
            get_or_build_m31_var(context, m31_cache, base, base_idx, limbs[j])
        });

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
