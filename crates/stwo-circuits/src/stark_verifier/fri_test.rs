use crate::circuits::blake::HashValue;
use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::ops::{Guess, mul};
use crate::circuits::simd::Simd;
use crate::circuits::test_utils::{packed_values, simd_from_u32s};
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::circle::repeated_double_point_simd;
use crate::stark_verifier::fri::fri_decommit;
use crate::stark_verifier::fri_proof::{FriCommitProof, FriProof};
use crate::stark_verifier::merkle::{AuthPath, AuthPaths};
use crate::stark_verifier::proof::ProofConfig;
use crate::stark_verifier::proof_from_stark_proof::construct_fri_auth_paths;
use crate::stark_verifier::select_queries::select_queries;
use itertools::{chain, zip_eq};
use num_traits::One;
use rstest::rstest;
use stwo::core::channel::{Blake2sChannel, Blake2sM31Channel};
use stwo::core::circle::Coset;
use stwo::core::fields::FieldExpOps;
use stwo::core::fields::m31::BaseField;
use stwo::core::fields::qm31::{QM31, SecureField};
use stwo::core::fri::{
    ExtendedFriProof, FriConfig, fold_coset as stwo_fold_coset, fold_line as stwo_fold_line,
};
use stwo::core::poly::circle::{CanonicCoset, CircleDomain};
use stwo::core::poly::line::LineDomain;
use stwo::core::queries::Queries;
use stwo::core::utils::bit_reverse_index;
use stwo::core::vcs::blake2_hash::Blake2sM31Hasher;
use stwo::core::vcs_lifted::blake2_merkle::{
    Blake2sM31MerkleChannel, Blake2sM31MerkleHasher, Blake2sMerkleChannel,
};
use stwo::prover::backend::CpuBackend;
use stwo::prover::backend::cpu::CpuCirclePoly;
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::{PolyOps, SecureEvaluation};

use super::{
    compute_twiddles_from_base_point, fold_coset, fri_commit, translate_base_point,
    validate_query_position_in_coset,
};

#[test]
fn test_fri_commit_regression() {
    let mut context = TraceContext::default();

    let init_digest = [
        qm31_from_u32s(375224163, 1270824854, 44060607, 991529112),
        qm31_from_u32s(1068130924, 1630210318, 1632828025, 1983481471),
    ];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let proof = FriCommitProof {
        layer_commitments: vec![
            HashValue(
                qm31_from_u32s(370372302, 356302922, 2040089875, 232934191),
                qm31_from_u32s(1279830905, 1240360672, 1788604172, 465814885),
            ),
            HashValue(
                qm31_from_u32s(1558212721, 609186473, 1554074721, 1956195301),
                qm31_from_u32s(1243917617, 135256448, 1193318416, 1792104990),
            ),
            HashValue(
                qm31_from_u32s(1017503040, 1411053946, 1805475392, 1906875756),
                qm31_from_u32s(2035075097, 617472393, 571220918, 1577790110),
            ),
            HashValue(
                qm31_from_u32s(1290083578, 670256590, 203247471, 492011214),
                qm31_from_u32s(353269841, 1619070080, 770215254, 1663098736),
            ),
        ],
        last_layer_coefs: vec![qm31_from_u32s(1802004671, 1018373769, 131996621, 1575090881)],
    };
    let proof_vars = proof.guess(&mut context);

    let alphas = fri_commit(&mut context, &mut channel, &proof_vars);
    assert_eq!(alphas.len(), 4);
    assert_eq!(context.get(alphas[0]), qm31_from_u32s(2047550788, 23895068, 1676134944, 263598239));
    assert_eq!(
        context.get(alphas[1]),
        qm31_from_u32s(1988032363, 1739489633, 826507892, 1797301629)
    );
    assert_eq!(
        context.get(alphas[2]),
        qm31_from_u32s(1957504342, 848565442, 1129943791, 1937962621)
    );
    assert_eq!(context.get(alphas[3]), qm31_from_u32s(1748651123, 2133979933, 232524784, 85583628));

    assert_eq!(
        context.get(channel.digest().0),
        qm31_from_u32s(968886948, 725376924, 836084817, 484428276)
    );
    assert_eq!(
        context.get(channel.digest().1),
        qm31_from_u32s(1805658819, 300032261, 172116750, 994058243)
    );

    context.validate_circuit();
}

#[rstest]
#[case::success(true)]
#[case::failure(false)]
fn test_validate_query_position_in_coset(#[case] success: bool) {
    let mut context = TraceContext::default();

    let mut const_var = |value: u32| context.constant(qm31_from_u32s(value, 0, 0, 0));

    // 3 queries, each with a coset of size 4 (2 index bits).
    let fri_coset_per_query = vec![
        vec![const_var(10), const_var(11), const_var(12), const_var(13)],
        vec![const_var(20), const_var(21), const_var(22), const_var(23)],
        vec![const_var(30), const_var(31), const_var(32), const_var(33)],
    ];

    // `select_by_index` interprets bits as little-endian: idx = b0 + 2*b1.
    // Query indices are: 0, 1, 2.
    let bits = vec![
        vec![const_var(0), const_var(1), const_var(0)], // b0
        vec![const_var(0), const_var(0), const_var(1)], // b1
    ];

    let mut fri_data = vec![const_var(10), const_var(21), const_var(32)];
    if !success {
        // Make the 3rd query inconsistent with bits=(0,1), which should point to value 32.
        fri_data[2] = const_var(33);
    }

    validate_query_position_in_coset(&mut context, &fri_coset_per_query, &fri_data, &bits);
    assert_eq!(context.is_circuit_valid(), success);
}

