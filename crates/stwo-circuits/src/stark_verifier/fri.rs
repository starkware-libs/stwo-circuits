use itertools::{Itertools, chain, zip, zip_eq};
use std::ops::Range;
use stwo::core::circle::CirclePoint;

use crate::circuits::blake::HashValue;
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
use crate::stark_verifier::merkle::{AuthPaths, hash_leaf_qm31, merkle_node, verify_merkle_path};

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

// /// Validates that the values in `fri_input` are consistent with the FRI commitment.
// pub fn fri_decommit<Value: IValue>(
//     context: &mut Context<Value>,
//     proof: &FriProof<Var>,
//     config: &FriConfig,
//     fri_input: &[Var],
//     bits: &[Vec<Var>],
//     points: &CirclePoint<Simd>,
//     alphas: &[Var],
// ) {
//     let FriProof {
//         commit: FriCommitProof { layer_commitments, last_layer_coefs },
//         auth_paths,
//         fri_siblings,
//     } = proof;

//     // Prepare twiddle factors.
//     let mut all_twiddles = vec![];
//     let points_y_inv = points.y.inv(context);
//     all_twiddles.push(Simd::unpack(context, &points_y_inv));

//     let mut points_x = points.x.clone();
//     let points_x_inv = points_x.inv(context);
//     all_twiddles.push(Simd::unpack(context, &points_x_inv));

//     for _ in 0..(layer_commitments.len() - 2) {
//         points_x = double_x_simd(context, &points_x);
//         eprintln!("Length: {}", points_x.len());
//         let points_x_inv = points_x.inv(context);
//         all_twiddles.push(Simd::unpack(context, &points_x_inv));
//     }

//     let mut fri_data = fri_input.iter().cloned().collect_vec();
//     for (tree_idx, (root, twiddles)) in zip_eq(layer_commitments, all_twiddles).enumerate() {
//         let siblings = &fri_siblings[tree_idx];

//         // Check merkle decommitment.
//         for (query_idx, (fri_query, sibling)) in zip_eq(&fri_data, siblings).enumerate() {
//             // Compute one layer of the Merkle tree with the query and its sibling.
//             let leaf = hash_leaf_qm31(context, *fri_query);
//             let leaf_sibling = hash_leaf_qm31(context, *sibling);

//             // Skip the first `tree_idx` LSBs, that are not relevant for this tree.
//             let bits_for_query = bits.iter().skip(tree_idx).map(|b| b[query_idx]).collect_vec();
//             let node = merkle_node(context, &leaf, &leaf_sibling, bits_for_query[0]);

//             let auth_path = auth_paths.at(tree_idx, query_idx);
//             verify_merkle_path(context, node, &bits_for_query[1..], *root, auth_path);
//         }

//         // Compute the next layer.
//         fri_data = zip_eq(zip_eq(fri_data, siblings), twiddles)
//             .map(|((fri_query, sibling), twiddle)| {
//                 let g = eval!(context, (fri_query) + (*sibling));
//                 let h = eval!(context, ((fri_query) - (*sibling)) * (twiddle));
//                 eval!(context, (g) + ((alphas[tree_idx]) * (h)))
//             })
//             .collect();
//     }

