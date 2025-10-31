use blake2::{Blake2s256, Digest};
use rstest::rstest;
use stwo::core::vcs::blake2_hash::reduce_to_m31;

use crate::circuits::blake::{blake, qm31_from_bytes};
use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::ops::eq;
use crate::circuits::stats::Stats;

#[rstest]
#[case::success(false)]
#[case::wrong_output(true)]
fn test_blake(#[case] wrong_output: bool) {
    let mut context = TraceContext::default();

    let input = [
        context.new_var(qm31_from_u32s(1, 2, 3, 4)),
        context.new_var(qm31_from_u32s(5, 6, 7, 8)),
        context.new_var(qm31_from_u32s(9, 10, 11, 12)),
        context.new_var(qm31_from_u32s(13, 14, 15, 16)),
        context.new_var(qm31_from_u32s(17, 0, 0, 0)),
    ];

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
    let out0 = context.new_var(qm31_from_bytes(expected_hash[0..16].try_into().unwrap()));
    let out1 = context.new_var(qm31_from_bytes(expected_hash[16..32].try_into().unwrap()));
    eq(&mut context, output.0, out0);
    eq(&mut context, output.1, out1);

    assert_eq!(context.stats, Stats { blake: 2, guess: 2, eq: 2, ..Stats::default() });

    assert_eq!(context.circuit.check(context.values()).is_ok(), !wrong_output);
}
