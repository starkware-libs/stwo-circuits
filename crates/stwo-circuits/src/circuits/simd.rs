use itertools::{Itertools, zip_eq};

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::{IValue, qm31_from_u32s};
use crate::circuits::ops::{Guess, add, eq, pointwise_mul, sub};
use crate::eval;

#[cfg(test)]
#[path = "simd_test.rs"]
pub mod test;

/// A vector of `M31` values, represented in packed form.
///
/// Allows to efficiently perform the same operation on multiple `M31` values at once.
#[derive(Clone, Debug)]
pub struct Simd {
    /// Each [Var] is a `QM31` value, which represents 4 `M31` values (the last element may
    /// represent less than 4 values).
    ///
    /// For example, `[a, b, c, d, e, f]` may be represented as:
    /// `[a + b * i + c * u + d * iu, e + f * i]`.
    ///
    /// Note that the last coordinates of the last element are unconstrained,
    /// they are *not* guaranteed to be zero. For example, [Simd::one] returns `1` for the unused
    /// coordinates.
    data: Vec<Var>,
    /// The number of `M31` values represented by [Self::data].
    len: usize,
}
impl Simd {
    /// Constructs a new [Simd] from packed data.
    pub fn from_packed(data: Vec<Var>, len: usize) -> Self {
        // Sanity check: the length of data must be `ceil(len / 4)`.
        assert_eq!(data.len(), len.div_ceil(4));
        Self { data, len }
    }

    /// Returns the number of `M31` values represented by this [Simd].
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if there are no `M31` values in this [Simd].
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the data in packed form.
    pub fn get_packed(&self) -> &[Var] {
        &self.data
    }

    /// Adds gates to the circuit that assert that the two [Simd]s are equal.
    pub fn eq(context: &mut Context<impl IValue>, a: &Simd, b: &Simd) {
        assert_eq!(a.len, b.len);

        let mut a_iter = a.data.iter();
        let mut b_iter = b.data.iter();
        let mut rem_elements = a.len;

        while rem_elements >= 4 {
            eq(context, *a_iter.next().unwrap(), *b_iter.next().unwrap());
            rem_elements -= 4;
        }

        // Handle the last elements.
        if rem_elements > 0 {
            let diff = eval!(context, (*a_iter.next().unwrap()) - (*b_iter.next().unwrap()));
            let mask = first_ones(context, rem_elements);
            let masked_diff = pointwise_mul(context, diff, mask);
            eq(context, masked_diff, context.zero());
        }

        assert!(a_iter.next().is_none());
        assert!(b_iter.next().is_none());
    }

    /// Computes the sum of two [Simd]s (pointwise).
    pub fn add(context: &mut Context<impl IValue>, a: &Simd, b: &Simd) -> Simd {
        assert_eq!(a.len, b.len);
        Simd {
            data: zip_eq(&a.data, &b.data).map(|(x, y)| add(context, *x, *y)).collect(),
            len: a.len,
        }
    }

    /// Computes the subtraction of two [Simd]s (pointwise).
    pub fn sub(context: &mut Context<impl IValue>, a: &Simd, b: &Simd) -> Simd {
        assert_eq!(a.len, b.len);
        Simd {
            data: zip_eq(&a.data, &b.data).map(|(x, y)| sub(context, *x, *y)).collect(),
            len: a.len,
        }
    }

    /// Computes the product of two [Simd]s (pointwise).
    pub fn mul(context: &mut Context<impl IValue>, a: &Simd, b: &Simd) -> Simd {
        assert_eq!(a.len, b.len);
        Simd {
            data: zip_eq(&a.data, &b.data).map(|(x, y)| pointwise_mul(context, *x, *y)).collect(),
            len: a.len,
        }
    }

    /// Returns an *unconstrained* [Simd] initialized (hint) with `1/x` if `x != 0`,
    /// and `0` if`x = 0`.
    /// NOTE: The caller MUST verify that the result is correct.
    pub fn guess_inv_or_zero(&self, context: &mut Context<impl IValue>) -> Simd {
        let values =
            self.data.iter().map(|x| context.get(*x).pointwise_inv_or_zero()).collect_vec();
        Simd { data: values.guess(context), len: self.len }
    }

    /// Verifies that the inputs are bits (0 or 1).
    pub fn assert_bits(&self, context: &mut Context<impl IValue>) {
        // TODO(lior): Consider doing it more efficiently, by adding a constraint of the form:
        //   `input * input = input`.
        let input_sqr = Simd::mul(context, self, self);
        Simd::eq(context, self, &input_sqr);
    }

    /// Returns an *unconstrained* [Simd] initialized (hint) with the LSB of each `M31` in the
    /// current [Simd].
    ///
    /// NOTE: The result is guaranteed to be either 0 or 1, but the caller MUST verify that it is
    /// indeed the LSB of the input.
    pub fn guess_lsb(&self, context: &mut Context<impl IValue>) -> Simd {
        let values = self.data.iter().map(|x| context.get(*x).pointwise_lsb()).collect_vec();
        let out = Simd { data: values.guess(context), len: self.len };
        out.assert_bits(context);
        out
    }
}

/// Returns a (constant) [Var] with the first `n` coordinates set to 1, and the rest to 0.
///
/// `n` must be between 1 and 3.
fn first_ones(context: &mut Context<impl IValue>, n: usize) -> Var {
    match n {
        1 => context.constant(qm31_from_u32s(1, 0, 0, 0)),
        2 => context.constant(qm31_from_u32s(1, 1, 0, 0)),
        3 => context.constant(qm31_from_u32s(1, 1, 1, 0)),
        _ => panic!("Unsupported number of ones: {n}"),
    }
}
