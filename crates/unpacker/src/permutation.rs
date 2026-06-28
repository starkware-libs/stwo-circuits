use std::collections::HashMap;

use circuits::blake::HashValue;
use circuits::circuit::Permutation;
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{Guess, eq};
use circuits::wrappers::{M31Wrapper, U32Wrapper};
use itertools::zip_eq;

#[cfg(test)]
#[path = "permutation_test.rs"]
pub mod test;

/// Constrains `outputs` to be a multiset permutation of `inputs`, treating each **unit** (a
/// fixed-width slice of `u32` words) as an atomic element. All units — inputs and outputs — must
/// have the same width `w`.
///
/// Every word is *tagged* with the index of its source unit in the `u` coordinate — words are
/// packed as `(low_u16, high_u16, 0, 0)`, so the `u` coordinate is free and a tagged word is
/// `(low_u16, high_u16, index, 0)`:
///
/// * each input unit `j` is tagged with the constant `j`;
/// * each output unit is tagged with a single base-field-guessed source index `s`, reused for all
///   `w` of its words.
///
/// A per-column [`Permutation`] gate then ties the tagged inputs to fresh permutation-output
/// variables, and each such variable is constrained equal to the corresponding tagged output. The
/// per-column multiset checks plus the shared-per-unit tag together guarantee each output unit
/// equals exactly one input unit, i.e. `outputs` is a genuine permutation of `inputs`.
///
/// Assumes the words of `inputs` and `outputs` are valid `u32` packings (`(low, high, 0, 0)`), i.e.
/// the [`U32Wrapper`] invariant; the soundness of the tag depends on the `u`/`iu` coordinates being
/// zero.
///
/// Panics if `inputs` and `outputs` have different lengths, if the units have inconsistent widths,
/// or if `outputs` is not a multiset permutation of `inputs` (some output unit has no matching
/// input unit).
pub fn permute_units<Value: IValue>(
    ctx: &mut Context<Value>,
    inputs: &[Vec<U32Wrapper<Var>>],
    outputs: &[Vec<U32Wrapper<Var>>],
) {
    let n = inputs.len();
    assert_eq!(n, outputs.len(), "inputs and outputs must have the same length");
    if n == 0 {
        return;
    }

    // Every unit (input and output) must have the same width, the number of permutation columns.
    let width = inputs[0].len();
    assert!(
        inputs.iter().chain(outputs).all(|u| u.len() == width),
        "all units must have the same width"
    );

    // The `width` u32 words of a unit, used as the lookup key for matching outputs to inputs.
    let key_from_unit = |ctx: &Context<Value>, unit: &[U32Wrapper<Var>]| -> Vec<u32> {
        unit.iter().map(|w| ctx.get(*w.get()).unpack_u32()).collect()
    };

    // Tag each input word with its unit index `j`: `(low, high, 0, 0) + (0, 0, j, 0)`, and map each
    // distinct input-unit value (its `width` u32 words) to the input indices that hold it.
    // Duplicate input units map to several indices and are consumed one per matching output.
    let mut tagged_in: Vec<Vec<Var>> = (0..width).map(|_| Vec::with_capacity(n)).collect();
    let mut indices_by_value: HashMap<Vec<u32>, Vec<usize>> = HashMap::with_capacity(n);
    for (j, unit) in inputs.iter().enumerate() {
        let tag = ctx.constant(qm31_from_u32s(0, 0, j as u32, 0));
        for (col, word) in zip_eq(tagged_in.iter_mut(), unit.iter()) {
            col.push(eval!(ctx, (*word.get()) + (tag)));
        }

        indices_by_value.entry(key_from_unit(ctx, unit)).or_default().push(j);
    }

    // `u = (0, 0, 1, 0)`; multiplying a base-field index `s` by it lifts it to `(0, 0, s, 0)`.
    let u = ctx.u();

    // `tagged_out[c]` collects the fresh permutation-output variables of word-column `c`.
    let mut tagged_out: Vec<Vec<Var>> = (0..width).map(|_| Vec::with_capacity(n)).collect();

    for out_unit in outputs {
        // Find and guess the source index: an as-yet-unused input whose unit equals this output,
        // constrained to the base field and lifted into the `u` coordinate. One guess per output
        // unit, shared across all of its words.
        let index_in_inputs = indices_by_value
            .get_mut(&key_from_unit(ctx, out_unit))
            .and_then(|idxs| idxs.pop())
            .expect("output unit is not a permutation of the inputs");
        let index_var = M31Wrapper::new_unsafe(Value::from_qm31(qm31_from_u32s(
            index_in_inputs as u32,
            0,
            0,
            0,
        )))
        .guess(ctx);
        let tag_u = eval!(ctx, (*index_var.get()) * (u));

        // Tag each word of this unit and add it to its word-column as one permutation output.
        for (col, out_word) in zip_eq(tagged_out.iter_mut(), out_unit.iter()) {
            // Tagged output word: `(low, high, 0, 0) + (0, 0, index, 0)`.
            let tagged_word = eval!(ctx, (*out_word.get()) + (tag_u));
            // The per-column Permutation gate yields its own fresh variable, so allocate one and
            // pin it to `tagged_word` with the `eq`. The gate (added per column below) then proves
            // these tagged outputs are a multiset-permutation of the column's tagged inputs.
            let perm_out = ctx.new_var(ctx.get(tagged_word));
            eq(ctx, perm_out, tagged_word);
            col.push(perm_out);
        }
    }

    // One Permutation gate per word-column: its tagged inputs and permutation outputs must agree as
    // multisets.
    for (in_col, out_col) in zip_eq(tagged_in, tagged_out) {
        ctx.stats.permutation_inputs += in_col.len();
        ctx.circuit.permutation.push(Permutation {
            inputs: in_col.iter().map(|v| v.idx).collect(),
            outputs: out_col.iter().map(|v| v.idx).collect(),
        });
    }
}

/// Constrains `outputs` to be a multiset permutation of `inputs`, treating each [`HashValue`] as an
/// atomic unit. Thin wrapper over [`permute_units`] with a unit width of
/// [`BLAKE2S_DIGEST_N_WORDS`](circuits::blake::BLAKE2S_DIGEST_N_WORDS).
pub fn permute_hash_values<Value: IValue>(
    ctx: &mut Context<Value>,
    inputs: &[HashValue<Var>],
    outputs: &[HashValue<Var>],
) {
    let units = |hashes: &[HashValue<Var>]| -> Vec<Vec<U32Wrapper<Var>>> {
        hashes.iter().map(|h| h.0.to_vec()).collect()
    };
    permute_units(ctx, &units(inputs), &units(outputs));
}
