use stwo::core::circle::{CirclePoint, M31_CIRCLE_GEN};
use stwo::core::fields::m31::M31;

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::div;
use crate::circuits::simd::Simd;
use crate::circuits::wrappers::M31Wrapper;
use crate::eval;

#[cfg(test)]
#[path = "circle_test.rs"]
pub mod test;

/// Computes `pi(x) = 2 * x^2 - 1`, which is the x-coordinate of the point `(x, y) + (x, y)`.
pub fn double_x(context: &mut Context<impl IValue>, value: Var) -> Var {
    let value_sqr = eval!(context, (value) * (value));
    eval!(context, ((value_sqr) + (value_sqr)) - (1))
}

/// Same as [double_x], but for [Simd].
pub fn double_x_simd(context: &mut Context<impl IValue>, value: &Simd) -> Simd {
    let value_sqr = Simd::mul(context, value, value);
    let value_sqr_times2 = Simd::add(context, &value_sqr, &value_sqr);
    let one = Simd::one(context, value.len());
    Simd::sub(context, &value_sqr_times2, &one)
}

/// Computes `p + p`.
pub fn double_point(
    context: &mut Context<impl IValue>,
    p: &CirclePoint<M31Wrapper<Var>>,
) -> CirclePoint<M31Wrapper<Var>> {
    let xy = eval!(context, (*p.x.get()) * (*p.y.get()));
    let new_y = eval!(context, (xy) + (xy));
    CirclePoint {
        x: M31Wrapper::new_unsafe(double_x(context, *p.x.get())),
        y: M31Wrapper::new_unsafe(new_y),
    }
}

/// Computes `p + p`.
pub fn double_point_simd(
    context: &mut Context<impl IValue>,
    p: &CirclePoint<Simd>,
) -> CirclePoint<Simd> {

}

/// Computes `p + p`.
pub fn repeated_double_point(
    context: &mut Context<impl IValue>,
    p: &CirclePoint<M31Wrapper<Var>>,
) -> CirclePoint<M31Wrapper<Var>> {
    let xy = eval!(context, (*p.x.get()) * (*p.y.get()));
    let new_y = eval!(context, (xy) + (xy));
    CirclePoint {
        x: M31Wrapper::new_unsafe(double_x(context, *p.x.get())),
        y: M31Wrapper::new_unsafe(new_y),
    }
}

/// Computes `point0 + point1` on the circle.
pub fn add_points(
    context: &mut Context<impl IValue>,
    point0: &CirclePoint<Var>,
    point1: &CirclePoint<Var>,
) -> CirclePoint<Var> {
    let x = eval!(context, ((point0.x) * (point1.x)) - ((point0.y) * (point1.y)));
    let y = eval!(context, ((point0.x) * (point1.y)) + ((point0.y) * (point1.x)));
    CirclePoint { x, y }
}

/// A version of [add_points] for [Simd].
pub fn add_points_simd(
    context: &mut Context<impl IValue>,
    point0: &CirclePoint<Simd>,
    point1: &CirclePoint<Simd>,
) -> CirclePoint<Simd> {
    let x0x1 = Simd::mul(context, &point0.x, &point1.x);
    let x0y1 = Simd::mul(context, &point0.x, &point1.y);
    let y0x1 = Simd::mul(context, &point0.y, &point1.x);
    let y0y1 = Simd::mul(context, &point0.y, &point1.y);
    let x = Simd::sub(context, &x0x1, &y0y1);
    let y = Simd::add(context, &x0y1, &y0x1);
    CirclePoint { x, y }
}

/// Computes the generator point of the subgroup of size `2^log_domain_size`.
pub fn generator_point(log_domain_size: usize) -> CirclePoint<M31> {
    M31_CIRCLE_GEN.repeated_double((31 - log_domain_size) as u32)
}

/// Computes the generator point of the subgroup of size `2^log_domain_size`, repeated `size` times.
pub fn generator_point_simd(
    context: &mut Context<impl IValue>,
    log_domain_size: usize,
    size: usize,
) -> CirclePoint<Simd> {
    let pt = generator_point(log_domain_size);
    CirclePoint { x: Simd::repeat(context, pt.x, size), y: Simd::repeat(context, pt.y, size) }
}

/// Computes the polynomial that vanishes on the canonical coset of size `2^log_trace_size`.
///
/// The polynomial is `pi^{log_trace_size - 1}(x) = pi(...(pi(x))...)`.
pub fn coset_vanishing_poly(
    context: &mut Context<impl IValue>,
    mut x: Var,
    log_trace_size: usize,
) -> Var {
    assert!(log_trace_size >= 1);

    for _ in 0..(log_trace_size - 1) {
        x = double_x(context, x);
    }
    x
}

/// Computes the inverse of the domain polynomial at `x`. See [coset_vanishing_poly].
pub fn denom_inverse(context: &mut Context<impl IValue>, x: Var, log_trace_size: usize) -> Var {
    let one = context.one();
    let denom = coset_vanishing_poly(context, x, log_trace_size);
    div(context, one, denom)
}
