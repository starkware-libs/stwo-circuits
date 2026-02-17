
use itertools::{Itertools, chain, zip, zip_eq};
use std::ops::Range;
use stwo::core::circle::CirclePoint;

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::{add, eq, mul, sub};
use crate::circuits::simd::Simd;
use crate::circuits::wrappers::M31Wrapper;
use crate::eval;
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::circle::{
    add_points_simd, double_x_simd, generator_point_simd, repeated_double_point_simd,
};
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
pub fn fri_decommit_with_jumps<Value: IValue>(
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
        fri_siblings, // TODO: This will be `fri_coset_per_query_per_tree`
    } = proof;

    let steps: Vec<usize> = vec![]; // TODO: part of config?

    let fri_coset_per_query_per_tree: Vec<Vec<Vec<Var>>> = vec![]; // TODO: see above 
    let mut fri_data = fri_input.iter().cloned().collect_vec();
    // Circle to line.
    fri_data = fold_circle_to_line(fri_data);
    // Line to line.
    // Prepare variables
    let n_bits = bits.len(); // TODO(Leo): check off by one errors.
    let n_queries = points.x.len();
    let mut base_point = points.clone();
    let zero = Simd::zero(context, n_queries);
    let mut bit_counter = 0;

    for (tree_idx, (root, step)) in
        zip_eq(layer_commitments, steps).enumerate().skip(1)
    {
        let fri_coset_per_query = &fri_coset_per_query_per_tree[tree_idx];
        let bit_range = (1 + bit_counter)..(1 + bit_counter + step);

        // Validate that the fri query is in the correct poisition inside the guessed `fri_coset_per_query`.
        validate_query_position_in_coset(
            context,
            fri_coset_per_query,
            &fri_data,
            &bits[bit_range.clone()],
        );

        // Check merkle decommitment.
        for (query_idx, coset_values) in fri_coset_per_query.iter().enumerate() {
            todo!()
        }

        // Update base point.
        base_point = update_base_point(context, base_point, packed_bits, bit_range);

        // Compute twiddles.
        let twiddles_per_fold_per_query = compute_twiddles_from_base_point(context, &base_point, step);

        // Compute alpha, alpha^2, ..., alpha^(2^(step - 1));
        let alphas: Vec<Var> = (0..step)
            .scan(alphas[tree_idx], |state, _| {
                let out = *state;
                *state = mul(context, *state, *state);
                Some(out)
            })
            .collect();

        // Compute the next layer.
        fri_data = zip_eq(fri_coset_per_query, twiddles_per_fold_per_query)
        .map(|(coset_values, twiddles_per_fold)| {
            fold_coset(context, &coset_values, &twiddles_per_fold, &alphas)
        })
        .collect();

        bit_counter += step as usize;
    }

    // Check last layer.
    assert_eq!(config.log_n_last_layer_coefs, 0);
    let last_layer_val = last_layer_coefs[0];
    for value in fri_data {
        eq(context, value, last_layer_val);
    }
}

fn compute_twiddles_from_base_point<Value: IValue>(context: &mut Context<Value>, base_point: &CirclePoint<Simd>, step: usize) -> Vec<Vec<Vec<Var>>> {
    let mut buf: Vec<Vec<Vec<Var>>> = vec![];
    let n_queries = base_point.x.len();
    let mut prev_x_coord: Vec<_> = compute_coset_points(context, base_point, step as u32 - 1)
        .iter() 
        .map(|p| p.x.clone())
        .collect();
    let mut prev_twiddles: Vec<_> = prev_x_coord.iter().map(|x| x.inv(context)).collect();
    buf.push(prev_twiddles.iter().map(|t_simd| Simd::unpack(context, t_simd)).collect());
    // Compute the rest of the folds
    if step >= 2 {for _ in 0..step - 2 {
        prev_x_coord = prev_x_coord.iter().map(|x| double_x_simd(context, x)).collect();
        prev_twiddles = prev_x_coord.iter().map(|x| x.inv(context)).collect();
        buf.push(prev_twiddles.iter().map(|t_simd| Simd::unpack(context, t_simd)).collect());
    }}
    let mut twiddles_per_fold_per_query = vec![];
    // Transpose
    for twiddles in buf.iter() {
        let mut res: Vec<Vec<Var>> = vec![];
        for i in 0..n_queries {
            res.push(twiddles.iter().map(|p| p[i]).collect());
        }
        twiddles_per_fold_per_query.push(res);
    }
    twiddles_per_fold_per_query
}

