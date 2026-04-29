use std::collections::HashMap;
use std::collections::hash_map::Entry;

use indexmap::IndexMap;
use itertools::Itertools;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::circuit::{Add, Mul};
use crate::context::{Context, Var};
use crate::eval;
use crate::ivalue::{IValue, qm31_from_u32s};
use crate::ops::{add, eq, output};

#[cfg(test)]
#[path = "finalize_constants_test.rs"]
mod test;

/// Wraps `finalize_constants_with_min_base` and calls it with a default value. The main reason
/// is to make testing easier by choosing a smaller minimum base.
// TODO(Leo): remove allow once integrated in the main flow.
#[allow(unused)]
fn finalize_constants(context: &mut Context<impl IValue>) {
    const DEFAULT_MIN_BASE: usize = 256;
    finalize_constants_with_min_base(context, DEFAULT_MIN_BASE);
}

/// Yields and constrains every constant in `context.constants()` via arithmetic gates,
/// All constants are derived from the QM31 extension element `u = (0, 0, 1, 0)` by using:
/// - A `+1` chain for consecutive M31 integer constants.
/// - Base decomposition (with a dynamic base B) for larger M31 values.
/// - Broadcast optimization for constants of the form `(x, x, x, x)`.
/// - QM31 basis combination (`i`, `u`, `iu`) for general extension-field constants.
///
/// # Notes
///
/// The `context.constants()` are tracked in two `IndexMap`s — `m31_constants` for values of the
/// form `(x, 0, 0, 0)` and `qm31_constants` for everything else. As each constant is yielded by a
/// gate (or, for `u`, by the public-output logup term), it is removed from its map. At the end
/// of `finalize_constants` both maps must be empty,
///
/// `m31_cache` maps each M31 value constructed in the process to its Var idx. Every Var in
/// m31_cache is guaranteed to be correctly yielded and constrained. Subsequent decomposition steps
/// (M31 limbs of QM31 constants, broadcast factors, etc.) reuse cached entries instead of
/// rebuilding them, so each distinct M31 value gets at most one yield gate.
///
/// `IndexMap` is used (rather than `HashMap`) so that iteration order is deterministic.
fn finalize_constants_with_min_base(context: &mut Context<impl IValue>, min_base: usize) {
    assert!(min_base >= 2);
    let mut m31_constants = IndexMap::<M31, Var>::new();
    let mut qm31_constants = IndexMap::<QM31, Var>::new();
    let mut m31_cache = HashMap::<M31, usize>::new();
    // Populate the maps.
    context.constants().iter().for_each(|(val, var)| {
        if let [x, M31(0), M31(0), M31(0)] = val.to_m31_array() {
            m31_constants.insert(x, *var);
        } else {
            qm31_constants.insert(*val, *var);
        }
    });
    let target_consecutive = find_max_consecutive(&m31_constants).max(min_base);

    // Yield and constrain the `zero` wire by adding a gate x + x = x.
    let zero_idx = context.zero().idx;
    context.circuit.add.push(Add { in0: zero_idx, in1: zero_idx, out: zero_idx });
    m31_cache.insert(M31(0), m31_constants.swap_remove(&M31(0)).unwrap().idx);

    // Yield the `u` wire by adding a trivial gate x + 0, then add it to the outputs. The constraint
    // on the wire comes from the next verifier which will need to add a logup_use_term with value
    // (0,0,1,0) to its public logup sum.
    let u_var = context.u();
    context.circuit.add.push(Add { in0: u_var.idx, in1: zero_idx, out: u_var.idx });
    output(context, u_var);
    qm31_constants.swap_remove(&qm31_from_u32s(0, 0, 1, 0));

    // Yield the `one` wire by adding a trivial gate x + 0 = x, then constrain it by adding a gate x
    // * u = u.
    let one = context.one();
    context.circuit.add.push(Add { in0: one.idx, in1: zero_idx, out: one.idx });
    let u_times_one = eval!(context, (u_var) * (one));
    eq(context, u_times_one, u_var);
    m31_cache.insert(M31(1), m31_constants.swap_remove(&M31(1)).unwrap().idx);

    // Build the +1 chain for consecutive M31 constants.
    build_plus_one_chain(context, &mut m31_constants, &mut m31_cache, target_consecutive);
    let m31_base = *m31_cache.keys().max().unwrap();

    // Decompose M31 constants not in the chain by expressing them in base `m31_base`.
    decompose_m31_constants(context, &mut m31_constants, &mut m31_cache, m31_base);
    assert!(m31_constants.is_empty());

    // Deal with the QM31 constants.
    // Build `i` and `i * u` to get the qm31_basis [i, u, iu].
    let two = Var { idx: *m31_cache.get(&2.into()).unwrap() };
    let i_var = eval!(context, ((u_var) * (u_var)) - (two));
    let iu_var = eval!(context, (i_var) * (u_var));
    let qm31_basis: [Var; 3] = [i_var, u_var, iu_var];
    // Build the broadcast QM31 constants, i.e. constants of the form (x, x, x, x), x != 0.
    decompose_broadcast_constants(
        context,
        &mut qm31_constants,
        &mut m31_cache,
        m31_base,
        qm31_basis,
    );
    // Build the remaining QM31 constants.
    decompose_qm31_constants(context, &mut qm31_constants, &mut m31_cache, m31_base, qm31_basis);
    assert!(qm31_constants.is_empty());
}

