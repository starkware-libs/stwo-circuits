use std::collections::HashMap;
use std::hash::Hash;

use indexmap::IndexMap;
use itertools::Itertools;
use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::circuit::{Add, Mul, Sub};
use crate::context::{Context, U_VALUE, Var};
use crate::ivalue::{IValue, qm31_from_u32s};
use crate::ops::output;

#[cfg(test)]
#[path = "finalize_constants_test.rs"]
mod test;

/// Wraps `finalize_constants_with_min_base` and calls it with a default value. The main reason
/// is to make testing easier by choosing a smaller minimum base.
pub fn finalize_constants(context: &mut Context<impl IValue>) {
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
pub(crate) fn finalize_constants_with_min_base(
    context: &mut Context<impl IValue>,
    min_base: usize,
) {
    assert!(min_base >= 2);
    let mut m31_constants = IndexMap::<M31, Var>::new();
    let mut qm31_constants = IndexMap::<QM31, Var>::new();
    let mut m31_cache = HashMap::<M31, usize>::new();
    let mut qm31_cache = HashMap::<QM31, usize>::new();
    // Populate the maps.
    context.constants().iter().for_each(|(val, var)| {
        if let [x, M31(0), M31(0), M31(0)] = val.to_m31_array() {
            m31_constants.insert(x, *var);
        } else {
            qm31_constants.insert(*val, *var);
        }
    });
    // TODO(dan): Consider adding the limbs from all cm31s and qm31s into find_max_consecutive.
    let m31_base = find_max_consecutive(&m31_constants).max(min_base);

    // Yield and constrain the `zero` wire by adding a gate x + x = x.
    let zero_idx = context.zero().idx;
    context.circuit.add.push(Add { in0: zero_idx, in1: zero_idx, out: zero_idx });
    assert_eq!(m31_constants.swap_remove(&M31(0)).unwrap().idx, zero_idx);
    m31_cache.insert(M31::zero(), zero_idx);
    qm31_cache.insert(QM31::zero(), zero_idx);

    // TODO(dan): Consider adding `u_inverse` constant with specialized constraints to save gates.
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
    qm31_cache.insert(U_VALUE, qm31_constants.swap_remove(&U_VALUE).unwrap().idx);
    m31_cache.insert(M31(1), m31_constants.swap_remove(&M31(1)).unwrap().idx);

    // Build the +1 chain for consecutive M31 constants.
    build_plus_one_chain(context, &mut m31_constants, &mut m31_cache, m31_base);
    let m31_base = M31::from(m31_base);

    // Decompose M31 constants not in the chain by expressing them in base `m31_base`.
    decompose_m31_constants(context, &mut m31_constants, &mut m31_cache, m31_base);
    assert!(m31_constants.is_empty());

    // Deal with the QM31 constants.
    // Build `i` and `i * u` to get the qm31_basis [i, u, iu].
    // We know `2` is already produced because `min_base >= 2`.
    let two = Var { idx: *m31_cache.get(&2.into()).unwrap() };
    // `u * u = 2 + i`.
    let i_plus_two_val = qm31_from_u32s(2, 1, 0, 0);
    let i_plus_two_var = from_constants_or_new(context, &mut qm31_constants, i_plus_two_val);
    context.circuit.mul.push(Mul { in0: u_var.idx, in1: u_var.idx, out: i_plus_two_var.idx });
    qm31_cache.insert(i_plus_two_val, i_plus_two_var.idx);

    let i_val = qm31_from_u32s(0, 1, 0, 0);
    let i_var = from_constants_or_new(context, &mut qm31_constants, i_val);
    context.circuit.sub.push(Sub { in0: i_plus_two_var.idx, in1: two.idx, out: i_var.idx });
    qm31_cache.insert(i_val, i_var.idx);

    let i_plus_one_val = qm31_from_u32s(1, 1, 0, 0);
    let i_plus_one_var = from_constants_or_new(context, &mut qm31_constants, i_plus_one_val);
    context.circuit.add.push(Add { in0: i_var.idx, in1: one.idx, out: i_plus_one_var.idx });
    qm31_cache.insert(i_plus_one_val, i_plus_one_var.idx);

    let u_plus_iu_val = qm31_from_u32s(0, 0, 1, 1);
    let u_plus_iu_var = from_constants_or_new(context, &mut qm31_constants, u_plus_iu_val);
    context.circuit.mul.push(Mul {
        in0: i_plus_one_var.idx,
        in1: u_var.idx,
        out: u_plus_iu_var.idx,
    });
    qm31_cache.insert(u_plus_iu_val, u_plus_iu_var.idx);

    let ones_val = qm31_from_u32s(1, 1, 1, 1);
    let ones_var = from_constants_or_new(context, &mut qm31_constants, ones_val);
    context.circuit.add.push(Add {
        in0: i_plus_one_var.idx,
        in1: u_plus_iu_var.idx,
        out: ones_var.idx,
    });
    qm31_cache.insert(ones_val, ones_var.idx);

    // Build the broadcast QM31 constants, i.e. constants of the form (x, x, x, x), x != 0.
    decompose_broadcast_constants(
        context,
        &mut qm31_constants,
        &mut m31_cache,
        &mut qm31_cache,
        m31_base,
    );
    // Build the remaining QM31 constants.
    decompose_qm31_constants(
        context,
        &mut qm31_constants,
        &mut m31_cache,
        &mut qm31_cache,
        m31_base,
    );
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
        let var = from_constants_or_new(context, m31_constants, M31::from(val));
        context.circuit.add.push(Add { in0: prev_var.idx, in1: one_idx, out: var.idx });
        m31_cache.insert(val.into(), var.idx);
        prev_var = var;
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
    while let Some(m31_val) = m31_constants.keys().next() {
        build_m31_from_base(context, m31_cache, m31_constants, base, *m31_val);
    }
}

/// Decomposes a value into base `base` limbs and builds it via Horner's evaluation:
/// `val = (...((limbs[n] * base  + limbs[n-1]) * base + limbs[n-2]) * base + ...) + limbs[0]`.
/// Intermediate values are cached in `m31_cache` for reuse.
/// Returns the index of the corresponding var.
fn build_m31_from_base(
    context: &mut Context<impl IValue>,
    m31_cache: &mut HashMap<M31, usize>,
    m31_constants: &mut IndexMap<M31, Var>,
    base: M31,
    val: M31,
) -> usize {
    // If already in cache, return directly.
    if m31_cache.contains_key(&val) {
        return *m31_cache.get(&val).unwrap();
    }

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
        let mul_idx = *m31_cache.entry(mul_val).or_insert_with(|| {
            let var = from_constants_or_new(context, m31_constants, mul_val);
            // Add a gate to yield and constraint `var`.
            context.circuit.mul.push(Mul { in0: acc_idx, in1: base_idx, out: var.idx });
            var.idx
        });

        // Build `acc_val * base + limb`.
        let add_val = mul_val + limb;
        // If add_val is not present in the cache, we add it.
        let add_idx = *m31_cache.entry(add_val).or_insert_with(|| {
            let var = from_constants_or_new(context, m31_constants, add_val);
            context.circuit.add.push(Add { in0: mul_idx, in1: limb_idx, out: var.idx });
            var.idx
        });

        acc_val = add_val;
        acc_idx = add_idx;
    }
    assert!(!m31_constants.contains_key(&val));
    *m31_cache.get(&val).unwrap()
}