fn simd_lanes(context: &TraceContext, simd: &Simd) -> Vec<u32> {
    let packed = packed_values(context, simd);
    (0..simd.len()).map(|i| packed[i / 4].to_m31_array()[i % 4].0).collect()
}

#[test]
fn test_fold_coset_matches_stwo_reference() {
    let mut context = TraceContext::default();
    let step = 3usize;

    let coset_values_qm31: Vec<_> = (0..(1 << step))
        .map(|i| qm31_from_u32s(i as u32 + 1, 2 * i as u32 + 3, 3 * i as u32 + 5, 5 * i as u32 + 7))
        .collect();
    let coset_values: Vec<_> = coset_values_qm31.iter().map(|x| context.constant(*x)).collect();

    let alpha_qm31 = qm31_from_u32s(17, 3, 5, 7);
    let mut alpha_pow = alpha_qm31;
    let mut alphas = Vec::with_capacity(step);
    for _ in 0..step {
        alphas.push(context.constant(alpha_pow));
        alpha_pow = alpha_pow * alpha_pow;
    }

    let mut fold_domain = LineDomain::new(Coset::half_odds(step as u32));
    let mut twiddles_per_fold = Vec::with_capacity(step);
    for i in 0..step {
        let twiddles_len = 1 << (step - i - 1);
        let twiddles = (0..twiddles_len)
            .map(|k| {
                let j = 2 * k;
                let x = fold_domain.at(bit_reverse_index(j, fold_domain.log_size()));
                context.constant(SecureField::from(x.inverse()))
            })
            .collect::<Vec<_>>();
        twiddles_per_fold.push(twiddles);
        fold_domain = fold_domain.double();
    }

    let actual = fold_coset(&mut context, &coset_values, &twiddles_per_fold, &alphas);
    let expected = stwo_fold_coset(
        &coset_values_qm31,
        LineDomain::new(Coset::half_odds(step as u32)),
        alpha_qm31,
    );

    assert_eq!(context.get(actual), expected);
    context.validate_circuit();
}

#[test]
fn test_jump_folding_matches_stwo_reference() {
    let fri_proof = stwo_jumps();

    let mut context = TraceContext::default();
    const LOG_DOMAIN_SIZE: usize = 8;
    // Keep the same query structure, but use a repeated index so `fri_decommit` can check a
    // constant last layer even with only the circle->line phase enabled.
    let query_indices = vec![11_u32, 12_u32, 13_u32, 14_u32];

    let config = ProofConfig {
        n_proof_of_work_bits: 0,
        n_preprocessed_columns: 0,
        n_trace_columns: 0,
        n_interaction_columns: 0,
        trace_columns_per_component: vec![],
        interaction_columns_per_component: vec![],
        cumulative_sum_columns: vec![],
        n_components: 0,
        fri: crate::stark_verifier::fri_proof::FriConfig {
            log_trace_size: 4,
            log_blowup_factor: 2,
            n_queries: query_indices.len(),
            log_n_last_layer_coefs: 0,
            steps: vec![2, 2, 1],
        },
        interaction_pow_bits: 0,
    };

    let input = simd_from_u32s(&mut context, query_indices.clone());
    let queries = select_queries(&mut context, &input, LOG_DOMAIN_SIZE);
    let alpha_values = [
        qm31_from_u32s(1011730217, 238354028, 1321702146, 1634795701),
        qm31_from_u32s(1690232064, 1294671291, 1616406021, 525755234),
        qm31_from_u32s(1975580628, 2062626494, 1340534631, 1939928290),
        qm31_from_u32s(160270922, 428202964, 1497289811, 1557635193),
    ];
//     Circle to line alpha: (1011730217 + 238354028) + (1321702146 + 1634795701
// Folding line alpha: (1690232064 + 1294671291) + (1616406021 + 525755234)u
// Folding line alpha: (1975580628 + 2062626494) + (1340534631 + 1939928290)u
// Folding line alpha: (160270922 + 428202964) + (1497289811 + 1557635193)u
    let alphas: Vec<_> = alpha_values.iter().map(|x| context.constant(*x)).collect();

    let query_locations = query_indices.iter().map(|x| *x as usize).collect::<Vec<_>>();
    let siblings = test_construct_fri_siblings(&fri_proof, &config, &query_locations);
    let auth_paths =
        test_construct_first_fri_auth_paths(&fri_proof, LOG_DOMAIN_SIZE, &query_locations);
    let fri_input: Vec<_> = query_locations
        .iter()
        .map(|query| context.constant(fri_proof.aux.first_layer.all_values[0][query]))
        .collect();
    let bits: Vec<Vec<_>> =
        queries.bits.iter().map(|simd| Simd::unpack(&mut context, simd)).collect();
    let packed_bits = queries.bits.clone();
    let points = queries.points.clone();

    // Expected output after circle->line fold for the repeated query.
    // let query = query_indices[0] as usize;
    // let query_val = fri_proof.aux.first_layer.all_values[0][&query];
    // let query_sibling = fri_proof.aux.first_layer.all_values[0][&(query ^ 1)];
    // let domain = CanonicCoset::new(LOG_DOMAIN_SIZE as u32).circle_domain();
    // let query_point = domain.at(bit_reverse_index(query, LOG_DOMAIN_SIZE as u32));
    // let twiddle = SecureField::from(query_point.y.inverse());


    let circuit_fri_proof = FriProof {
        commit: FriCommitProof {
            layer_commitments: chain!(
                [fri_proof.proof.first_layer.commitment.into()],
                fri_proof.proof.inner_layers.iter().map(|layer| layer.commitment.into()),
            )
            .collect(),
            last_layer_coefs: fri_proof.proof.last_layer_poly.coeffs
        },
        auth_paths,
        circle_fri_siblings: siblings.0,
        line_coset_vals_per_query_per_tree: siblings.1,
    };
    let circuit_fri_proof = circuit_fri_proof.guess(&mut context);

    fri_decommit(
        &mut context,
        &circuit_fri_proof,
        &config.fri,
        &fri_input,
        &bits,
        &packed_bits,
        &points,
        &alphas,
    );

    context.validate_circuit();
}
/// Returns an evaluation of a random polynomial with degree `2^log_degree`.
///
/// The evaluation domain size is `2^(log_degree + log_blowup_factor)`.
fn polynomial_evaluation(
    log_degree: u32,
    log_blowup_factor: u32,
) -> SecureEvaluation<CpuBackend, BitReversedOrder> {
    let poly = CpuCirclePoly::new(vec![BaseField::one(); 1 << log_degree]);
    let coset = Coset::half_odds(log_degree + log_blowup_factor - 1);
    let domain = CircleDomain::new(coset);
    let values = poly.evaluate(domain);
    SecureEvaluation::new(domain, values.into_iter().map(SecureField::from).collect())
}

