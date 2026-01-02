use crate::circuit_air::components::{eq, qm31_ops};
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::eval;
use crate::stark_verifier::circle::denom_inverse;
use crate::stark_verifier::component::CircuitEval;
use crate::stark_verifier::component::CompositionConstraintAccumulator;
use crate::stark_verifier::statement::{EvaluateArgs, Statement};

pub struct CircuitStatement {
    pub qm31_ops: qm31_ops::CircuitQm31OpsComponent,
    pub eq: eq::CircuitEqComponent,
}
impl Statement for CircuitStatement {
    fn evaluate(&self, context: &mut Context<impl IValue>, args: EvaluateArgs<'_>) -> Var {
        let EvaluateArgs {
            oods_samples,
            pt,
            log_domain_size,
            composition_polynomial_coeff,
            interaction_elements,
            claimed_sums,
        } = args;

        let mut evaluation_accumulator = CompositionConstraintAccumulator {
            oods_samples,
            composition_polynomial_coeff,
            interaction_elements,
            claimed_sums,
            accumulation: context.zero(),
            fracs: Vec::new(),
        };

        self.qm31_ops.evaluate(context, &mut evaluation_accumulator);
        self.eq.evaluate(context, &mut evaluation_accumulator);

        let final_evaluation = evaluation_accumulator.finalize();

        let denom_inverse = denom_inverse(context, pt.x, log_domain_size);
        eval!(context, (final_evaluation) * (denom_inverse))
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<impl IValue>,
        _interaction_elements: [Var; 2],
    ) -> Var {
        let mut sum = context.zero();

        let qm31_ops_claimed_sum = context.constant(self.qm31_ops.claimed_sum);
        sum = eval!(context, (sum) + (qm31_ops_claimed_sum));

        let eq_claimed_sum = context.constant(self.eq.claimed_sum);
        sum = eval!(context, (sum) + (eq_claimed_sum));

        sum
    }
}
