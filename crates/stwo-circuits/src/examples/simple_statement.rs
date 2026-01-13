use num_traits::One;
use stwo::core::fields::m31::M31;

use super::simple_air::FIB_SEQUENCE_LENGTH;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::div;
use crate::eval;
use crate::examples::simple_air::{LOG_SIZE_LONG, LOG_SIZE_SHORT};
use crate::stark_verifier::constraint_eval::{CircuitEval, CompositionConstraintAccumulator};
use crate::stark_verifier::logup::combine_term;
use crate::stark_verifier::statement::Statement;

pub struct SimpleStatement<Value: IValue> {
    log_component_sizes: Vec<u32>,
    components: Vec<Box<dyn CircuitEval<Value>>>,
}

impl<Value: IValue> Default for SimpleStatement<Value> {
    fn default() -> Self {
        Self {
            log_component_sizes: vec![LOG_SIZE_LONG, LOG_SIZE_SHORT],
            components: vec![
                Box::new(SquaredFibonacciComponent { preprocessed_column_idx: 1 }),
                Box::new(SquaredFibonacciComponent { preprocessed_column_idx: 0 }),
            ],
        }
    }
}

pub struct SquaredFibonacciComponent {
    pub preprocessed_column_idx: usize,
}
impl<Value: IValue> CircuitEval<Value> for SquaredFibonacciComponent {
    fn evaluate(
        &self,
        context: &mut Context<Value>,
        trace_columns: &[Var],
        acc: &mut CompositionConstraintAccumulator<'_>,
    ) {
        let const_val = acc.get_preprocessed_column(self.preprocessed_column_idx);
        let [a, b, c, d] = *trace_columns else { panic!("Expected 4 trace columns") };

        // Constraints.
        let constraint0_val = eval!(context, (c) - ((((a) * (a)) + ((b) * (b))) + (const_val)));
        acc.add_constraint(context, constraint0_val);

        let constraint1_val = eval!(context, (d) - ((((b) * (b)) + ((c) * (c))) + (const_val)));
        acc.add_constraint(context, constraint1_val);

        // Logup constraint.
        acc.add_to_relation(context, context.one(), &[c, d]);
        acc.add_to_relation(context, context.one(), &[c, d]);
        acc.add_to_relation(context, context.one(), &[c, d]);
    }
}

fn squared_fibonacci_public_logup_sum(
    context: &mut Context<impl IValue>,
    prev_sum: Var,
    interaction_elements: [Var; 2],
    log_n_instances: u32,
) -> Var {
    let mut sum = prev_sum;
    for j in 0..(1 << log_n_instances) {
        let mut a: M31 = M31::one();
        let mut b: M31 = j.into();
        for _ in 0..(FIB_SEQUENCE_LENGTH - 2) {
            (a, b) = (b, a * a + b * b + M31::from(j));
        }
        let elements = [context.constant(a.into()), context.constant(b.into())];
        let denom1 = combine_term(context, &elements, interaction_elements);

        let denom = eval!(context, (denom1) * (denom1));
        let numerator = eval!(context, (denom1) + (denom1));

        let frac0 = div(context, numerator, denom);
        let frac1 = div(context, context.one(), denom1);
        let frac = eval!(context, (frac0) + (frac1));

        // Note that the sum is negated because we want to use the values that are yielded in
        // the witness.
        sum = eval!(context, (sum) - (frac));
    }
    sum
}

impl<Value: IValue> Statement<Value> for SimpleStatement<Value> {
    fn get_components(&self) -> &[Box<dyn CircuitEval<Value>>] {
        &self.components
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<Value>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let mut prev_sum = context.zero();
        for log_n_instances in &self.log_component_sizes {
            prev_sum = squared_fibonacci_public_logup_sum(
                context,
                prev_sum,
                interaction_elements,
                *log_n_instances,
            );
        }
        prev_sum
    }
}
