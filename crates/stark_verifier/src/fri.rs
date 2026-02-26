use circuits::blake::{HashValue, blake};
use circuits::utils::select_by_index;
use itertools::{Itertools, zip_eq};
use stwo::core::circle::CirclePoint;
use stwo::core::fields::FieldExpOps;

use crate::channel::Channel;
use crate::circle::{
    add_points_simd, double_x_simd, generator_point_simd, repeated_double_point_simd,
};
use crate::fri_proof::{FriCommitProof, FriConfig, FriProof};
use crate::merkle::{
    AuthPath, AuthPaths, hash_leaf_qm31, hash_node, merkle_node, verify_merkle_path,
};
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{eq, mul, pointwise_mul};
use circuits::simd::Simd;
use circuits::wrappers::M31Wrapper;

#[cfg(test)]
#[path = "fri_test.rs"]
pub mod test;

const PACKED_FRI_LEAF_LOG_SIZE: usize = 2;
const PACKED_FRI_LEAF_SIZE: usize = 1 << PACKED_FRI_LEAF_LOG_SIZE;

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

#[allow(clippy::too_many_arguments)]
/// Validates that the values in `fri_input` are consistent with the FRI commitment.
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
        circle_fri_siblings,
        line_coset_vals_per_query_per_tree,
    } = proof;
    let steps = &config.line_fold_steps_aux;
    let n_inner_layers = steps.len();
    let mut layer_log_size = config.log_evaluation_domain_size() - 1;
    let first_layer_pack = config.pack_leaves && config.log_evaluation_domain_size() >= 2;
    // Circle to line decommitment.
    let mut fri_data = decommit_circle_to_line(
        context,
        &layer_commitments[0],
        circle_fri_siblings,
        auth_paths,
        fri_input,
        bits,
        points,
        alphas[0],
        first_layer_pack,
    );
    // Line to line decommitment.
    let mut base_point = translate_by_lsb(context, points, &packed_bits[0]);
    let mut bit_counter = 0;
    let mut total_line_to_line_updates = 0usize;

    for (tree_idx, ((root, step), coset_per_query)) in
        zip_eq(zip_eq(&layer_commitments[1..], steps), line_coset_vals_per_query_per_tree)
            .enumerate()
    {
        // The range of the lowest `step`-many significant bits of the current query positions.
        let bit_range = (1 + bit_counter)..(1 + bit_counter + step);
        // Validate that the fri query is in the correct position inside the guessed
        // `fri_coset_per_query`.
        let pack_layer = config.pack_leaves && layer_log_size >= 2;
        if pack_layer && *step == 1 {
            for (query_idx, (query_value, coset_values)) in
                zip_eq(&fri_data, coset_per_query).enumerate()
            {
                assert_eq!(coset_values.len(), PACKED_FRI_LEAF_SIZE);
                let inner_bit = bits[1 + bit_counter][query_idx];
                let extra_bit = bits[1 + bit_counter + 1][query_idx];
                let should_be_query =
                    select_by_index(context, coset_values, &[inner_bit, extra_bit]);
                eq(context, *query_value, should_be_query);
            }
        } else {
            validate_query_position_in_coset(
                context,
                coset_per_query,
                &fri_data,
                &bits[bit_range.clone()],
            );
        }

        // Check merkle decommitment.
        let mut hash_leaves_updates = 0usize;
        let mut coset_root_updates = 0usize;
        let mut auth_updates = 0usize;
        for (query_idx, coset_values) in coset_per_query.iter().enumerate() {
            let coset_root = if pack_layer {
                assert!(
                    coset_values.len() == (1 << *step) || (*step == 1 && coset_values.len() == 4)
                );
                let leaf_hashes = if *step == 1 {
                    let before = context.stats.blake_updates;
                    let leaf = hash_packed_fri_leaf(context, coset_values);
                    hash_leaves_updates += context.stats.blake_updates - before;
                    vec![leaf]
                } else {
                    let before = context.stats.blake_updates;
                    let hashes = coset_values
                        .chunks(PACKED_FRI_LEAF_SIZE)
                        .map(|chunk| hash_packed_fri_leaf(context, chunk))
                        .collect_vec();
                    hash_leaves_updates += context.stats.blake_updates - before;
                    hashes
                };
                let before = context.stats.blake_updates;
                let root = fold_hashes_to_root(
                    context,
                    leaf_hashes,
                    step.saturating_sub(PACKED_FRI_LEAF_LOG_SIZE),
                );
                coset_root_updates += context.stats.blake_updates - before;
                root
            } else {
                let before = context.stats.blake_updates;
                let mut buf: Vec<HashValue<Var>> =
                    coset_values.iter().map(|val| hash_leaf_qm31(context, *val)).collect();
                hash_leaves_updates += context.stats.blake_updates - before;
                let before = context.stats.blake_updates;
                for fold in 0..*step {
                    for i in 0..1 << (step - fold - 1) {
                        let (even, odd) = (buf[2 * i], buf[2 * i + 1]);
                        buf[i] = hash_node(context, even, odd);
                    }
                }
                coset_root_updates += context.stats.blake_updates - before;
                buf[0]
            };
            // Verify the rest of the authentication path.
            let auth_path = auth_paths.at(tree_idx + 1, query_idx); // We add 1 because the outer loop is 0 based.
            let auth_path = AuthPath(auth_path.0.to_vec());

            let bits_for_query = bits.iter().map(|b| b[query_idx]).collect_vec();
            let bits_used_by_path = auth_path.0.len();
            let bits_tail = &bits_for_query[(bits_for_query.len() - bits_used_by_path)..];
            let before = context.stats.blake_updates;
            verify_merkle_path(context, coset_root, bits_tail, *root, &auth_path);
            auth_updates += context.stats.blake_updates - before;
        }
        println!(
            "Line-to-line {tree_idx}-th inner layer | Hash leaves | # updates: {}",
            hash_leaves_updates
        );
        println!(
            "Line-to-line {tree_idx}-th inner layer | Coset root | # updates: {}",
            coset_root_updates
        );
        println!(
            "Line-to-line {tree_idx}-th inner layer | Authentication | # updates: {}",
            auth_updates
        );
        total_line_to_line_updates += hash_leaves_updates + coset_root_updates + auth_updates;

        // Translate base_point to the base of the current coset.
        base_point = translate_base_point(context, base_point, &packed_bits[bit_range]);
        // Compute twiddles.
        let twiddles_per_fold_per_query =
            compute_twiddles_from_base_point(context, &base_point, *step);

        // Compute alpha, alpha^2, ..., alpha^(2^(step - 1));
        let mut alpha_powers = Vec::with_capacity(*step);
        let mut alpha_pow = alphas[tree_idx + 1];
        alpha_powers.push(alpha_pow);
        for _ in 0..step - 1 {
            alpha_pow = mul(context, alpha_pow, alpha_pow);
            alpha_powers.push(alpha_pow);
        }

        // Compute the next layer.
        if pack_layer && *step == 1 {
            fri_data = zip_eq(coset_per_query, twiddles_per_fold_per_query)
                .enumerate()
                .map(|(query_idx, (coset, twiddles_per_fold))| {
                    let extra_bit = bits[1 + bit_counter + 1][query_idx];
                    let even =
                        eval!(context, (coset[0]) + ((extra_bit) * ((coset[2]) - (coset[0]))));
                    let odd =
                        eval!(context, (coset[1]) + ((extra_bit) * ((coset[3]) - (coset[1]))));
                    fold_coset(context, &[even, odd], &twiddles_per_fold, &alpha_powers)
                })
                .collect();
        } else {
            fri_data = zip_eq(coset_per_query, twiddles_per_fold_per_query)
                .map(|(coset, twiddles_per_fold)| {
                    fold_coset(context, coset, &twiddles_per_fold, &alpha_powers)
                })
                .collect();
        }

        // Don't add unused gates in the last iteration.
        if tree_idx != n_inner_layers - 1 {
            bit_counter += *step;
            base_point = repeated_double_point_simd(context, &base_point, *step);
        }
        layer_log_size -= *step;
    }
    // The last base point's y-coord may hasn't been used by the compute_twiddles if the last step
    // was = 1.
    if *steps.last().unwrap() == 1 {
        Simd::mark_partly_used(context, &base_point.y);
    }
    println!("Total line to line updates: {total_line_to_line_updates}");
    // Check last layer.
    assert_eq!(config.log_n_last_layer_coefs, 0);
    let last_layer_val = last_layer_coefs[0];
    for value in fri_data {
        eq(context, value, last_layer_val);
    }
}

