use crate::circuit_air::components::{eq, qm31_ops};
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::eval;
use crate::stark_verifier::circle::denom_inverse;
use crate::stark_verifier::constraint_eval::CircuitEval;
use crate::stark_verifier::constraint_eval::CompositionConstraintAccumulator;
use crate::stark_verifier::statement::{EvaluateArgs, Statement};

pub struct CircuitStatement {
    pub eq: eq::CircuitEqComponent,
    pub qm31_ops: qm31_ops::CircuitQm31OpsComponent,
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
            component_sizes,
        } = args;

        let mut evaluation_accumulator = CompositionConstraintAccumulator {
            oods_samples,
            composition_polynomial_coeff,
            interaction_elements,
            claimed_sums,
            component_sizes,
            accumulation: context.zero(),
            terms: Vec::new(),
        };

        self.eq.evaluate(context, &mut evaluation_accumulator);
        self.qm31_ops.evaluate(context, &mut evaluation_accumulator);

        let final_evaluation = evaluation_accumulator.finalize();

        let denom_inverse = denom_inverse(context, pt.x, log_domain_size);
        eval!(context, (final_evaluation) * (denom_inverse))
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<impl IValue>,
        _interaction_elements: [Var; 2],
    ) -> Var {
        context.zero()
    }

    fn column_log_sizes(&self, component_log_sizes: Vec<Var>) -> [Vec<Var>; 2] {
        let [size_0, size_1] = component_log_sizes[..] else {
            panic!("Expected 2 component log sizes");
        };

        let trace_column_log_sizes = vec![
            size_0, size_0, size_0, size_0, size_0, size_0, size_0, size_0, size_1, size_1, size_1,
            size_1, size_1, size_1, size_1, size_1, size_1, size_1, size_1, size_1,
        ];
        let interaction_column_log_sizes = vec![
            size_0, size_0, size_0, size_0, size_1, size_1, size_1, size_1, size_1, size_1, size_1,
            size_1,
        ];

        [trace_column_log_sizes, interaction_column_log_sizes]
    }
}
