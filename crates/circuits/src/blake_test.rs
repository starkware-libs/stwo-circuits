use blake2::{Blake2s256, Digest};
use rstest::rstest;
use stwo::core::vcs::blake2_hash::reduce_to_m31;

use crate::blake::{blake, blake_qm31, qm31_from_bytes};
use crate::context::TraceContext;
use crate::finalize_constants::finalize_constants;
use crate::ivalue::qm31_from_u32s;
use crate::ops::{Guess, eq, guess};
use crate::stats::Stats;

#[rstest]
#[case::success(false)]
#[case::wrong_output(true)]
fn test_blake(#[case] wrong_output: bool) {
    let mut context = TraceContext::default();

    let input = [
        qm31_from_u32s(1, 2, 3, 4),
        qm31_from_u32s(5, 6, 7, 8),
        qm31_from_u32s(9, 10, 11, 12),
        qm31_from_u32s(13, 14, 15, 16),
        qm31_from_u32s(17, 0, 0, 0),
    ]
    .guess(&mut context);

    let mut hasher = Blake2s256::new();
    hasher.update([
        1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8, 0,
        0, 0, 9, 0, 0, 0, 10, 0, 0, 0, 11, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 15, 0,
        0, 0, 16, 0, 0, 0, 17, 0,
    ]);
    let mut expected_hash: [u8; 32] = reduce_to_m31(hasher.finalize().into());
    if wrong_output {
        expected_hash[0] += 1;
    }

    let output = blake(&mut context, &input, 66);
    let out0 = guess(&mut context, qm31_from_bytes(expected_hash[0..16].try_into().unwrap()));
    let out1 = guess(&mut context, qm31_from_bytes(expected_hash[16..32].try_into().unwrap()));
    eq(&mut context, output.0, out0);
    eq(&mut context, output.1, out1);

    assert_eq!(
        context.stats,
        Stats {
            equals: 2,
            add: 14,
            sub: 0,
            mul: 37,
            div: 0,
            pointwise_mul: 36,
            guess: 7,
            blake_updates: 1,
            permutation_inputs: 0,
            outputs: 0,
            triple_xor: 16,
            m31_to_u32: 20
        }
    );

    crate::finalize_constants::finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.circuit.check_yields();

    assert_eq!(context.is_circuit_valid(), !wrong_output);
}

#[test]
fn test_blake_qm31() {
    let mut context = TraceContext::default();

    let message: [u32; 16] = [
        930933030, 1766240503, 3660871006, 388409270, 1948594622, 3119396969, 3924579183,
        2089920034, 3857888532, 929304360, 1810891574, 860971754, 1822893775, 2008495810,
        2958962335, 2340515744,
    ];
    let n_bytes = 64;

    let input_values = [
        qm31_from_u32s(message[0], message[1], message[2], message[3]),
        qm31_from_u32s(message[4], message[5], message[6], message[7]),
        qm31_from_u32s(message[8], message[9], message[10], message[11]),
        qm31_from_u32s(message[12], message[13], message[14], message[15]),
    ];

    let expected = blake_qm31(&input_values, n_bytes);

    let input = input_values.guess(&mut context);
    let output = blake(&mut context, &input, n_bytes);

    let out0 = guess(&mut context, expected.0);
    let out1 = guess(&mut context, expected.1);
    eq(&mut context, output.0, out0);
    eq(&mut context, output.1, out1);

    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    assert!(context.is_circuit_valid());
}