fn update_base_point<Value: IValue>(
    context: &mut Context<Value>,
    mut base_point: CirclePoint<Simd>,
    packed_bits: &[Simd],
    bit_range: Range<usize>,
) -> CirclePoint<Simd> {
    let n_queries = base_point.x.len();
    let zero = Simd::zero(context, n_queries);
    for i in bit_range {
        let cur_gen_pt = generator_point_simd(context, i, n_queries);
        let minus_y_coord = Simd::sub(context, &zero, &cur_gen_pt.y);
        let minus_cur_gen_pt = CirclePoint { x: cur_gen_pt.x, y: minus_y_coord };
        // Select between `point` and `point - cur_gen_pt`.
        let point_if_bit = add_points_simd(context, &base_point, &minus_cur_gen_pt);
        base_point = CirclePoint {
            x: Simd::select(context, &packed_bits[i], &base_point.x, &point_if_bit.x),
            y: Simd::select(context, &packed_bits[i], &base_point.y, &point_if_bit.y),
        };
    }
    base_point
}

fn validate_query_position_in_coset<Value: IValue>(
    context: &mut Context<Value>,
    fri_coset_per_query: &[Vec<Var>],
    fri_data: &[Var],
    bits: &[Vec<Var>],
) {
    for (query_idx, (query_value, coset)) in zip_eq(fri_data, fri_coset_per_query).enumerate() {
        let bits: Vec<Var> = bits.iter().map(|b| b[query_idx]).collect();
        let should_be_query = select_by_index(context, coset, bits);
        eq(context, *query_value, should_be_query);
    }
}

fn fold_coset<Value: IValue>(
    context: &mut Context<Value>,
    coset_values: &[Var],
    twiddles_per_fold: &[Vec<Var>],
    alphas: &[Var],
) -> Var {
    // TODO: add asserts on lengths.
    let mut values = coset_values.to_vec();
    for (i, twiddles) in twiddles_per_fold.iter().enumerate() {
        let mut buf = vec![];
        for (v, t) in zip_eq(values.chunks_exact(2), twiddles) {
            let [v1, v2] = v.try_into().unwrap();
            let g = eval!(context, (v1) + (v2));
            let h = eval!(context, ((v1) - (v2)) * (*t));
            buf.push(eval!(context, (g) + ((alphas[i]) * (h))));
        }
        values = buf;
    }
    values[0]
}

fn fold_circle_to_line(data: Vec<Var>) -> Vec<Var> {
    todo!()
}

fn compute_coset_points<Value: IValue>(
    context: &mut Context<Value>,
    base_point: &CirclePoint<Simd>,
    log_size: u32,
) -> Vec<CirclePoint<Simd>> {
    // Early return for trivial log_size.
    if log_size == 0 {
        return vec![base_point.clone()];
    }
    let gen_pt = generator_point_simd(context, log_size as usize, base_point.x.len());
    let mut curr_pt = base_point.clone();
    let mut coset = vec![curr_pt.clone()];
    for _ in 0..(1 << log_size) - 1 as usize {
        curr_pt = add_points_simd(context, &curr_pt, &gen_pt);
        coset.push(curr_pt.clone());
    }
    // Bit reverse
    stwo::core::utils::bit_reverse(&mut coset);
    coset
}

fn select_by_index<Value: IValue>(
    context: &mut Context<Value>,
    values: &[Var],
    index_bits: Vec<Var>,
) -> Var {
    assert!(values.len().is_power_of_two());
    assert_eq!(values.len().ilog2() as usize, index_bits.len());

    // TODO(Leo): use simd?
    let mut one_hot = vec![context.one()];
    for bit in index_bits.into_iter() {
        let one_minus_bit = sub(context, context.one(), bit);
        let mut res = Vec::with_capacity(2 * one_hot.len());
        res.extend(one_hot.iter().map(|x| mul(context, *x, one_minus_bit)));
        res.extend(one_hot.iter().map(|x| mul(context, *x, bit)));
        one_hot = res;
    }

    zip_eq(one_hot, values).fold(context.zero(), |acc, (bit, v)| {
        let m = mul(context, bit, *v);
        add(context, acc, m)
    })
}
