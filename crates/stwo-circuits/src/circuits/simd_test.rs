use stwo::core::fields::qm31::QM31;

use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::simd::Simd;
use crate::circuits::test_utils::simd_from_u32s;

/// Given a [Simd], returns the values of the packed `QM31`s.
fn packed_values(context: &mut TraceContext, simd: &Simd) -> Vec<QM31> {
    simd.get_packed().iter().map(|v| context.get(*v)).collect()
}

#[test]
fn test_simd_basic_ops() {
    let mut context = TraceContext::default();

    let a = simd_from_u32s(&mut context, vec![1, 2, 3, 4, 5, 6]);
    let b = simd_from_u32s(&mut context, vec![7, 9, 11, 13, 15, 17]);

    assert_eq!(a.len(), 6);

    let a_add_b = Simd::add(&mut context, &a, &b);
    assert_eq!(
        packed_values(&mut context, &a_add_b),
        &[qm31_from_u32s(8, 11, 14, 17), qm31_from_u32s(20, 23, 0, 0)]
    );

    let b_sub_a = Simd::sub(&mut context, &b, &a);
    assert_eq!(
        packed_values(&mut context, &b_sub_a),
        &[qm31_from_u32s(6, 7, 8, 9), qm31_from_u32s(10, 11, 0, 0)]
    );

    let a_mul_b = Simd::mul(&mut context, &a, &b);
    assert_eq!(
        packed_values(&mut context, &a_mul_b),
        &[qm31_from_u32s(7, 18, 33, 52), qm31_from_u32s(75, 102, 0, 0)]
    );

    context.circuit.check(context.values()).unwrap();
}
