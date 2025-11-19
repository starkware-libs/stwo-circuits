use stwo::core::circle::CirclePoint;

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::simd::Simd;
use crate::eval;

#[cfg(test)]
#[path = "circle_test.rs"]
pub mod test;

/// Computes `pi(x) = 2 * x^2 - 1`, which is the x-coordinate of the point `(x, y) + (x, y)`.
pub fn double_x(context: &mut Context<impl IValue>, value: Var) -> Var {
    let value_sqr = eval!(context, (value) * (value));
    eval!(context, ((value_sqr) + (value_sqr)) - (1))
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