//     // Check last layer.
//     assert_eq!(config.log_n_last_layer_coefs, 0);
//     let last_layer_val = last_layer_coefs[0];
//     for value in fri_data {
//         eq(context, value, last_layer_val);
//     }
// }

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
        line_coset_vals_per_query_per_tree,
        circle_fri_siblings,
    } = proof;
    // for (tree, line_coset_vals_per_query) in line_coset_vals_per_query_per_tree.iter().enumerate() {
    //     eprintln!("Tree: {}", tree + 1);
    //     for (query_idx, line_coset_vals) in line_coset_vals_per_query.iter().enumerate() {
    //         eprintln!("query: {query_idx}, coset val len: {}", line_coset_vals.len());
    //     }
    // }
    // TODO: Remove copy
    let steps = config.steps.to_vec();
    // Circle to line decommitment.
    let mut fri_data = decommit_circle_to_line(
        context,
        &layer_commitments[0],
        &circle_fri_siblings,
        auth_paths,
        fri_input,
        bits,
        points,
        alphas[0],
    );
    // Line to line decommitment.
    let mut base_point = translate_base_point_2(context, points, &packed_bits[0]);
    let mut bit_counter = 0;

    for (tree_idx, ((root, step), coset_per_query)) in zip_eq(zip_eq(&layer_commitments[1..], steps), line_coset_vals_per_query_per_tree).enumerate() {
        let bit_range = (1 + bit_counter)..(1 + bit_counter + step);
        eprintln!("Tree: {}", tree_idx + 1);
        eprintln!("Bit range: {:?}", bit_range);
        eprintln!("Base point: x {:?} y {:?}", context.get(base_point.x.get_packed()[0]), context.get(base_point.y.get_packed()[0]));
        // Validate that the fri query is in the correct position inside the guessed
        // `fri_coset_per_query`.
        validate_query_position_in_coset(
            context,
            coset_per_query,
            &fri_data,
            &bits[bit_range.clone()],
        );

        // Check merkle decommitment.
        for (query_idx, coset_values) in coset_per_query.iter().enumerate() {
            println!("Inside merkle decommitment");
        }

        // Translate base_point to the base of the current coset.
        base_point = translate_base_point(context, base_point, &packed_bits[bit_range]);
        eprintln!("Translated base point: x {:?}, y: {:?}", context.get(base_point.x.get_packed()[0]), context.get(base_point.y.get_packed()[0]));

        // Compute twiddles.
        let twiddles_per_fold_per_query =
            compute_twiddles_from_base_point(context, &base_point, step);

        // Compute alpha, alpha^2, ..., alpha^(2^(step - 1));
        let alphas: Vec<Var> = (0..step)
            .scan(alphas[tree_idx + 1], |state, _| {
                let out = *state;
                *state = mul(context, *state, *state);
                Some(out)
            })
            .collect();

        // Compute the next layer.
        fri_data = zip_eq(coset_per_query, twiddles_per_fold_per_query)
            .map(|(coset, twiddles_per_fold)| {
                fold_coset(context, &coset, &twiddles_per_fold, &alphas)
            })
            .collect();

        bit_counter += step as usize;
        base_point = repeated_double_point_simd(context, &base_point, step);
    }

    // Check last layer.
    assert_eq!(config.log_n_last_layer_coefs, 0);
    let last_layer_val = last_layer_coefs[0];
    for value in fri_data {
        eq(context, value, last_layer_val);
    }
}

/// A vector v = [v_1, ..., v_step], where v_i is a vector of length 2^(step - i), corresponding to
/// the twiddles of the internal fold.
type JumpTwiddles = Vec<Vec<Var>>;

fn compute_twiddles_from_base_point<Value: IValue>(
    context: &mut Context<Value>,
    base_point: &CirclePoint<Simd>,
    step: usize,
) -> Vec<JumpTwiddles> {
    let mut buf: Vec<Vec<Vec<Var>>> = vec![];
    let n_queries = base_point.x.len();
    let mut prev_x_coord: Vec<Simd> = compute_half_coset_points(context, base_point, step as u32)
        .iter()
        .map(|p| p.x.clone())
        .collect();
    let mut prev_twiddles: Vec<_> = prev_x_coord.iter().map(|x| x.inv(context)).collect();
    // Print the outermost twiddles.
    prev_twiddles.iter().for_each(|t| eprintln!("Outermost twiddle {:?}", context.get(t.get_packed()[0]))     );
    buf.push(prev_twiddles.iter().map(|t_simd| Simd::unpack(context, t_simd)).collect());

    for _ in 0..step - 1 {
        prev_x_coord = prev_x_coord.iter().step_by(2).map(|x| double_x_simd(context, x)).collect();
        prev_twiddles = prev_x_coord.iter().map(|x| x.inv(context)).collect();
        buf.push(prev_twiddles.iter().map(|t_simd| Simd::unpack(context, t_simd)).collect());
    }
    let mut twiddles_per_fold_per_query: Vec<JumpTwiddles> = vec![];

    // Transpose
    for i in 0..n_queries {
        let mut res: JumpTwiddles = vec![];
        for twiddles in buf.iter() {
            let tmp = twiddles.iter().map(|p| p[i]).collect();
            res.push(tmp);
        }
        twiddles_per_fold_per_query.push(res);
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
            x: Simd::select(context, &bit, &base_point.x, &point_if_bit.x),
            y: Simd::select(context, &bit, &base_point.y, &point_if_bit.y),
        };
    }
    base_point
}

