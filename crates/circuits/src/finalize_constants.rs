use std::collections::HashMap;

use indexmap::IndexMap;
use itertools::Itertools;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::circuit::{Add, Mul};
use crate::context::{Context, Var};
use crate::ivalue::{IValue, qm31_from_u32s};
use crate::ops::{add, output};

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
    let m31_base = find_max_consecutive(&m31_constants).max(min_base);

    // Yield and constrain the `zero` wire by adding a gate x + x = x.
    let zero_idx = context.zero().idx;
    context.circuit.add.push(Add { in0: zero_idx, in1: zero_idx, out: zero_idx });
    m31_cache.insert(M31(0), m31_constants.swap_remove(&M31(0)).unwrap().idx);

    // Yield and constrain the `one` and `u` wires.
    let one = context.one();
    let u_var = context.u();
    // Yield the `one` wire.
    context.circuit.add.push(Add { in0: one.idx, in1: zero_idx, out: one.idx });
    // `u * x = u => x = 1`. This yield `u` and constrains `one`. The value of `u_var` is going to
    // be enforced through a log up constraint in the next verifier.
    context.circuit.mul.push(Mul { in0: u_var.idx, in1: one.idx, out: u_var.idx });
    output(context, u_var);
    // Remove 1 and u from the constants.
    qm31_constants.swap_remove(&qm31_from_u32s(0, 0, 1, 0)).unwrap();
    m31_cache.insert(M31(1), m31_constants.swap_remove(&M31(1)).unwrap().idx);

    // Build the +1 chain for consecutive M31 constants.
    build_plus_one_chain(context, &mut m31_constants, &mut m31_cache, m31_base);
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

/// Builds the +1 chain: Add gates for 1+1=2, 2+1=3, ..., up to `m31_base`. This yields
/// and constrains the output wire.
///
/// For each value, if a constant Var with that M31 value exists, the Add gate uses that Var as exit
/// wire. Otherwise a fresh Var is allocated.
fn build_plus_one_chain(
    context: &mut Context<impl IValue>,
    m31_constants: &mut IndexMap<M31, Var>,
    m31_cache: &mut HashMap<M31, usize>,
    m31_base: usize,
) {
    let one_idx = context.one().idx;
    let mut prev_var = context.one();

    for val in 2..=m31_base {
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