/// Finds the largest integer N such that all values in [0, N] are present as constants.
///
/// # Panics
///
/// Panics if `m31_constants` doesn't contain zero.
fn find_max_consecutive(m31_constants: &IndexMap<M31, Var>) -> usize {
    assert!(m31_constants.contains_key(&M31(0)));
    let m31_values = m31_constants.keys().map(|k| k.0).sorted();
    // After sorting, a consecutive run from 0 satisfies m31_values[i] == i.
    let n_consecutive =
        m31_values.enumerate().position(|(i, v)| i != v as usize).unwrap_or(m31_constants.len());
    // The assert at the beginning ensures that n_consecutive > 0, so this subtraction does not
    // overflow.
    n_consecutive - 1
}

/// Builds the +1 chain: Add gates for 1+1=2, 2+1=3, ..., up to `target_consecutive`. This yields
/// and constrains the output wire.
///
/// For each value, if a constant Var with that M31 value exists, the Add gate uses that Var as exit
/// wire. Otherwise a fresh Var is allocated.
fn build_plus_one_chain(
    context: &mut Context<impl IValue>,
    m31_constants: &mut IndexMap<M31, Var>,
    m31_cache: &mut HashMap<M31, usize>,
    target_consecutive: usize,
) {
    let one_idx = context.one().idx;
    let mut prev_var = context.one();

    for val in 2..=target_consecutive {
        let next_var = if let Some(v) = m31_constants.swap_remove(&M31::from(val)) {
            context.circuit.add.push(Add { in0: prev_var.idx, in1: one_idx, out: v.idx });
            v
        } else {
            add(context, prev_var, context.one())
        };
        m31_cache.insert(val.into(), next_var.idx);
        prev_var = next_var;
    }
}

/// Yields and constrains every remaining M31 constant in `m31_constants` by decomposing it into
/// base `base` limbs and building it via Horner's evaluation.
///
/// All limb values must already be present in `m31_cache` (this is guaranteed when `base` is the
/// length of the `+1` chain built by `build_plus_one_chain`, since limbs are then in `[0, base)`).
/// Each constant yielded is removed from `m31_constants`; intermediate values produced during the
/// reconstruction are added to `m31_cache` and may also be drawn from `m31_constants` if they
/// happen to coincide with a pending constant. The loop terminates with `m31_constants` empty.
fn decompose_m31_constants(
    context: &mut Context<impl IValue>,
    m31_constants: &mut IndexMap<M31, Var>,
    m31_cache: &mut HashMap<M31, usize>,
    base: M31,
) {
    while !m31_constants.is_empty() {
        let m31_val = m31_constants.keys().next().unwrap();
        build_m31_from_base(context, m31_cache, m31_constants, base, *m31_val);
    }
}

