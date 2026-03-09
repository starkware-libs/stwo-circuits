use circuits::blake::HashValue;
use circuits::utils::select_by_index;
use itertools::{Itertools, zip_eq};
use stwo::core::circle::CirclePoint;
use stwo::core::vcs_lifted::verifier::{LOG_PACKED_LEAF_SIZE, PACKED_LEAF_SIZE};

use crate::channel::Channel;
use crate::circle::{
    add_points_simd, compute_half_coset_points, double_x_simd, minus_generator_point_simd,
    repeated_double_point_simd,
};
use crate::fri_proof::{
    FriCommitProof, FriConfig, FriProof, FriWitness, compute_all_line_fold_steps,
};
use crate::merkle::{hash_leaf_qm31, hash_node, hash_packed_leaf_qm31s, verify_merkle_path};
use crate::select_queries::Queries;
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::{eq, mul};
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
    mut bits: &[Vec<Var>],
    queries: Queries,
    alphas: &[Var],
) {
    let FriProof {
        commit: FriCommitProof { layer_commitments, last_layer_coefs },
        auth_paths,
        witness: FriWitness(witness_per_query_per_tree),
    } = proof;
    let all_line_fold_steps = compute_all_line_fold_steps(
        config.log_trace_size - 1 - config.log_n_last_layer_coefs,
        config.line_fold_step,
    );
    // TODO(Leo): remove the hardcoded 1 in next PR.
    let all_fold_steps = [&[1], all_line_fold_steps.as_slice()].concat();
    let n_layers = all_fold_steps.len();
    let mut fri_data = fri_input.to_vec();
    let mut base_point = queries.points.clone();
    let mut packed_bits = queries.bits.as_slice();

    for (tree_idx, ((root, step), witness_per_query)) in
        zip_eq(zip_eq(layer_commitments, all_fold_steps), witness_per_query_per_tree).enumerate()
    {
        let is_circle_to_line = tree_idx == 0;
        let log_layer_size = bits.len();
        let lowest_bits = bits.split_off(..step).unwrap();
        let packed_lowest_bits = packed_bits.split_off(..step).unwrap();
        // Validate that the fri query is in the correct position inside the guessed
        // `witness_per_query`.
        validate_query_position_in_coset(context, witness_per_query, &fri_data, lowest_bits);

        // Check merkle decommitment.
        for (query_idx, witness) in witness_per_query.iter().enumerate() {
            let pack_leaves = log_layer_size >= 2 && step > 1;
            // Compute the leaves.
            let (mut leaves, n_folds): (Vec<HashValue<Var>>, usize) = if pack_leaves {
                (
                    witness
                        .chunks(PACKED_LEAF_SIZE)
                        .map(|chunk| hash_packed_leaf_qm31s(context, chunk.try_into().unwrap()))
                        .collect(),
                    step - LOG_PACKED_LEAF_SIZE as usize,
                )
            } else {
                (witness.iter().map(|val| hash_leaf_qm31(context, *val)).collect(), step)
            };

            // Compute the merkle root of the witness values.
            let witness_root = {
                for fold in 0..n_folds {
                    for i in 0..1 << (n_folds - fold - 1) {
                        let (even, odd) = (leaves[2 * i], leaves[2 * i + 1]);
                        leaves[i] = hash_node(context, even, odd);
                    }
                }
                leaves[0]
            };
            // Verify the rest of the authentication path.
            let auth_path = auth_paths.at(tree_idx, query_idx);
            let bits_for_query = bits.iter().map(|b| b[query_idx]).collect_vec();
            verify_merkle_path(context, witness_root, &bits_for_query, *root, auth_path);
        }

        // Translate base_point to the base of the current circle domain (if we're in the circle to
        // line step) or coset (if we're in a line to line step).
        base_point =
            translate_to_base_point(context, base_point, packed_lowest_bits, is_circle_to_line);
        // Compute twiddles.
        let twiddles_per_fold_per_query =
            compute_twiddles_from_base_point(context, &base_point, step, is_circle_to_line);

        // Compute alpha, alpha^2, ..., alpha^(2^(step - 1));
        let mut alpha_powers = Vec::with_capacity(step);
        let mut alpha_pow = alphas[tree_idx];
        alpha_powers.push(alpha_pow);
        for _ in 0..step - 1 {
            alpha_pow = mul(context, alpha_pow, alpha_pow);
            alpha_powers.push(alpha_pow);
        }

        // Compute the next layer.
        fri_data = zip_eq(witness_per_query, twiddles_per_fold_per_query)
            .map(|(witness, twiddles_per_fold)| {
                fold_coset(context, witness, &twiddles_per_fold, &alpha_powers)
            })
            .collect();

        let n_doubles = if is_circle_to_line { step - 1 } else { step };
        // Don't add unused gates in the last iteration.
        if tree_idx != n_layers - 1 {
            base_point = repeated_double_point_simd(context, &base_point, n_doubles);
        }
    }
    // The last base point's y-coords hasn't been used by `compute_twiddles_from_base_point` if the
    // last step was = 1.
    if *all_line_fold_steps.last().unwrap() == 1 {
        Simd::mark_partly_used(context, &base_point.y);
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
/// - `witness_per_query`: for each query, the values of the layer's polynomial on the coset (in the
///   case of the circle-to-line fold, it's a circle domain) containing the query point. The domain
///   log size is equal to this layer's fri fold step.
/// - `fri_data`: the query values.
/// - `bits`: for each query, the coset log size-many lowest significant bits of the query position.
fn validate_query_position_in_coset<Value: IValue>(
    context: &mut Context<Value>,
    witness_per_query: &[Vec<Var>],
    fri_data: &[Var],
    bits: &[Vec<Var>],
) {
    for (query_idx, (query_value, coset)) in zip_eq(fri_data, witness_per_query).enumerate() {
        let bits: Vec<Var> = bits.iter().map(|b| b[query_idx]).collect();
        let expected_query_value = select_by_index(context, coset, &bits);
        eq(context, *query_value, expected_query_value);
    }
}

/// Computes the twiddles needed for a single FRI fold step.
///
/// For a circle-to-line fold, the first twiddle uses y-coordinates and the rest use x-coordinates.
/// For a line-to-line fold, all twiddles use x-coordinates.
///
/// # Arguments
///
/// - `context`: the circuit's context.
/// - `base_point`: for each query, the first point of the coset of log size `fold_step` that
///   contains the query. More precisely, if the query index has a little-endian bit decomposition
///   a₁a₂a₃a₄...aₙ then its base point is the circle point with index 0...0a_{step + 1}...aₙ. So,
///   for example, for a query with index 101110 and step = 2, its base point has index 001110.
/// - `fold_step`: the folding step for the current FRI fold.
/// - `is_circle_to_line`: whether this is the first (circle-to-line) fold.
fn compute_twiddles_from_base_point<Value: IValue>(
    context: &mut Context<Value>,
    base_point: &CirclePoint<Simd>,
    fold_step: usize,
    is_circle_to_line: bool,
) -> Vec<Vec<Vec<Var>>> {
    assert!(fold_step > 0);
    let n_queries = base_point.x.len();
    let mut twiddles_per_fold_per_query: Vec<Vec<Vec<Var>>> =
        vec![vec![vec![]; fold_step]; n_queries];

    if is_circle_to_line && fold_step == 1 {
        append_twiddles_at_fold(
            context,
            &mut twiddles_per_fold_per_query,
            0,
            std::slice::from_ref(&base_point.y.clone()),
        );
        return twiddles_per_fold_per_query;
    }
    // For circle-to-line, the witness domain is a circle domain with half-coset of size `fold_step
    // - 1`.
    let coset_log_size = if is_circle_to_line { fold_step - 1 } else { fold_step };
    let coset_points = compute_half_coset_points(context, base_point, coset_log_size as u32);

    // The first fold of the circle-to-line step uses y-coordinate twiddles (one per pair of
    // conjugate points). For each point in the half coset, we produce both y and -y - recall that
    // in if i = 0 (mod 4), then the points in the circle domain, in bit-reversed order, at indices
    // [i, i + 3] are of the form (x,y), (x,-y), (-x,-y), (-x,y).
    if is_circle_to_line {
        let zero = Simd::zero(context, n_queries);
        let y_coords: Vec<Simd> = coset_points
            .iter()
            .flat_map(|p| [p.y.clone(), Simd::sub(context, &zero, &p.y)])
            .collect();
        append_twiddles_at_fold(context, &mut twiddles_per_fold_per_query, 0, &y_coords);
    }

    let mut x_coords: Vec<Simd> = coset_points.into_iter().map(|p| p.x).collect();
    let start_fold = usize::from(is_circle_to_line);
    for fold_idx in start_fold..fold_step {
        append_twiddles_at_fold(context, &mut twiddles_per_fold_per_query, fold_idx, &x_coords);
        // Don't add unused gates in the last iteration.
        if fold_idx != fold_step - 1 {
            x_coords = x_coords.iter().step_by(2).map(|x| double_x_simd(context, x)).collect();
        }
    }
    twiddles_per_fold_per_query
}

/// Appends the inverse twiddle factors for a single fold index across all queries.
///
/// For each coordinate in `coords`, computes its inverse and unpacks it. Each scalar is then pushed
/// into the corresponding query's twiddle list at position `fold_idx`.
///
/// # Arguments
///
/// - `context`: the circuit context.
/// - `twiddles_per_fold_per_query`: accumulator indexed as `[query][fold][twiddle_within_fold]`.
/// - `fold_idx`: which fold these twiddles belong to.
/// - `coords`: the SIMD-packed coordinates (y-coordinates for the first circle-to-line fold,
///   x-coordinates otherwise) whose inverses are the twiddle factors.
fn append_twiddles_at_fold<Value: IValue>(
    context: &mut Context<Value>,
    twiddles_per_fold_per_query: &mut [Vec<Vec<Var>>],
    fold_idx: usize,
    coords: &[Simd],
) {
    for coord in coords {
        let coord_inv = coord.inv(context);
        let unpacked = Simd::unpack(context, &coord_inv);
        for (query_twiddles, twiddle) in zip_eq(twiddles_per_fold_per_query.iter_mut(), unpacked) {
            query_twiddles[fold_idx].push(twiddle);
        }
    }
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
fn translate_to_base_point<Value: IValue>(
    context: &mut Context<Value>,
    mut base_point: CirclePoint<Simd>,
    packed_bits: &[Simd],
    is_circle_to_line: bool,
) -> CirclePoint<Simd> {
    let n_queries = base_point.x.len();
    let mut packed_bits = packed_bits;

    if is_circle_to_line {
        let zero = Simd::zero(context, n_queries);
        let minus_y_coord = Simd::sub(context, &zero, &base_point.y);
        let minus_y_point = CirclePoint { x: base_point.x.clone(), y: minus_y_coord };
        // Select between `point` and `point - g_0` (implemented by negating `y`).
        base_point = CirclePoint {
            x: base_point.x.clone(),
            y: Simd::select(context, &packed_bits[0], &base_point.y, &minus_y_point.y),
        };
        packed_bits = &packed_bits[1..];
    }

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
