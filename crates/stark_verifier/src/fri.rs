use circuits::blake::HashValue;
use circuits::utils::select_by_index;
use itertools::{Itertools, zip_eq};
use stwo::core::circle::CirclePoint;

use crate::channel::Channel;
use crate::circle::{
    add_points_simd, compute_half_coset_points, double_x_simd, minus_generator_point_simd,
};
use crate::fri_proof::{FriCommitProof, FriConfig, FriProof};
use crate::merkle::{AuthPaths, hash_leaf_qm31, merkle_node, verify_merkle_path};
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

/// Computes the first layer of FRI (circle-to-line fold).
// TODO(Leo): remove the dead_code allow.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn decommit_circle_to_line<Value: IValue>(
    context: &mut Context<Value>,
    root: &HashValue<Var>,
    siblings: &[Var],
    auth_paths: &AuthPaths<Var>,
    fri_input: &[Var],
    bits: &[Vec<Var>],
    points: &CirclePoint<Simd>,
    alpha: Var,
) -> Vec<Var> {
    let points_y_inv = points.y.inv(context);
    let twiddles = Simd::unpack(context, &points_y_inv);
    // Check merkle decommitment.
    for (query_idx, (fri_query, sibling)) in zip_eq(fri_input, siblings).enumerate() {
        // Compute one layer of the Merkle tree with the query and its sibling.
        let leaf = hash_leaf_qm31(context, *fri_query);
        let leaf_sibling = hash_leaf_qm31(context, *sibling);

        let bits_for_query = bits.iter().map(|b| b[query_idx]).collect_vec();
        let node = merkle_node(context, &leaf, &leaf_sibling, bits_for_query[0]);

        let auth_path = auth_paths.at(0, query_idx);
        verify_merkle_path(context, node, &bits_for_query[1..], *root, auth_path);
    }

    // Compute the next layer.
    zip_eq(zip_eq(fri_input, siblings), twiddles)
        .map(|((fri_query, sibling), twiddle)| {
            let g = eval!(context, (*fri_query) + (*sibling));
            let h = eval!(context, ((*fri_query) - (*sibling)) * (twiddle));
            eval!(context, (g) + ((alpha) * (h)))
        })
        .collect()
}

/// Folds a coset of log size n to a point using the folding coefficients `alphas`.
/// `twiddles_per_fold[i]` contains the twiddles needed at fold i, and has length 2^(n - 1 - i).
// TODO(Leo): remove the allow.
#[allow(dead_code)]
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
// TODO(Leo): remove the allow.
#[allow(dead_code)]
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

/// Computes the twiddles needed to fold a line domain of log size `n` to a line domain of log size
/// `n - fold_step`.
///
/// # Arguments
///
/// - `context`: the circuit's context.
/// - `base_point`: for each query, the first point of the coset of log size `fold_step` that
///   contains the query. More precisely, if the query index has a little-endian bit decomposition
///   a₁a₂a₃a₄...aₙ then its base point is the circle point with index 0...0a_{step + 1}...aₙ. So,
///   for example, for a query with index 101110 and step = 2, its base point has index 001110.
/// - `fold_step`: the folding step for the current line-to-line FRI fold.
// TODO(Leo): remove the allow.
#[allow(dead_code)]
fn compute_twiddles_from_base_point<Value: IValue>(
    context: &mut Context<Value>,
    base_point: &CirclePoint<Simd>,
    fold_step: usize,
) -> Vec<Vec<Vec<Var>>> {
    let n_queries = base_point.x.len();
    let mut twiddles_per_fold_per_query: Vec<Vec<Vec<Var>>> =
        vec![vec![vec![]; fold_step]; n_queries];
    let mut x_coords: Vec<Simd> = compute_half_coset_points(context, base_point, fold_step as u32)
        .into_iter()
        .map(|p| p.x)
        .collect();
    for i in 0..fold_step {
        for x in &x_coords {
            let x_inv = x.inv(context);
            let unpacked = Simd::unpack(context, &x_inv);

            for (query_twiddles, twiddle) in zip_eq(&mut twiddles_per_fold_per_query, unpacked) {
                query_twiddles[i].push(twiddle);
            }
        }
        // Don't add unused gates in the last iteration.
        if i != fold_step - 1 {
            x_coords = x_coords.iter().step_by(2).map(|x| double_x_simd(context, x)).collect();
        }
    }
    twiddles_per_fold_per_query
}

/// Translates each packed query point to the base point of its local FRI coset.
///
/// For each SIMD lane `j`, when the corresponding bit is `1`, the function subtracts the circle
/// generator corresponding to this bit position; when it is `0`, it leaves the point unchanged.
///
/// Applying this for all provided bits clears the least significant `step`-many bits of each query.
///
/// # Arguments
///
/// - `context`: the circuit context.
/// - `base_point`: packed query points to translate.
/// - `packed_bits`: the least significant `step`-many bits of the current queries (where `step` is
///   the fold_step of the current FRI fold).
// TODO(Leo): remove the allow.
#[allow(dead_code)]
fn translate_to_base_point<Value: IValue>(
    context: &mut Context<Value>,
    mut base_point: CirclePoint<Simd>,
    packed_bits: &[Simd],
) -> CirclePoint<Simd> {
    let n_queries = base_point.x.len();
    for (i, bit) in packed_bits.iter().enumerate() {
        // The group inverse of the generator of the subgroup of size 2^(i+1).
        let minus_cur_gen_pt = minus_generator_point_simd(context, i + 1, n_queries);
        // Select between `point` and `point - cur_gen_pt`.
        let point_if_bit = add_points_simd(context, &base_point, &minus_cur_gen_pt);
        base_point = CirclePoint {
            x: Simd::select(context, bit, &base_point.x, &point_if_bit.x),
            y: Simd::select(context, bit, &base_point.y, &point_if_bit.y),
        };
    }
    base_point
}

/// For each SIMD lane of `point`, maps the corresponding point to its group inverse if the bit is
/// 1, else leaves it unchanged.
///
/// # Arguments
///
/// - `context`: the circuit context.
/// - `point`: packed input points.
/// - `bit`: SIMD selector (0 or 1) per lane.
// TODO(Leo): remove the allow.
#[allow(dead_code)]
fn translate_by_lsb<Value: IValue>(
    context: &mut Context<Value>,
    point: &CirclePoint<Simd>,
    bit: &Simd,
) -> CirclePoint<Simd> {
    let n_queries = point.x.len();
    let zero = Simd::zero(context, n_queries);
    let minus_y_coord = Simd::sub(context, &zero, &point.y);
    let minus_y_point = CirclePoint { x: point.x.clone(), y: minus_y_coord };
    // Select between `point` and `point - g_0` (implemented by negating `y`).
    CirclePoint { x: point.x.clone(), y: Simd::select(context, bit, &point.y, &minus_y_point.y) }
}