fn compute_twiddles_from_base_point<Value: IValue>(
    context: &mut Context<Value>,
    base_point: &CirclePoint<Simd>,
    step: usize,
) -> Vec<Vec<Vec<Var>>> {
    let n_queries = base_point.x.len();
    let mut twiddles_per_fold_per_query: Vec<Vec<Vec<Var>>> = vec![vec![vec![]; step]; n_queries];
    let mut x_coords: Vec<Simd> = compute_half_coset_points(context, base_point, step as u32)
        .into_iter()
        .map(|p| p.x)
        .collect();
    for fold in 0..step {
        for x in &x_coords {
            let x_inv = x.inv(context);
            let unpacked = Simd::unpack(context, &x_inv);

            for (query_twiddles, twiddle) in
                twiddles_per_fold_per_query.iter_mut().zip(unpacked.into_iter())
            {
                query_twiddles[fold].push(twiddle);
            }
        }
        // Don't add unused gates in the last iteration.
        if fold != step - 1 {
            x_coords = x_coords.iter().step_by(2).map(|x| double_x_simd(context, x)).collect();
        }
    }

    twiddles_per_fold_per_query
}

fn translate_base_point<Value: IValue>(
    context: &mut Context<Value>,
    mut base_point: CirclePoint<Simd>,
    packed_bits: &[Simd],
) -> CirclePoint<Simd> {
    let n_queries = base_point.x.len();
    let zero = Simd::zero(context, n_queries);
    for (i, bit) in packed_bits.iter().enumerate() {
        let cur_gen_pt = generator_point_simd(context, i + 1, n_queries);
        let minus_y_coord = Simd::sub(context, &zero, &cur_gen_pt.y);
        let minus_cur_gen_pt = CirclePoint { x: cur_gen_pt.x, y: minus_y_coord };
        // Select between `point` and `point - cur_gen_pt`.
        let point_if_bit = add_points_simd(context, &base_point, &minus_cur_gen_pt);
        base_point = CirclePoint {
            x: Simd::select(context, bit, &base_point.x, &point_if_bit.x),
            y: Simd::select(context, bit, &base_point.y, &point_if_bit.y),
        };
    }
    base_point
}

