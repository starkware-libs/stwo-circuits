use circuits::blake::HashValue;
use circuits::utils::select_by_index;
use itertools::{Itertools, zip_eq};
use stwo::core::circle::CirclePoint;

use crate::channel::Channel;
use crate::circle::{
    add_points_simd, compute_half_coset_points, double_x_simd, minus_generator_point_simd,
    repeated_double_point_simd,
};
use crate::fri_proof::{FriCommitProof, FriConfig, FriProof, compute_all_line_fold_steps};
use crate::merkle::{AuthPath, hash_leaf_qm31, hash_node, verify_merkle_path};
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
#[allow(clippy::too_many_arguments)]
pub fn fri_decommit<Value: IValue>(
    context: &mut Context<Value>,
    proof: &FriProof<Var>,
    config: &FriConfig,
    fri_input: &[Var],
    bits: &[Vec<Var>],
    packed_bits: &[Simd],
    points: &CirclePoint<Simd>,
    alphas: &[Var],
) {
    let FriProof {
        commit: FriCommitProof { layer_commitments, last_layer_coefs },
        auth_paths,
        witness_per_query_per_tree,
    } = proof;
    let all_line_fold_steps = compute_all_line_fold_steps(
        config.log_trace_size - config.circle_fold_step - config.log_n_last_layer_coefs,
        config.line_fold_step,
    );
    let all_fold_steps = [&[config.circle_fold_step], all_line_fold_steps.as_slice()].concat();
    let n_layers = all_fold_steps.len();

    let mut bit_counter = 0;
    let mut fri_data = fri_input.to_vec();
    let mut base_point = points.clone();
    
    for (tree_idx, ((root, step), witness_per_query)) in
        zip_eq(zip_eq(layer_commitments, &all_fold_steps), witness_per_query_per_tree)
            .enumerate()
    {
        // The range of the lowest `step`-many significant bits of the current query positions.
        let bit_range = bit_counter..(bit_counter + step);
        let is_circle_to_line = bit_range.contains(&0);
        // Validate that the fri query is in the correct position inside the guessed
        // `fri_coset_per_query`.
        validate_query_position_in_coset(
            context,
            witness_per_query,
            &fri_data,
            &bits[bit_range.clone()],
        );

        // Check merkle decommitment.
        for (query_idx, coset_values) in witness_per_query.iter().enumerate() {
            // Compute the leaves.
            let mut buf: Vec<HashValue<Var>> =
                coset_values.iter().map(|val| hash_leaf_qm31(context, *val)).collect();
            // Compute the the merkle root of the coset values.
            let coset_root = {
                for fold in 0..*step {
                    for i in 0..1 << (step - fold - 1) {
                        let (even, odd) = (buf[2 * i], buf[2 * i + 1]);
                        buf[i] = hash_node(context, even, odd);
                    }
                }
                buf[0]
            };
            // Verify the rest of the authentication path.
            let auth_path = auth_paths.at(tree_idx, query_idx); // We add 1 because the outer loop is 0 based.
            let auth_path = AuthPath(auth_path.0.to_vec());

            let bits_for_query =
                bits.iter().skip(bit_counter + step).map(|b| b[query_idx]).collect_vec();

            verify_merkle_path(context, coset_root, &bits_for_query, *root, &auth_path);
        }

        // Translate base_point to the base of the current coset.
        base_point = translate_to_base_point(
            context,
            base_point,
            &packed_bits[bit_range],
            is_circle_to_line,
        );
        // Compute twiddles.
        let twiddles_per_fold_per_query =
            compute_twiddles_from_base_point(context, &base_point, *step, is_circle_to_line);

        // Compute alpha, alpha^2, ..., alpha^(2^(step - 1));
        let mut alpha_powers = Vec::with_capacity(*step);
        let mut alpha_pow = alphas[tree_idx];
        alpha_powers.push(alpha_pow);
        for _ in 0..step - 1 {
            alpha_pow = mul(context, alpha_pow, alpha_pow);
            alpha_powers.push(alpha_pow);
        }

        // Compute the next layer.
        fri_data = zip_eq(witness_per_query, twiddles_per_fold_per_query)
            .map(|(coset, twiddles_per_fold)| {
                fold_coset(context, coset, &twiddles_per_fold, &alpha_powers)
            })
            .collect();

        bit_counter += step;
        let n_doubles = if is_circle_to_line { *step - 1 } else { *step };
        if tree_idx != n_layers - 1 {
            base_point = repeated_double_point_simd(context, &base_point, n_doubles);
        }
    }
    // The last base point's y-coord may hasn't been used by the compute_twiddles if the last step
    // was = 1.
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
/// - `fri_coset_per_query`: for each query, the values of the layer's polynomial on the "line
///   coset" containing the query point. The coset log size is equal to this layer's fri fold step.
/// - `fri_data`: the query values.
/// - `bits`: for each query, the coset log size-many lowest significant bits of the query position.
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

    let half_coset_log_size = if is_circle_to_line { fold_step - 1 } else { fold_step };
    let coset_points = compute_half_coset_points(context, base_point, half_coset_log_size as u32);

    // At the first fold, circle-to-line uses y-coordinate twiddles.
    if is_circle_to_line {
        let y_coords: Vec<Simd> = if fold_step == 1 {
            coset_points.iter().map(|p| p.y.clone()).collect()
        } else {
            let zero = Simd::zero(context, n_queries);
            coset_points
                .iter()
                .flat_map(|p| [p.y.clone(), Simd::sub(context, &zero, &p.y)])
                .collect()
        };
        append_twiddles_at_fold(
            context,
            &mut twiddles_per_fold_per_query,
            0,
            &y_coords,
        );
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
