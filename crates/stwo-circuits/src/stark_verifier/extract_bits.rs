use stwo::core::fields::m31::M31;

use crate::circuits::context::Context;
use crate::circuits::ivalue::IValue;
use crate::circuits::simd::Simd;

#[cfg(test)]
#[path = "extract_bits_test.rs"]
pub mod test;

/// For each `M31` value in the given [Simd], returns the value as a 31-bit integer in the range
/// `[0, 2^31 - 1)`.
pub fn extract_bits(context: &mut Context<impl IValue>, input: &Simd) -> [Simd; 31] {
    let inv_two = Simd::repeat(context, M31::from(2).inverse(), input.len());

    let mut value = input.clone();
    let mut bits = Vec::new();
    for _ in 0..30 {
        let lsb = value.guess_lsb(context);
        bits.push(lsb.clone());
        value = Simd::sub(context, &value, &lsb);
        value = Simd::mul(context, &value, &inv_two);
    }

    // `value` is now the MSB. Check that it is 0 or 1.
    value.assert_bits(context);
    bits.push(value.clone());

    // Check that `0` is represented as `0b0000...0000`, rather than `0b1111...1111`.
    validate_extract_bits(context, input, &bits[0]);

    bits.try_into().unwrap()
}

/// Forbids the case `0b1111...1111`, as this number should be represented as `0`.
fn validate_extract_bits(context: &mut Context<impl IValue>, input: &Simd, lsb: &Simd) {
    // Check the constraint
    //   `(input * aux - 1) * lsb = 0`,
    // which guarantees that if the input is 0, then the LSB must be 0.
    //
    // If `input != 0`, we simply choose `aux = 1/input` which satisfies the constraint.
    let zero = Simd::zero(context, input.len());
    let one = Simd::one(context, input.len());

    let aux = input.guess_inv_or_zero(context);
    let input_times_aux = Simd::mul(context, input, &aux);
    let input_times_aux_minus_one = Simd::sub(context, &input_times_aux, &one);
    let constraint_val = Simd::mul(context, &input_times_aux_minus_one, lsb);
    Simd::eq(context, &constraint_val, &zero);
}
