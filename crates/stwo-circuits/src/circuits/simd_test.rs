use indoc::formatdoc;
use num_traits::One;
use rstest::rstest;
use stwo::core::fields::cm31::CM31;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::circuits::context::{Context, TraceContext};
use crate::circuits::ivalue::{NoValue, qm31_from_u32s};
use crate::circuits::ops::{Guess, guess};
use crate::circuits::simd::Simd;
use crate::circuits::test_utils::{packed_values, simd_from_u32s};

#[test]
fn test_repeat() {
    let mut context = TraceContext::default();
    let a = Simd::repeat(&mut context, M31::from(2), 6);
    let twos = qm31_from_u32s(2, 2, 2, 2);
    assert_eq!(a.len(), 6);
    assert_eq!(packed_values(&context, &a), &[twos, twos]);
    context.circuit.check(context.values()).unwrap();
}

#[test]
fn test_zero() {
    let mut context = TraceContext::default();
    let a = Simd::zero(&mut context, 6);
    let zero = 0.into();
    assert_eq!(a.len(), 6);
    assert_eq!(packed_values(&context, &a), &[zero, zero]);
    context.circuit.check(context.values()).unwrap();
}

#[test]
fn test_one() {
    let mut context = TraceContext::default();
    let a = Simd::one(&mut context, 6);
    let ones = qm31_from_u32s(1, 1, 1, 1);
    assert_eq!(a.len(), 6);
    assert_eq!(packed_values(&context, &a), &[ones, ones]);
    context.circuit.check(context.values()).unwrap();
}
#[test]
fn test_simd_basic_ops() {
    let mut context = TraceContext::default();

    let a = simd_from_u32s(&mut context, vec![1, 2, 3, 4, 5, 6]);
    let b = simd_from_u32s(&mut context, vec![7, 9, 11, 13, 15, 17]);

    assert_eq!(a.len(), 6);

    let a_add_b = Simd::add(&mut context, &a, &b);
    assert_eq!(
        packed_values(&context, &a_add_b),
        &[qm31_from_u32s(8, 11, 14, 17), qm31_from_u32s(20, 23, 0, 0)]
    );

    let b_sub_a = Simd::sub(&mut context, &b, &a);
    assert_eq!(
        packed_values(&context, &b_sub_a),
        &[qm31_from_u32s(6, 7, 8, 9), qm31_from_u32s(10, 11, 0, 0)]
    );

    let a_mul_b = Simd::mul(&mut context, &a, &b);
    assert_eq!(
        packed_values(&context, &a_mul_b),
        &[qm31_from_u32s(7, 18, 33, 52), qm31_from_u32s(75, 102, 0, 0)]
    );

    context.circuit.check(context.values()).unwrap();
}

#[test]
fn test_eq_circuit8() {
    let mut context = Context::<NoValue>::default();
    let a_simd = Simd::from_packed(vec![NoValue; 2].guess(&mut context), 8);
    let b_simd = Simd::from_packed(vec![NoValue; 2].guess(&mut context), 8);
    Simd::eq(&mut context, &a_simd, &b_simd);
    assert_eq!(format!("{:?}", a_simd.data), "[[2], [3]]");
    assert_eq!(format!("{:?}", b_simd.data), "[[4], [5]]");
    assert_eq!(
        format!("{:?}", context.circuit),
        formatdoc!(
            "
            [0] = [0] + [0]
            [1] = [1] + [0]
            [2] = [2] + [0]
            [3] = [3] + [0]
            [4] = [4] + [0]
            [5] = [5] + [0]
            [2] = [4]
            [3] = [5]
            "
        )
    );
}

#[test]
fn test_eq_circuit7() {
    let mut context = Context::<NoValue>::default();
    let a_simd = Simd::from_packed(vec![NoValue; 2].guess(&mut context), 7);
    let b_simd = Simd::from_packed(vec![NoValue; 2].guess(&mut context), 7);
    Simd::eq(&mut context, &a_simd, &b_simd);
    assert_eq!(format!("{:?}", a_simd.data), "[[2], [3]]");
    assert_eq!(format!("{:?}", b_simd.data), "[[4], [5]]");
    assert_eq!(
        format!("{:?}", context.constants()),
        "{(0 + 0i) + (0 + 0i)u: [0], (1 + 0i) + (0 + 0i)u: [1], (1 + 1i) + (1 + 0i)u: [7]}"
    );
    assert_eq!(
        format!("{:?}", context.circuit),
        formatdoc!(
            "
            [0] = [0] + [0]
            [1] = [1] + [0]
            [2] = [2] + [0]
            [3] = [3] + [0]
            [4] = [4] + [0]
            [5] = [5] + [0]
            [7] = [7] + [0]
            [6] = [3] - [5]
            [8] = [6] x [7]
            [2] = [4]
            [8] = [0]
            "
        )
    );
}