fn translate_by_lsb<Value: IValue>(
    context: &mut Context<Value>,
    point: &CirclePoint<Simd>,
    bit: &Simd,
) -> CirclePoint<Simd> {
    let n_queries = point.x.len();
    let zero = Simd::zero(context, n_queries);
    let minus_y_coord = Simd::sub(context, &zero, &point.y);
    let minus_y_point = CirclePoint { x: point.x.clone(), y: minus_y_coord };
    // Select between `point` and `point - cur_gen_pt`.
    CirclePoint {
        x: Simd::select(context, bit, &point.x, &minus_y_point.x),
        y: Simd::select(context, bit, &point.y, &minus_y_point.y),
    }
}

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
    pack_leaves: bool,
) -> Vec<Var> {
    let points_y_inv = points.y.inv(context);
    let twiddles = Simd::unpack(context, &points_y_inv);
    let mut hash_leaves_updates = 0usize;
    let mut coset_root_updates = 0usize;
    let mut auth_updates = 0usize;
    if !pack_leaves {
        for (query_idx, (fri_query, sibling)) in zip_eq(fri_input, siblings).enumerate() {
            let before = context.stats.blake_updates;
            let leaf = hash_leaf_qm31(context, *fri_query);
            let leaf_sibling = hash_leaf_qm31(context, *sibling);
            hash_leaves_updates += context.stats.blake_updates - before;

            let bits_for_query = bits.iter().map(|b| b[query_idx]).collect_vec();
            let before = context.stats.blake_updates;
            let node = merkle_node(context, &leaf, &leaf_sibling, bits_for_query[0]);
            coset_root_updates += context.stats.blake_updates - before;

            let auth_path = auth_paths.at(0, query_idx);
            let bits_used_by_path = auth_path.0.len();
            let bits_tail = &bits_for_query[(bits_for_query.len() - bits_used_by_path)..];
            let before = context.stats.blake_updates;
            verify_merkle_path(context, node, bits_tail, *root, auth_path);
            auth_updates += context.stats.blake_updates - before;
        }
        println!("Circle-to-line | Hash leaves | # updates: {}", hash_leaves_updates);
        println!("Circle-to-line | Coset root | # updates: {}", coset_root_updates);
        println!("Circle-to-line | Authentication | # updates: {}", auth_updates);
        return zip_eq(zip_eq(fri_input, siblings), twiddles)
            .map(|((fri_query, sibling), twiddle)| {
                let g = eval!(context, (*fri_query) + (*sibling));
                let h = eval!(context, ((*fri_query) - (*sibling)) * (twiddle));
                eval!(context, (g) + ((alpha) * (h)))
            })
            .collect();
    }

    assert_eq!(siblings.len(), fri_input.len() * PACKED_FRI_LEAF_SIZE);
    let mut folded = Vec::with_capacity(fri_input.len());
    for (query_idx, (fri_query, twiddle)) in zip_eq(fri_input, twiddles).enumerate() {
        let leaf_values =
            &siblings[(query_idx * PACKED_FRI_LEAF_SIZE)..((query_idx + 1) * PACKED_FRI_LEAF_SIZE)];
        let bits_for_query = bits.iter().map(|b| b[query_idx]).collect_vec();
        let auth_path = auth_paths.at(0, query_idx);

        let before = context.stats.blake_updates;
        let leaf = hash_packed_fri_leaf(context, leaf_values);
        hash_leaves_updates += context.stats.blake_updates - before;
        let bits_used_by_path = auth_path.0.len();
        let bits_tail = &bits_for_query[(bits_for_query.len() - bits_used_by_path)..];
        let before = context.stats.blake_updates;
        verify_merkle_path(context, leaf, bits_tail, *root, auth_path);
        auth_updates += context.stats.blake_updates - before;

        let one_minus_lsb = eval!(context, (1) - (bits_for_query[0]));
        let sibling = select_by_index(context, leaf_values, &[one_minus_lsb, bits_for_query[1]]);
        let g = eval!(context, (*fri_query) + (sibling));
        let h = eval!(context, ((*fri_query) - (sibling)) * (twiddle));
        folded.push(eval!(context, (g) + ((alpha) * (h))));
    }
    println!("Circle-to-line | Hash leaves | # updates: {}", hash_leaves_updates);
    println!("Circle-to-line | Coset root | # updates: {}", coset_root_updates);
    println!("Circle-to-line | Authentication | # updates: {}", auth_updates);

    folded
}

