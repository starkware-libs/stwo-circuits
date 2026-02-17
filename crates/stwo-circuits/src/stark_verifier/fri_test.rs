use crate::circuits::blake::HashValue;
use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::ops::Guess;
use crate::circuits::simd::Simd;
use crate::circuits::test_utils::{packed_values, simd_from_u32s};
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::fri_proof::FriCommitProof;
use crate::stark_verifier::select_queries::select_queries;
use rstest::rstest;

use super::{fri_commit, update_base_point, validate_query_position_in_coset};

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
fn test_update_base_point_logic() {
    let mut context = TraceContext::default();
    const LOG_DOMAIN_SIZE: usize = 7;

    // Query indices are interpreted by `select_queries`.
    let query_indices = vec![0b1001111_u32, 0b0110101_u32, 0b1100010_u32];
    let input = simd_from_u32s(&mut context, query_indices.clone());
    let queries = select_queries(&mut context, &input, LOG_DOMAIN_SIZE);

    // Test several jump windows: each should clear the corresponding consumed bits.
    let jump_windows = [(0_usize, 2_usize), (2, 2), (1, 3)];
    for (bit_counter, step) in jump_windows {
        let bit_range = (1 + bit_counter)..(1 + bit_counter + step);
        let updated_base_point =
            update_base_point(&mut context, queries.points.clone(), &queries.bits, bit_range.clone());

        let mask = bit_range.clone().fold(0_u32, |acc, i| acc | (1_u32 << i));
        let expected_indices = query_indices.iter().map(|q| *q & !mask).collect::<Vec<_>>();
        let expected_input = simd_from_u32s(&mut context, expected_indices);
        let expected_queries = select_queries(&mut context, &expected_input, LOG_DOMAIN_SIZE);

        assert_eq!(
            simd_lanes(&context, &updated_base_point.x),
            simd_lanes(&context, &expected_queries.points.x)
        );
        assert_eq!(
            simd_lanes(&context, &updated_base_point.y),
            simd_lanes(&context, &expected_queries.points.y)
        );
    }

    context.validate_circuit();
}