/// Yields and constrains every "broadcast" QM31 constant in `qm31_constants` (i.e. constants of
/// the form `(x, x, x, x)` with `x != 0`) by expressing them as `x * (1, 1, 1, 1)`.
///
/// For each broadcast constant, the M31 scalar `x` is retrieved from `m31_cache` (and built via
/// `build_m31_from_base` if missing), and a single Mul gate `x * (1,1,1,1) = (x,x,x,x)` yields and
/// constrains the constant's wire.
///
/// Non-broadcast entries of `qm31_constants` are left untouched; broadcast entries are removed
/// once yielded.
fn decompose_broadcast_constants(
    context: &mut Context<impl IValue>,
    qm31_constants: &mut IndexMap<QM31, Var>,
    m31_cache: &mut HashMap<M31, usize>,
    qm31_cache: &mut HashMap<QM31, usize>,
    base: M31,
) {
    let ones_idx = *qm31_cache.get(&qm31_from_u32s(1, 1, 1, 1)).unwrap();
    qm31_constants.retain(|qm31_value, qm31_var| {
        let m31_value = qm31_value.0.0;
        let is_broadcast = qm31_value.to_m31_array() == [m31_value; 4];
        if !is_broadcast {
            return true;
        }
        let m31_idx =
            build_m31_from_base(context, m31_cache, &mut IndexMap::new(), base, m31_value);
        // Add a gate m31_val * (1, 1, 1, 1) = qm31_var.
        context.circuit.mul.push(Mul { in0: m31_idx, in1: ones_idx, out: qm31_var.idx });
        qm31_cache.insert(*qm31_value, qm31_var.idx);
        // Remove the element from qm31_constants.
        false
    });
}

