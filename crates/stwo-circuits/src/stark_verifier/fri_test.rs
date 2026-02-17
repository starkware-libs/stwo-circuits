use crate::circuits::blake::HashValue;
use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::ops::{Guess, mul};
use crate::circuits::simd::Simd;
use crate::circuits::test_utils::{packed_values, simd_from_u32s};
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::circle::repeated_double_point_simd;
use crate::stark_verifier::fri::fri_decommit;
use crate::stark_verifier::fri_proof::FriCommitProof;
use crate::stark_verifier::select_queries::select_queries;
use itertools::zip_eq;
use num_traits::One;
use rstest::rstest;
use stwo::core::circle::Coset;
use stwo::core::fields::m31::BaseField;
use stwo::core::fields::qm31::{QM31, SecureField};
use stwo::core::fri::{FriConfig, fold_coset as stwo_fold_coset, fold_line as stwo_fold_line};
use stwo::core::poly::circle::CircleDomain;
use stwo::core::poly::line::LineDomain;
use stwo::core::utils::bit_reverse_index;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleChannel;
use stwo::prover::backend::CpuBackend;
use stwo::prover::backend::cpu::CpuCirclePoly;
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::SecureEvaluation;

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


// #[test]
// fn test_jump_folding_matches_stwo_reference() {
//     let mut context = TraceContext::default();
//     const LOG_DOMAIN_SIZE: usize = 8;
//     let steps = [2_usize, 1_usize, 2_usize];

//     // Query indices in the circle evaluation domain.
//     let query_indices = vec![0b0101_1011_u32, 0b1010_0110_u32, 0b0011_1100_u32, 0b1110_0001_u32];
//     let input = simd_from_u32s(&mut context, query_indices.clone());
//     let queries = select_queries(&mut context, &input, LOG_DOMAIN_SIZE);

//     // Start from the first line layer (after circle->line fold).
//     let mut query_positions: Vec<usize> =
//         query_indices.iter().map(|q| (*q as usize) >> 1).collect();
//     let mut line_domain = LineDomain::new(Coset::half_odds((LOG_DOMAIN_SIZE - 1) as u32));

//     // Deterministic layer values.
//     let mut layer_values_qm31: Vec<QM31> = (0..(1 << (LOG_DOMAIN_SIZE - 1)))
//         .map(|i| qm31_from_u32s(i as u32 + 1, 2 * i as u32 + 3, 3 * i as u32 + 5, 5 * i as u32 + 7))
//         .collect();
//     let mut layer_values_vars: Vec<_> =
//         layer_values_qm31.iter().map(|x| context.constant(*x)).collect();

//     let alpha_values = [
//         qm31_from_u32s(17, 3, 5, 7),
//         qm31_from_u32s(11, 13, 2, 9),
//         qm31_from_u32s(19, 4, 1, 6),
//     ];
//     let alphas: Vec<_> = alpha_values.iter().map(|x| context.constant(*x)).collect();

//     let mut base_point = queries.points.clone();
//     let mut bit_counter = 0usize;

//     fri_decommit(context, proof, config, fri_input, bits, packed_bits, points, &alphas);

//     context.validate_circuit();
// }
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

type FriProver<'a> = stwo::prover::fri::FriProver<'a, CpuBackend, Blake2sMerkleChannel>;
fn stwo_jumps() {
        const LOG_BLOWUP_FACTOR: u32 = 2;

        let config = FriConfig::new(2, LOG_BLOWUP_FACTOR, 3, 2);
        let column = polynomial_evaluation(6, LOG_BLOWUP_FACTOR);
        let twiddles = CpuBackend::precompute_twiddles(column.domain.half_coset);

        let prover = FriProver::commit(&mut test_channel(), config, &column, &twiddles);
        let queries = Queries::from_positions(vec![0, 3], 6 + LOG_BLOWUP_FACTOR);
        prover.decommit_on_queries(&queries);
    }