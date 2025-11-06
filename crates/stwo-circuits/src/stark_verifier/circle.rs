use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::eval;

/// Computes `pi(x) = 2 * x^2 - 1`, which is the x-coordinate of the point `(x, y) + (x, y)`.
pub fn double_x(context: &mut Context<impl IValue>, value: Var) -> Var {
    let value_sqr = eval!(context, (value) * (value));
    eval!(context, ((value_sqr) + (value_sqr)) - (1))
}