#[test]
fn test_eq() {
    // Go over possible lengths.
    for len in 0..=8_usize {
        let n_qm31s = len.div_ceil(4);
        // Go over all possible coordinates in the padded array, including coordinates greater than
        // the actual length.
        for wrong_coord in 0..(4 * n_qm31s) {
            let mut context = TraceContext::default();

            let mut vals: Vec<u32> = (0..8).collect();
            let a = vec![
                qm31_from_u32s(vals[0], vals[1], vals[2], vals[3]),
                qm31_from_u32s(vals[4], vals[5], vals[6], vals[7]),
            ]
            .guess(&mut context);

            vals[wrong_coord] += 1;
            let b = vec![
                qm31_from_u32s(vals[0], vals[1], vals[2], vals[3]),
                qm31_from_u32s(vals[4], vals[5], vals[6], vals[7]),
            ]
            .guess(&mut context);

            Simd::eq(
                &mut context,
                &Simd::from_packed(a[..n_qm31s].to_vec(), len),
                &Simd::from_packed(b[..n_qm31s].to_vec(), len),
            );

            // There should be an error if the wrong coordinate is within the range of the actual
            // length.
            assert_eq!(context.circuit.check(context.values()).is_err(), wrong_coord < len);
        }
    }
}

#[test]
fn test_guess_inv_or_zero() {
    let mut context = TraceContext::default();

    let a = simd_from_u32s(&mut context, vec![2, 0, 3]);
    let a_inv = a.guess_inv_or_zero(&mut context);
    assert_eq!(a_inv.len(), 3);

    let one = M31::one();
    assert_eq!(
        packed_values(&context, &a_inv),
        &[QM31(CM31(one / M31::from(2), 0.into()), CM31(one / M31::from(3), 0.into()))]
    );

    let mut values = context.values().clone();
    context.circuit.check(&values).unwrap();

    // As the value is not enforced, the circuit passes with a changed value as well.
    values[a_inv.get_packed()[0].idx] += QM31::from(1);
    context.circuit.check(&values).unwrap();
}

#[rstest]
#[case::zeros([0, 0, 0, 0], true)]
#[case::ones([1, 1, 1, 0], true)]
#[case::mix([1, 0, 1, 0], true)]
#[case::two([2, 0, 0, 0], false)]
#[case::last_coord_ignored([0, 0, 0, 2], true)]
fn test_assert_bits(#[case] vals: [u32; 4], #[case] success: bool) {
    let mut context = TraceContext::default();

    let a = Simd::from_packed(
        vec![guess(&mut context, qm31_from_u32s(vals[0], vals[1], vals[2], vals[3]))],
        3,
    );
    a.assert_bits(&mut context);

    assert_eq!(context.circuit.check(context.values()).is_ok(), success);
}

#[test]
fn test_guess_lsb() {
    let mut context = TraceContext::default();

    let a = simd_from_u32s(&mut context, vec![7, 1, 4, 6, 11, 8]);
    let b = a.guess_lsb(&mut context);

    assert_eq!(b.len(), 6);
    assert_eq!(
        packed_values(&context, &b),
        [qm31_from_u32s(1, 1, 0, 0), qm31_from_u32s(1, 0, 0, 0)]
    );

    context.circuit.check(context.values()).unwrap();
}

#[test]
fn test_select() {
    let mut context = TraceContext::default();

    let selector = simd_from_u32s(&mut context, vec![1, 0, 1]);
    let if_zero = simd_from_u32s(&mut context, vec![1, 2, 3]);
    let if_one = simd_from_u32s(&mut context, vec![4, 5, 6]);

    let result = Simd::select(&mut context, &selector, &if_zero, &if_one);

    assert_eq!(result.len(), 3);
    assert_eq!(packed_values(&context, &result), &[qm31_from_u32s(4, 2, 6, 0)]);

    context.circuit.check(context.values()).unwrap();
}
