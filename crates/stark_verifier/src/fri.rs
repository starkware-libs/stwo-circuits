use circuits::utils::select_by_index;
use itertools::{Itertools, zip_eq};
use stwo::core::circle::CirclePoint;

use crate::channel::Channel;
use crate::circle::double_x_simd;
use crate::fri_proof::{FriCommitProof, FriConfig, FriProof};
use crate::merkle::{hash_leaf_qm31, merkle_node, verify_merkle_path};
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::eq;
use circuits::simd::Simd;

#[cfg(test)]
#[path = "fri_test.rs"]
pub mod test;

/// Commits to the FRI layers and returns the random alphas.
pub fn fri_commit(
    context: &mut Context<impl IValue>,
    channel: &mut Channel,
    proof: &FriCommitProof<Var>,
) -> Vec<Var> {
    let mut alphas = Vec::new();
    for root in &proof.layer_commitments {
        channel.mix_commitment(context, *root);
        alphas.push(channel.draw_qm31(context));
    }
    channel.mix_qm31s(context, proof.last_layer_coefs.iter().cloned());

    alphas
}

/// Validates that the values in `fri_input` are consistent with the FRI commitment.
pub fn fri_decommit<Value: IValue>(
    context: &mut Context<Value>,
    proof: &FriProof<Var>,
    config: &FriConfig,
    fri_input: &[Var],
    bits: &[Vec<Var>],
    points: &CirclePoint<Simd>,
    alphas: &[Var],
) {
    let FriProof {
        commit: FriCommitProof { layer_commitments, last_layer_coefs },
        auth_paths,
        fri_siblings,
    } = proof;

    // Prepare twiddle factors.
    let mut all_twiddles = vec![];
    let points_y_inv = points.y.inv(context);
    all_twiddles.push(Simd::unpack(context, &points_y_inv));

    let mut points_x = points.x.clone();
    let points_x_inv = points_x.inv(context);
    all_twiddles.push(Simd::unpack(context, &points_x_inv));

    for _ in 0..(layer_commitments.len() - 2) {
        points_x = double_x_simd(context, &points_x);
        let points_x_inv = points_x.inv(context);
        all_twiddles.push(Simd::unpack(context, &points_x_inv));
    }

    let mut fri_data = fri_input.iter().cloned().collect_vec();
    for (tree_idx, (root, twiddles)) in zip_eq(layer_commitments, all_twiddles).enumerate() {
        let siblings = &fri_siblings[tree_idx];

        // Check merkle decommitment.
        for (query_idx, (fri_query, sibling)) in zip_eq(&fri_data, siblings).enumerate() {
            // Compute one layer of the Merkle tree with the query and its sibling.
            let leaf = hash_leaf_qm31(context, *fri_query);
            let leaf_sibling = hash_leaf_qm31(context, *sibling);

            // Skip the first `tree_idx` LSBs, that are not relevant for this tree.
            let bits_for_query = bits.iter().skip(tree_idx).map(|b| b[query_idx]).collect_vec();
            let node = merkle_node(context, &leaf, &leaf_sibling, bits_for_query[0]);

            let auth_path = auth_paths.at(tree_idx, query_idx);
            verify_merkle_path(context, node, &bits_for_query[1..], *root, auth_path);
        }

        // Compute the next layer.
        fri_data = zip_eq(zip_eq(fri_data, siblings), twiddles)
            .map(|((fri_query, sibling), twiddle)| {
                let g = eval!(context, (fri_query) + (*sibling));
                let h = eval!(context, ((fri_query) - (*sibling)) * (twiddle));
                eval!(context, (g) + ((alphas[tree_idx]) * (h)))
            })
            .collect();
    }

    // Check last layer.
    assert_eq!(config.log_n_last_layer_coefs, 0);
    let last_layer_val = last_layer_coefs[0];
    for value in fri_data {
        eq(context, value, last_layer_val);
    }
}

/// Folds a coset of log size n to a point using the folding coefficients `alphas`.
/// `twiddles_per_fold[i]` contains the twiddles needed at fold i, and has length 2^(n - 1 - i).
#[expect(dead_code)]
fn fold_coset<Value: IValue>(
    context: &mut Context<Value>,
    coset_values: &[Var],
    twiddles_per_fold: &[Vec<Var>],
    alphas: &[Var],
) -> Var {
    assert_eq!(twiddles_per_fold.len(), alphas.len());
    assert_eq!(coset_values.len(), 1 << twiddles_per_fold.len());
    let mut values = coset_values.to_vec();

    for (i, twiddles) in twiddles_per_fold.iter().enumerate() {
        for (j, t) in twiddles.iter().enumerate() {
            let (even, odd) = (values[2 * j], values[2 * j + 1]);
            let g = eval!(context, (even) + (odd));
            let h = eval!(context, ((even) - (odd)) * (*t));
            values[j] = eval!(context, (g) + ((alphas[i]) * (h)));
        }
    }
    values[0]
}

/// Verifies that the query value is in the correct position among the guessed coset values.
///
/// # Arguments
///
/// - `context`: the circuit's context.
/// - `fri_coset_per_query`: for each query, the values of the layer's polynomial on the "line
///   coset" containing the query point. The coset log size is equal to this layer's fri fold step.
/// - `fri_data`: the query values.
/// - `bits`: for each query, the coset log size-many lowest significant bits of the query position.
#[expect(dead_code)]
fn validate_query_position_in_coset<Value: IValue>(
    context: &mut Context<Value>,
    fri_coset_per_query: &[Vec<Var>],
    fri_data: &[Var],
    bits: &[Vec<Var>],
) {
    for (query_idx, (query_value, coset)) in zip_eq(fri_data, fri_coset_per_query).enumerate() {
        let bits: Vec<Var> = bits.iter().map(|b| b[query_idx]).collect();
        let expected_query_value = select_by_index(context, coset, &bits);
        eq(context, *query_value, expected_query_value);
    }
}
