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
use crate::stark_verifier::merkle::{hash_leaf_qm31, merkle_node, verify_merkle_path};

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
        eprintln!("Length: {}", points_x.len());
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

/// Validates that the values in `fri_input` are consistent with the FRI commitment.
pub fn fri_decommit_<Value: IValue>(
    context: &mut Context<Value>,
    proof: &FriProof<Var>,
    config: &FriConfig,
    fri_input: &[Var],
    bits: &[Vec<Var>],
    points: &CirclePoint<Simd>,
    // base_points[i] are the base points for layer i of all the query points. (i..e. from largest
    // to lowest.)
    base_points: Vec<CirclePoint<Simd>>,
    alphas: &[Var],
) {
    let FriProof {
        commit: FriCommitProof { layer_commitments, last_layer_coefs },
        auth_paths,
        fri_siblings,
    } = proof;

    // ASSUME that fri siblings contains all the required siblings for each tree.
    // e.g. fri siblings is a Vec<Vec<Vec<Var>>> where v[layer][query_idx][k] is the k-th sibling of
    // the the query_idx's query. in layer.
    // Prepare twiddle factors. Which sbould be again a Vec<Vec<Vec<Var>>.
    let mut all_twiddles = vec![];
    let points_y_inv = points.y.inv(context);
    let tmp = Simd::unpack(context, &points_y_inv).into_iter().map(|x| vec![x]).collect();
    all_twiddles.push(tmp);

    let steps: Vec<u32> = vec![]; // Part of config?
        let mut points_x = points.x.clone();
    // let points_x_inv = points_x.inv(context);
    // all_twiddles.push(Simd::unpack(context, &points_x_inv));

    for step in steps[1..].iter() {
        base_

        points_x = double_x_simd(context, &points_x);
        eprintln!("Length: {}", points_x.len());
        let points_x_inv = points_x.inv(context);
        all_twiddles.push(Simd::unpack(context, &points_x_inv));
        points_x = repeated_double_x_simd(context, &points_x);
    }
    let mut points_x = points.x.clone();
    // for (base_point, step) in zip_eq(&base_points[1..], &steps[1..]) { // The first step is the circle to line which is hardcoded to 1 for now.
    //     // For each query point, compute the folding coset containing the query.
    
    //     let coset_x_coords: Vec<Vec<Var>> = compute_coset(context, base_point, *step - 1)
    //         .iter()
    //         .map(|x| {
    //             let x_inv = x.inv(context);
    //             Simd::unpack(context, &x_inv)
    //         })
    //         .collect();
    //     // Transpose
    //     let mut res = vec![];
    //     for i in 0..coset_x_coords.len() {
    //         res.push(coset_x_coords.iter().map(|p| p[i]).collect());
    //     }
    //     all_twiddles.push(res);
    // }

    let mut fri_siblings: Vec<Vec<Vec<Var>>> = vec![];// This will be part of the fri proof. Added here to avoid red squiggles.
    let mut fri_data = fri_input.iter().cloned().collect_vec();
    let mut bit_count = 0;
    for (tree_idx, ((root, twiddles_per_query), step)) in zip_eq(zip_eq(layer_commitments, all_twiddles), steps).enumerate() {
        let siblings_per_query = &fri_siblings[tree_idx];

        let coset_values_per_query = full_coset(context, siblings_per_query, &fri_data, &bits[bit_count..(bit_count + step as usize)]);
        // Check merkle decommitment.
        for (query_idx, coset_values) in coset_values_per_query.iter().enumerate() {
            // Compute one layer of the Merkle tree with the query and its sibling.
            let leaf = hash_leaf_qm31(context, *fri_query);
            let leaf_sibling = hash_leaf_qm31(context, *coset_siblings);

            // Skip the first `tree_idx` LSBs, that are not relevant for this tree.
            let bits_for_query = bits.iter().skip(tree_idx).map(|b| b[query_idx]).collect_vec();
            let node = merkle_node(context, &leaf, &leaf_sibling, bits_for_query[0]);

            let auth_path = auth_paths.at(tree_idx, query_idx); // Need correct auth paths.
            verify_merkle_path(context, node, &bits_for_query[1..], *root, auth_path);
        }

        // Compute the next layer.
        fri_data = zip_eq(coset_values_per_query, twiddles_per_query)
            .map(|(coset_values, twiddles)| {
                fold_coset(context, &coset_values, &twiddles)
            })
            .collect();

        bit_count += step as usize;
    }

    // Check last layer.
    assert_eq!(config.log_n_last_layer_coefs, 0);
    let last_layer_val = last_layer_coefs[0];
    for value in fri_data {
        eq(context, value, last_layer_val);
    }
}

fn compute_coset<Value: IValue>(context: &mut Context<Value>, base_points: &CirclePoint<Simd>, fold_step: u32) -> Vec<Simd> {
    todo!()
}

fn full_coset<Value: IValue>(context: &mut Context<Value>, v: &[Vec<Var>], fri_data: &[Var], bits: &[Vec<Var>]) -> Vec<Vec<Var>> {
    todo!()
}

// twiddles has half the length of coset values.
fn fold_coset<Value: IValue>(context: &mut Context<Value>, coset_values: &[Var], twiddles: &[Var]) -> Var {
    todo!()
}