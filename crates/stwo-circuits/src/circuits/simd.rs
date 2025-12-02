use itertools::{Itertools, zip_eq};
use num_traits::{One, Zero};
use stwo::core::fields::FieldExpOps;
use stwo::core::fields::cm31::CM31;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::circuits::EXTENSION_DEGREE;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::{IValue, qm31_from_u32s};
use crate::circuits::ops::{Guess, add, eq, pointwise_mul, sub};
use crate::circuits::wrappers::M31Wrapper;
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
        // Sanity check: the length of data must be `ceil(len / EXTENSION_DEGREE)`.
        assert_eq!(data.len(), len.div_ceil(EXTENSION_DEGREE));
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

    /// Returns a [Simd] consisting of `len` copies of the given constant [M31] value.
    pub fn repeat(context: &mut Context<impl IValue>, value: M31, len: usize) -> Simd {
        let v = context.constant(QM31(CM31(value, value), CM31(value, value)));
        Simd { data: vec![v; len.div_ceil(EXTENSION_DEGREE)], len }
    }

    /// Returns a [Simd] consisting of `len` copies of `0`.
    pub fn zero(context: &mut Context<impl IValue>, len: usize) -> Simd {
        Self::repeat(context, M31::zero(), len)
    }

    /// Returns a [Simd] consisting of `len` copies of `1`.
    pub fn one(context: &mut Context<impl IValue>, len: usize) -> Simd {
        Self::repeat(context, M31::one(), len)
    }

    /// Adds gates to the circuit that assert that the two [Simd]s are equal.
    pub fn eq(context: &mut Context<impl IValue>, a: &Simd, b: &Simd) {
        assert_eq!(a.len, b.len);

        let n_chunks = a.len / EXTENSION_DEGREE;
        let n_rem_elements = a.len % EXTENSION_DEGREE;

        for i in 0..n_chunks {
            eq(context, a.data[i], b.data[i]);
        }

        // Handle the last elements.
        if n_rem_elements > 0 {
            let diff = eval!(context, (a.data[n_chunks]) - (b.data[n_chunks]));
            let mask = first_ones(context, n_rem_elements);
            let masked_diff = pointwise_mul(context, diff, mask);
            eq(context, masked_diff, context.zero());
        }
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

    /// Computes the inverse of each `M31` in the current [Simd].
    ///
    /// In particular, guarantees that all values are non-zero.
    pub fn inv(&self, context: &mut Context<impl IValue>) -> Simd {
        let res = self.guess_inv_or_zero(context);
        let prod = Simd::mul(context, &res, self);
        let one = Simd::one(context, self.len);

        // Note that `Simd::eq` applies only to the first `self.len` values.
        // The rest will remain unconstrained.
        Simd::eq(context, &prod, &one);

        res
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

    /// Returns `if_zero` if `selector` is 0, and `if_one` if `selector` is 1.
    /// Assumption: `selector` is either 0 or 1.
    pub fn select(
        context: &mut Context<impl IValue>,
        selector: &Simd,
        if_zero: &Simd,
        if_one: &Simd,
    ) -> Simd {
        // Compute: `if_one - if_zero`.
        let x = Simd::sub(context, if_one, if_zero);
        // Compute: `selector * (if_one - if_zero)`.
        let y = Simd::mul(context, selector, &x);
        // Compute `if_zero + selector * (if_one - if_zero)`.
        Simd::add(context, if_zero, &y)
    }

    /// Unpacks a [Simd] into a vector of [Var]s, where each [Var] represents a single [M31] value.
    pub fn unpack(context: &mut Context<impl IValue>, input: &Simd) -> Vec<Var> {
        let unit_vecs = [
            context.constant(qm31_from_u32s(1, 0, 0, 0)),
            context.constant(qm31_from_u32s(0, 1, 0, 0)),
            context.constant(qm31_from_u32s(0, 0, 1, 0)),
            context.constant(qm31_from_u32s(0, 0, 0, 1)),
        ];
        let unit_vecs_inv = [
            context.constant(qm31_from_u32s(0, 1, 0, 0).inverse()),
            context.constant(qm31_from_u32s(0, 0, 1, 0).inverse()),
            context.constant(qm31_from_u32s(0, 0, 0, 1).inverse()),
        ];

        (0..input.len)
            .map(|i| {
                let qm31_var = input.data[i / 4];
                let coord = i % 4;
                // To obtain the `coord`-th coordinate, `c`, start with pointwise multiplication
                // by a unit vector. This results in `c * unit_vecs[coord]`.
                let x = pointwise_mul(context, qm31_var, unit_vecs[coord]);
                // Then, divide by `unit_vecs[coord]` to get `c`.
                if coord == 0 { x } else { eval!(context, (x) * (unit_vecs_inv[coord - 1])) }
            })
            .collect_vec()
    }

    /// Packs a vector of [M31] values into [Simd].
    pub fn pack(context: &mut Context<impl IValue>, values: &[M31Wrapper<Var>]) -> Simd {
        let unit_vecs = [
            context.constant(qm31_from_u32s(0, 1, 0, 0)),
            context.constant(qm31_from_u32s(0, 0, 1, 0)),
            context.constant(qm31_from_u32s(0, 0, 0, 1)),
        ];

        let n = values.len();
        let data = (0..n.div_ceil(4))
            .map(|i| {
                let mut res = *values[4 * i].get();
                for j in 1..4 {
                    if 4 * i + j == n {
                        break;
                    }
                    res = eval!(context, (res) + ((unit_vecs[j - 1]) * (*values[4 * i + j].get())));
                }
                res
            })
            .collect();
        Simd::from_packed(data, values.len())
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