/// Computes the constant a + bi, adds it to the cache, and returns its index.
fn add_cm31_constant(
    context: &mut Context<impl IValue>,
    qm31_constants: &mut IndexMap<QM31, Var>,
    m31_cache: &mut HashMap<M31, usize>,
    qm31_cache: &mut HashMap<QM31, usize>,
    base: M31,
    i_idx: usize,
    [a, b]: [u32; 2],
) -> usize {
    let [a_idx, b_idx] = [a, b]
        .map(|x| build_m31_from_base(context, m31_cache, &mut IndexMap::new(), base, M31::from(x)));
    if b == 0 {
        // In this case a + bi = a is in the m31_cache but not in qm31_cache.
        return a_idx;
    }

    let bi = qm31_from_u32s(0, b, 0, 0);
    let bi_idx = *qm31_cache.entry(bi).or_insert_with(|| {
        let var = from_constants_or_new(context, qm31_constants, bi);
        context.circuit.mul.push(Mul { in0: i_idx, in1: b_idx, out: var.idx });
        var.idx
    });

    let a_plus_bi = qm31_from_u32s(a, b, 0, 0);
    *qm31_cache.entry(a_plus_bi).or_insert_with(|| {
        let var = from_constants_or_new(context, qm31_constants, a_plus_bi);
        context.circuit.add.push(Add { in0: a_idx, in1: bi_idx, out: var.idx });
        var.idx
    })
}

fn decompose_qm31_constants(
    context: &mut Context<impl IValue>,
    qm31_constants: &mut IndexMap<QM31, Var>,
    m31_cache: &mut HashMap<M31, usize>,
    qm31_cache: &mut HashMap<QM31, usize>,
    base: M31,
) {
    let i_idx = *qm31_cache.get(&qm31_from_u32s(0, 1, 0, 0)).unwrap();
    let u_idx = context.u().idx;

    while let Some(&qm31_val) = qm31_constants.keys().next() {
        let [a, b, c, d]: [u32; 4] = qm31_val.to_m31_array().map(|x| x.0);

        let a_plus_bi_idx =
            add_cm31_constant(context, qm31_constants, m31_cache, qm31_cache, base, i_idx, [a, b]);
        let c_plus_di_idx =
            add_cm31_constant(context, qm31_constants, m31_cache, qm31_cache, base, i_idx, [c, d]);

        let cu_plus_diu = qm31_from_u32s(0, 0, c, d);
        let cu_plus_diu_idx = *qm31_cache.entry(cu_plus_diu).or_insert_with(|| {
            let var = from_constants_or_new(context, qm31_constants, cu_plus_diu);
            context.circuit.mul.push(Mul { in0: c_plus_di_idx, in1: u_idx, out: var.idx });
            var.idx
        });

        let _ = *qm31_cache.entry(qm31_val).or_insert_with(|| {
            let var = from_constants_or_new(context, qm31_constants, qm31_val);
            context.circuit.add.push(Add {
                in0: a_plus_bi_idx,
                in1: cu_plus_diu_idx,
                out: var.idx,
            });
            var.idx
        });
    }
}

/// Returns a variable corresponding to a constant with value `val`.
/// If the constant was requested as a variable in the list of required constants `constants`,
/// we remove it from the list and return it. Otherwise, we create and return a new variable.
fn from_constants_or_new<T>(
    context: &mut Context<impl IValue>,
    constants: &mut IndexMap<T, Var>,
    val: T,
) -> Var
where
    T: Into<QM31> + Eq + Hash,
{
    if let Some(const_var) = constants.swap_remove(&val) {
        const_var
    } else {
        context.new_var(IValue::from_qm31(val.into()))
    }
}