type FriProver<'a> = stwo::prover::fri::FriProver<'a, CpuBackend, Blake2sM31MerkleChannel>;

fn stwo_jumps() -> ExtendedFriProof<Blake2sM31Hasher> {
    const LOG_BLOWUP_FACTOR: u32 = 2;

    let config = FriConfig::new(0, LOG_BLOWUP_FACTOR, 3, 2);
    let column = polynomial_evaluation(6, LOG_BLOWUP_FACTOR);
    let twiddles = CpuBackend::precompute_twiddles(column.domain.half_coset);

    let prover = FriProver::commit(&mut Blake2sM31Channel::default(), config, &column, &twiddles);
    let queries = Queries::new(&vec![11, 12, 13, 14], 6 + LOG_BLOWUP_FACTOR);
    let point = CanonicCoset::new(8).at(bit_reverse_index(11, 8));
    println!("{:?}", point);
    let proof = prover.decommit_on_queries(&queries);
    proof
}

pub fn test_construct_fri_siblings(
    proof: &ExtendedFriProof<Blake2sM31MerkleHasher>,
    config: &ProofConfig,
    query_locations: &[usize],
) -> (Vec<QM31>, Vec<Vec<Vec<QM31>>>) {
    let mut line_coset_vals_per_query_per_tree = vec![vec![]; config.log_trace_size() - 1];
    let mut circle_fri_siblings = vec![];
    for query in query_locations {
        circle_fri_siblings.push(proof.aux.first_layer.all_values[0][&(query ^ 1)]);
        let mut pos = query >> 1;
        for (tree_idx, (layer, step)) in
            zip_eq(&proof.aux.inner_layers, &config.fri.steps).enumerate()
        {
            let start = (pos >> step) << step;
            let line_coset_vals: Vec<_> =
                (start..start + (1 << step)).map(|i| layer.all_values[0][&i]).collect();
            line_coset_vals_per_query_per_tree[tree_idx].push(line_coset_vals);
            pos >>= step;
        }
    }
    (circle_fri_siblings, line_coset_vals_per_query_per_tree)
}

pub fn test_construct_first_fri_auth_paths(
    proof: &ExtendedFriProof<Blake2sM31MerkleHasher>,
    log_domain_size: usize,
    query_locations: &[usize],
) -> AuthPaths<QM31> {
    let first_layer_paths = query_locations
        .iter()
        .map(|query| {
            let mut auth_path = AuthPath(vec![]);
            let mut pos = *query;
            for j in 0..log_domain_size {
                let hash = proof.aux.first_layer.decommitment.all_node_values[j][&(pos ^ 1)];
                if j > 0 {
                    auth_path.0.push(hash.into());
                }
                pos >>= 1;
            }
            auth_path
        })
        .collect();
    AuthPaths { data: vec![first_layer_paths] }
}
