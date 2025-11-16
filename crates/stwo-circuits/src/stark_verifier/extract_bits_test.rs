use rstest::rstest;

use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::test_utils::{packed_values, simd_from_u32s};
use crate::stark_verifier::extract_bits::{extract_bits, validate_extract_bits};

#[test]
fn test_extract_bits() {
    let mut context = TraceContext::default();
    let input = simd_from_u32s(&mut context, vec![0, 12, 2_u32.pow(31) - 2, 2_u32.pow(31) - 1]);
    let bits = extract_bits(&mut context, &input);
    assert_eq!(packed_values(&context, &bits[0]), &[qm31_from_u32s(0, 0, 0, 0)]);
    assert_eq!(packed_values(&context, &bits[1]), &[qm31_from_u32s(0, 0, 1, 0)]);
    assert_eq!(packed_values(&context, &bits[2]), &[qm31_from_u32s(0, 1, 1, 0)]);
    assert_eq!(packed_values(&context, &bits[3]), &[qm31_from_u32s(0, 1, 1, 0)]);
    for bit in &bits[4..31] {
        assert_eq!(packed_values(&context, bit), &[qm31_from_u32s(0, 0, 1, 0)]);
    }

    context.circuit.check(context.values()).unwrap();
}

#[rstest]
#[case::success(true)]
#[case::failure(false)]
fn test_validate_extract_bits(#[case] success: bool) {
    let mut context = TraceContext::default();
    // If input is not zero, both 0 and 1 are allowed by `validate_extract_bits`.
    // If it is zero, only lsb=0 is allowed.
    let lsb = simd_from_u32s(&mut context, vec![if success { 0 } else { 1 }, 1, 0, 1, 1, 1]);
    let input = simd_from_u32s(&mut context, vec![0, 1, 2, 3, 4, 5]);

    validate_extract_bits(&mut context, &input, &lsb);

    assert_eq!(context.circuit.check(context.values()).is_ok(), success);
}