fn translate_base_point_2<Value: IValue>(
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
            x: Simd::select(context, &bit, &point.x, &minus_y_point.x),
            y: Simd::select(context, &bit, &point.y, &minus_y_point.y),
        }

}


// This is per query.
fn fold_coset<Value: IValue>(
    context: &mut Context<Value>,
    coset_values: &[Var],
    twiddles_per_fold: &[Vec<Var>],
    alphas: &[Var],
) -> Var {
    assert_eq!(twiddles_per_fold.len(), alphas.len());
    assert_eq!(coset_values.len(), 1 << twiddles_per_fold.len());
    // for (i, twiddles) in twiddles_per_fold.iter().enumerate() {
    //     assert_eq!(twiddles.len(), 1 << (twiddles_per_fold.len() - i - 1));
    // }
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
    // for (query_idx, (fri_query, sibling)) in zip_eq(fri_input, siblings).enumerate() {
    //     // Compute one layer of the Merkle tree with the query and its sibling.
    //     let leaf = hash_leaf_qm31(context, *fri_query);
    //     let leaf_sibling = hash_leaf_qm31(context, *sibling);

    //     // Skip the first `tree_idx` LSBs, that are not relevant for this tree.
    //     let bits_for_query = bits.iter().map(|b| b[query_idx]).collect_vec();
    //     let node = merkle_node(context, &leaf, &leaf_sibling, bits_for_query[0]);

    //     let auth_path = auth_paths.at(0, query_idx);
    //     verify_merkle_path(context, node, &bits_for_query[1..], *root, auth_path);
    // }

    // Compute the next layer.
    zip_eq(zip_eq(fri_input, siblings), twiddles)
        .map(|((fri_query, sibling), twiddle)| {
            let g = eval!(context, (*fri_query) + (*sibling));
            let h = eval!(context, ((*fri_query) - (*sibling)) * (twiddle));
            eval!(context, (g) + ((alpha) * (h)))
        })
        .collect()
}

fn compute_half_coset_points<Value: IValue>(
    context: &mut Context<Value>,
    base_point: &CirclePoint<Simd>,
    log_size: u32,
) -> Vec<CirclePoint<Simd>> {
    if log_size == 0 {
        return vec![base_point.clone()];
    }
    let gen_pt = generator_point_simd(context, log_size as usize, base_point.x.len());
    let mut curr_pt = base_point.clone();
    let mut half_coset = vec![curr_pt.clone()];
    for _ in 0..(1 << (log_size - 1)) - 1 as usize {
        curr_pt = add_points_simd(context, &curr_pt, &gen_pt);
        half_coset.push(curr_pt.clone());
    }
    // Bit reverse
    stwo::core::utils::bit_reverse(&mut half_coset);
    half_coset
}

fn validate_query_position_in_coset<Value: IValue>(
    context: &mut Context<Value>,
    fri_coset_per_query: &[Vec<Var>],
    fri_data: &[Var],
    bits: &[Vec<Var>],
) {
    for (query_idx, (query_value, coset)) in zip_eq(fri_data, fri_coset_per_query).enumerate() {
        println!("Coset");
        coset.iter().for_each(|x| println!("{:?}", context.get(*x)));
        eprintln!("Query: {:?}", context.get(*query_value));
        let bits: Vec<Var> = bits.iter().map(|b| b[query_idx]).collect();
        let should_be_query = select_by_index(context, coset, bits);
        debug_assert_eq!(context.get(should_be_query), context.get(*query_value));
        eq(context, *query_value, should_be_query);
    }
}
/// Implements a multiplexer. 
/// Given a vector `values` and an index (represented in its bit decomposition `index_bits`)
/// returns a new variable equal to `values[index]`.
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
        let mut res = vec![];
        let one_minus_bit = sub(context, context.one(), bit);
        res.extend(one_hot.iter().map(|x| mul(context, *x, one_minus_bit)));
        res.extend(one_hot.iter().map(|x| mul(context, *x, bit)));
        one_hot = res;
    }

    zip_eq(one_hot, values).fold(context.zero(), |acc, (bit, v)| {
        let m = mul(context, bit, *v);
        add(context, acc, m)
    })
}
