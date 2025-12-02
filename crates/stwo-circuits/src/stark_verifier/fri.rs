use itertools::{Itertools, zip_eq};
use stwo::core::circle::CirclePoint;

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::eq;
use crate::circuits::simd::Simd;
use crate::eval;
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::circle::double_x_simd;
use crate::stark_verifier::fri_proof::{FriCommitProof, FriConfig, FriProof};
use crate::stark_verifier::merkle::{hash_leaf_qm31, verify_merkle_path};

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
    alphas: &Vec<Var>,
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
        // Check merkle decommitment.
        for (query_idx, fri_query) in fri_data.iter().enumerate() {
            let leaf = hash_leaf_qm31(context, *fri_query);
            let auth_path = auth_paths.at(tree_idx, query_idx);
            let bits_for_query = bits.iter().map(|b| b[query_idx]).collect_vec();
            verify_merkle_path(context, leaf, &bits_for_query[tree_idx..], *root, &auth_path);
        }

        // Compute the next layer.
        fri_data = zip_eq(zip_eq(fri_data, &fri_siblings[tree_idx]), twiddles)
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