/// Decomposes a value into base `base` limbs and builds it via Horner's evaluation:
/// `val = (...((limbs[n] * base  + limbs[n-1]) * base + limbs[n-2]) * base + ...) + limbs[0]`.
/// Intermediate values are cached in `m31_cache` for reuse.
fn build_m31_from_base(
    context: &mut Context<impl IValue>,
    m31_cache: &mut HashMap<M31, usize>,
    m31_constants: &mut IndexMap<M31, Var>,
    base: M31,
    val: M31,
) {
    // Decompose `val` into its base `base` limbs (least significant first).
    let mut limbs = Vec::<M31>::new();
    let mut remaining = val.0;
    while remaining > 0 {
        limbs.push((remaining % base.0).into());
        remaining /= base.0;
    }
    assert!(!limbs.is_empty());

    // Build from the most significant limb down: `acc = (...((limbs[n] * base + limbs[n-1]) *
    // base)...).
    let mut acc_val = limbs.pop().unwrap();
    let mut acc_idx = *m31_cache.get(&acc_val).expect("Limb must be in cache.");
    let base_idx = *m31_cache.get(&base).unwrap();

    for &limb in limbs.iter().rev() {
        let limb_idx = *m31_cache.get(&limb).expect("Limb must be in cache.");

        // Build `acc_val * base`.
        let mul_val = acc_val * base;
        // If mul_val is not present in the cache, we add it.
        if let Entry::Vacant(e) = m31_cache.entry(mul_val) {
            // If it's one of the circuit constants, we take it.
            let var = if let Some(const_var) = m31_constants.swap_remove(&mul_val) {
                const_var
            } else {
                // Otherwise we build a brand new variable.
                context.new_var(IValue::from_qm31(mul_val.into()))
            };
            // Add a gate to yield and constraint `var`.
            context.circuit.mul.push(Mul { in0: acc_idx, in1: base_idx, out: var.idx });
            e.insert(var.idx);
        };
        let mul_idx = *m31_cache.get(&mul_val).unwrap();

        // Build `acc_val * base + limb`.
        let add_val = mul_val + limb;
        // If add_val is not present in the cache, we add it.
        if let Entry::Vacant(e) = m31_cache.entry(add_val) {
            let var = if let Some(const_var) = m31_constants.swap_remove(&add_val) {
                const_var
            } else {
                context.new_var(IValue::from_qm31(add_val.into()))
            };
            context.circuit.add.push(Add { in0: mul_idx, in1: limb_idx, out: var.idx });
            e.insert(var.idx);
        };

        acc_val = add_val;
        acc_idx = *m31_cache.get(&add_val).unwrap();
    }
    assert!(m31_cache.contains_key(&val));
    assert!(!m31_constants.contains_key(&val));
}

fn decompose_broadcast_constants(
    context: &mut Context<impl IValue>,
    qm31_constants: &mut IndexMap<QM31, Var>,
    m31_cache: &mut HashMap<M31, usize>,
    base: M31,
    qm31_basis: [Var; 3],
) {
    let [i_var, u_var, iu_var] = qm31_basis;
    let one = context.one();
    let ones = eval!(context, ((one) + (i_var)) + ((u_var) + (iu_var)));

    qm31_constants.retain(|qm31_value, qm31_var| {
        let is_broadcast = qm31_value.to_m31_array().iter().tuple_windows().all(|(x, y)| x == y);
        if !is_broadcast {
            return true;
        }
        let m31_value = qm31_value.0.0;
        // If m31_value is not in the cache, add it.
        if !m31_cache.contains_key(&m31_value) {
            build_m31_from_base(context, m31_cache, &mut IndexMap::new(), base, m31_value);
        }
        let m31_idx = *m31_cache.get(&m31_value).unwrap();
        // Add a gate m31_val * (1, 1, 1, 1) = qm31_var.
        context.circuit.mul.push(Mul { in0: m31_idx, in1: ones.idx, out: qm31_var.idx });
        // Remove the element from qm31_constants.
        false
    });
}

fn decompose_qm31_constants(
    context: &mut Context<impl IValue>,
    qm31_constants: &mut IndexMap<QM31, Var>,
    m31_cache: &mut HashMap<M31, usize>,
    base: M31,
    qm_basis: [Var; 3],
) {
    let [i_var, u_var, iu_var] = qm_basis;

    for (qm31_value, qm31_var) in qm31_constants.drain(..) {
        let limbs = qm31_value.to_m31_array();
        let [a, b, c, d]: [Var; 4] = std::array::from_fn(|j| {
            let m31_val = limbs[j];
            if !m31_cache.contains_key(&m31_val) {
                build_m31_from_base(context, m31_cache, &mut IndexMap::new(), base, m31_val);
            }
            Var { idx: *m31_cache.get(&m31_val).unwrap() }
        });

        // a + b*i + c*u + d*iu → output to reserved idx
        let first_half = eval!(context, (a) + ((b) * (i_var)));
        let second_half = eval!(context, ((c) * (u_var)) + ((d) * (iu_var)));
        context.circuit.add.push(Add {
            in0: first_half.idx,
            in1: second_half.idx,
            out: qm31_var.idx,
        });
    }
}
