use stwo::core::circle::CirclePoint;

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::from_partial_evals;
use crate::eval;
use crate::stark_verifier::circle::double_x;

pub const N_COMPOSITION_COLUMNS: usize = 8;

#[cfg(test)]
#[path = "oods_test.rs"]
pub mod test;

/// Computes the expected value of the composition polynomial at the OODS point, based on the
/// broken composition polynomial commitment.
pub fn extract_expected_composition_eval(
    context: &mut Context<impl IValue>,
    composition_eval_at_oods: &[Var; N_COMPOSITION_COLUMNS],
    oods_point: CirclePoint<Var>,
    log_evaluation_domain_size: usize,
) -> Var {
    let composition_eval_at_oods_left =
        from_partial_evals(context, composition_eval_at_oods[0..4].try_into().unwrap());
    let composition_eval_at_oods_right =
        from_partial_evals(context, composition_eval_at_oods[4..8].try_into().unwrap());

    // Compute:
    //  `x = pi^{log_evaluation_domain_size - 2}(oods_point.x) = pi(pi(...pi(oods_point.x)...))`.
    let mut x = oods_point.x;
    for _ in 0..log_evaluation_domain_size - 2 {
        x = double_x(context, x);
    }

    eval!(context, (composition_eval_at_oods_left) + ((x) * (composition_eval_at_oods_right)))
}
