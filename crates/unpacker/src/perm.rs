use std::collections::HashMap;

use circuits::blake::{BLAKE2S_DIGEST_N_WORDS, HashValue};
use circuits::circuit::Permutation;
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{Guess, eq};
use circuits::wrappers::M31Wrapper;
use itertools::zip_eq;

#[cfg(test)]
#[path = "perm_test.rs"]
pub mod test;

/// Constrains `outputs` to be a multiset permutation of `inputs`, treating each [`HashValue`] as an
/// atomic unit, and returns `outputs`.
///
/// The [`Permutation`] gate only proves multiset equality over a flat list of variables, so the
/// permutation must be applied to each of the [`BLAKE2S_DIGEST_N_WORDS`] word-columns separately.
/// Permuting the columns independently would, on its own, let the prover assemble an output "hash"
/// from words belonging to *different* input hashes (each column only proves its own multiset is
/// preserved). To forbid that, every word is *tagged* with the index of its source hash in the `u`
/// coordinate — words are packed as `(low_u16, high_u16, 0, 0)`, so the `u` coordinate is free and
/// a tagged word is `(low_u16, high_u16, index, 0)`:
///
/// * each input hash `j` is tagged with the constant `j`;
/// * each output hash is tagged with a single base-field-guessed source index `s`, reused for all
///   [`BLAKE2S_DIGEST_N_WORDS`] of its words. Reusing one `s` per output hash is what forces every
///   word of that output to originate from the same input hash.
///
/// A per-column [`Permutation`] gate then ties the tagged inputs to fresh permutation-output
/// variables, and each such variable is constrained equal to the corresponding tagged output. The
/// per-column multiset checks plus the shared-per-hash tag together guarantee each output hash
/// equals exactly one input hash, i.e. `outputs` is a genuine permutation of `inputs`.
///
/// Assumes the words of `inputs` and `outputs` are valid `u32` packings (`(low, high, 0, 0)`), i.e.
/// the [`U32Wrapper`](circuits::wrappers::U32Wrapper) invariant; the soundness of the tag depends
/// on the `u`/`iu` coordinates being zero.
///
/// Panics if `inputs` and `outputs` have different lengths.
pub fn permute_hash_values<Value: IValue>(
    ctx: &mut Context<Value>,
    inputs: impl ExactSizeIterator<Item = HashValue<Var>>,
    outputs: impl ExactSizeIterator<Item = HashValue<Var>>,
) -> Vec<HashValue<Var>> {
    let inputs: Vec<HashValue<Var>> = inputs.collect();
    let outputs: Vec<HashValue<Var>> = outputs.collect();
    let n = inputs.len();
    assert_eq!(n, outputs.len(), "inputs and outputs must have the same length");

    // Tag each input word with its hash index `j`: `(low, high, 0, 0) + (0, 0, j, 0)`.
    let tagged_in: Vec<[Var; BLAKE2S_DIGEST_N_WORDS]> = inputs
        .iter()
        .enumerate()
        .map(|(j, hash)| {
            let tag = ctx.constant(qm31_from_u32s(0, 0, j as u32, 0));
            std::array::from_fn(|c| eval!(ctx, (*hash[c].get()) + (tag)))
        })
        .collect();

    // Map each distinct input-hash value (its eight u32 words) to the input indices that hold it,
    // so the source index for each output can be looked up in O(1) instead of by a linear scan.
    // Duplicate input hashes map to several indices and are consumed one per matching output.
    let mut indices_by_value: HashMap<[u32; BLAKE2S_DIGEST_N_WORDS], Vec<usize>> = HashMap::new();
    for (j, hash) in inputs.iter().enumerate() {
        let key = std::array::from_fn(|c| ctx.get(*hash[c].get()).unpack_u32());
        indices_by_value.entry(key).or_default().push(j);
    }

    // `u = (0, 0, 1, 0)`; multiplying a base-field index `s` by it lifts it to `(0, 0, s, 0)`.
    let u = ctx.u();

    // `permuted[c]` collects the fresh permutation-output variables of word-column `c`.
    let mut permuted: Vec<Vec<Var>> =
        (0..BLAKE2S_DIGEST_N_WORDS).map(|_| Vec::with_capacity(n)).collect();

    for out_hash in &outputs {
        let key: [u32; BLAKE2S_DIGEST_N_WORDS] =
            std::array::from_fn(|c| ctx.get(*out_hash[c].get()).unpack_u32());
        // Witness: an as-yet-unused input index whose hash equals this output (each input index is
        // consumed once). Falls back to 0 if `outputs` is not actually a permutation of `inputs`,
        // which leaves the per-column multiset checks unsatisfiable.
        let src = indices_by_value.get_mut(&key).and_then(|idxs| idxs.pop()).unwrap_or(0);

        // Guess the source index (constrained to the base field) and lift it into the `u`
        // coordinate. One guess per output hash, shared across all of its words.
        let src_var = M31Wrapper::new_unsafe(Value::from_qm31(qm31_from_u32s(src as u32, 0, 0, 0)))
            .guess(ctx);
        let tag_u = eval!(ctx, (*src_var.get()) * (u));

        for (col, out_word) in zip_eq(permuted.iter_mut(), out_hash.iter()) {
            // Tagged output word: `(low, high, 0, 0) + (0, 0, src, 0)`.
            let tagged_out = eval!(ctx, (*out_word.get()) + (tag_u));
            // The per-column Permutation gate yields its own fresh variable, so allocate one and
            // pin it to `tagged_out` with the `eq`. The gate (added per column below) then proves
            // these tagged outputs are a multiset-permutation of the column's tagged inputs.
            let tagged_out_value = ctx.get(tagged_out);
            let perm_out = ctx.new_var(tagged_out_value);
            eq(ctx, perm_out, tagged_out);
            col.push(perm_out);
        }
    }

    // One Permutation gate per word-column: its tagged inputs and permutation outputs must agree as
    // multisets.
    for (c, out_col) in permuted.into_iter().enumerate() {
        let in_col: Vec<usize> = tagged_in.iter().map(|w| w[c].idx).collect();
        ctx.stats.permutation_inputs += in_col.len();
        ctx.circuit
            .permutation
            .push(Permutation { inputs: in_col, outputs: out_col.iter().map(|v| v.idx).collect() });
    }

    outputs
}