fn compute_half_coset_points<Value: IValue>(
    context: &mut Context<Value>,
    base_point: &CirclePoint<Simd>,
    log_size: u32,
) -> Vec<CirclePoint<Simd>> {
    assert!(log_size > 0);
    let gen_pt = generator_point_simd(context, log_size as usize, base_point.x.len());
    let mut curr_pt = base_point.clone();
    let mut half_coset = vec![curr_pt.clone()];
    for _ in 0..(1 << (log_size - 1)) - 1 {
        curr_pt = add_points_simd(context, &curr_pt, &gen_pt);
        half_coset.push(curr_pt.clone());
    }
    // Bit reverse
    stwo::core::utils::bit_reverse(&mut half_coset);
    half_coset
}

fn fold_hashes_to_root<Value: IValue>(
    context: &mut Context<Value>,
    mut hashes: Vec<HashValue<Var>>,
    n_folds: usize,
) -> HashValue<Var> {
    for fold in 0..n_folds {
        let n_nodes = 1 << (n_folds - fold - 1);
        for i in 0..n_nodes {
            hashes[i] = hash_node(context, hashes[2 * i], hashes[2 * i + 1]);
        }
    }
    hashes[0]
}

fn hash_packed_fri_leaf<Value: IValue>(
    context: &mut Context<Value>,
    values: &[Var],
) -> HashValue<Var> {
    assert_eq!(values.len(), PACKED_FRI_LEAF_SIZE);
    let mut m31_values = Vec::with_capacity(PACKED_FRI_LEAF_SIZE * 4);
    for coord in 0..4 {
        for value in values {
            m31_values.push(extract_qm31_coord(context, *value, coord));
        }
    }
    let leaf_packed = Simd::pack(context, &m31_values);
    blake(context, leaf_packed.get_packed(), m31_values.len() * 4)
}

fn extract_qm31_coord<Value: IValue>(
    context: &mut Context<Value>,
    value: Var,
    coord: usize,
) -> M31Wrapper<Var> {
    let unit_vec = match coord {
        0 => qm31_from_u32s(1, 0, 0, 0),
        1 => qm31_from_u32s(0, 1, 0, 0),
        2 => qm31_from_u32s(0, 0, 1, 0),
        3 => qm31_from_u32s(0, 0, 0, 1),
        _ => unreachable!(),
    };
    let unit_vec_var = context.constant(unit_vec);
    let mut x = pointwise_mul(context, value, unit_vec_var);
    if coord > 0 {
        let inv = context.constant(unit_vec.inverse());
        x = eval!(context, (x) * (inv));
    }
    let masked = pointwise_mul(context, x, context.one());
    M31Wrapper::new_unsafe(masked)
}

/// Folds a coset of log size n to a point using the folding coefficients `alphas`.
/// `twiddles_per_fold[i]` contains the twiddles needed at fold i, and has length 2^(n - 1 - i).
pub fn fold_coset<Value: IValue>(
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
pub fn validate_query_position_in_coset<Value: IValue>(
    context: &mut Context<Value>,
    fri_coset_per_query: &[Vec<Var>],
    fri_data: &[Var],
    bits: &[Vec<Var>],
) {
    for (query_idx, (query_value, coset)) in zip_eq(fri_data, fri_coset_per_query).enumerate() {
        let bits: Vec<Var> = bits.iter().map(|b| b[query_idx]).collect();
        let should_be_query = select_by_index(context, coset, &bits);
        eq(context, *query_value, should_be_query);
    }
}

pub fn compute_steps(line_log_ratio: usize, line_fold_step: usize) -> Vec<usize> {
    // // The line-to-line folding steps of FRI are all equal to `line_fold_step`, except possibly
    // the last.
    let n_folds = line_log_ratio.div_ceil(line_fold_step);
    let rem = line_log_ratio % line_fold_step;
    let mut line_fold_steps = vec![line_fold_step; n_folds];
    line_fold_steps[n_folds - 1] = if rem == 0 { line_fold_step } else { rem };
    line_fold_steps
}
