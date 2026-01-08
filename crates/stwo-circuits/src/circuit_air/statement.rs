use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::eval;
use crate::stark_verifier::circle::denom_inverse;
use crate::stark_verifier::constraint_eval::CircuitEval;
use crate::stark_verifier::constraint_eval::CompositionConstraintAccumulator;
use crate::stark_verifier::statement::{EvaluateArgs, Statement};

pub struct CircuitStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
}
impl<Value: IValue> Statement<Value> for CircuitStatement<Value> {
    fn evaluate(&self, context: &mut Context<Value>, args: EvaluateArgs<'_>) -> Var {
        let EvaluateArgs {
            oods_samples,
            pt,
            log_domain_size,
            composition_polynomial_coeff,
            interaction_elements,
            component_data,
        } = args;

        let mut evaluation_accumulator = CompositionConstraintAccumulator {
            oods_samples,
            composition_polynomial_coeff,
            interaction_elements,
            component_data,
            accumulation: context.zero(),
            terms: Vec::new(),
        };

        for component in &self.components {
            component.evaluate(context, &mut evaluation_accumulator);
        }

        let final_evaluation = evaluation_accumulator.finalize();

        let denom_inverse = denom_inverse(context, pt.x, log_domain_size);
        eval!(context, (final_evaluation) * (denom_inverse))
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<Value>,
        _interaction_elements: [Var; 2],
    ) -> Var {
        context.zero()
    }
}
