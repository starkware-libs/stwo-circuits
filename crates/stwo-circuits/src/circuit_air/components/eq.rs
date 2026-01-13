use crate::circuit_air::components::prelude::*;

pub const N_PREPROCESSED_COLUMNS: usize = 2;
pub const N_TRACE_COLUMNS: usize = 8;

pub struct Eval {
    pub log_size: u32,
    pub gate_lookup_elements: relations::Gate,
}

pub type Component = FrameworkComponent<Eval>;

impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        self.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let in0_address =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "eq_in0_address".to_owned() });
        let in1_address =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "eq_in1_address".to_owned() });

        let in0_col0 = eval.next_trace_mask();
        let in0_col1 = eval.next_trace_mask();
        let in0_col2 = eval.next_trace_mask();
        let in0_col3 = eval.next_trace_mask();
        let in1_col4 = eval.next_trace_mask();
        let in1_col5 = eval.next_trace_mask();
        let in1_col6 = eval.next_trace_mask();
        let in1_col7 = eval.next_trace_mask();

        // in0 col 0 equals in1 col 4.
        eval.add_constraint(in0_col0.clone() - in1_col4.clone());

        // in0 col 1 equals in1 col 5.
        eval.add_constraint(in0_col1.clone() - in1_col5.clone());

        // in0 col 2 equals in1 col 6.
        eval.add_constraint(in0_col2.clone() - in1_col6.clone());

        // in0 col 3 equals in1 col 7.
        eval.add_constraint(in0_col3.clone() - in1_col7.clone());

        eval.add_to_relation(RelationEntry::new(
            &self.gate_lookup_elements,
            E::EF::one(),
            &[
                in0_address.clone(),
                in0_col0.clone(),
                in0_col1.clone(),
                in0_col2.clone(),
                in0_col3.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.gate_lookup_elements,
            E::EF::one(),
            &[
                in1_address.clone(),
                in1_col4.clone(),
                in1_col5.clone(),
                in1_col6.clone(),
                in1_col7.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}

pub struct CircuitEqComponent {
    pub preprocessed_column_indices: [usize; N_PREPROCESSED_COLUMNS],
}

impl<Value: IValue> CircuitEval<Value> for CircuitEqComponent {
    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &ComponentData<'_>,
        acc: &mut CompositionConstraintAccumulator<'_>,
    ) {
        let [in0_address, in1_address] = acc
            .get_preprocessed_columns::<N_PREPROCESSED_COLUMNS>(self.preprocessed_column_indices);

        let [in0_col0, in0_col1, in0_col2, in0_col3, in1_col4, in1_col5, in1_col6, in1_col7] =
            *component_data.trace_columns
        else {
            panic!("Expected {N_TRACE_COLUMNS} trace columns")
        };

        // in0 col 0 equals in1 col 4.
        let constraint0_val = eval!(context, (in0_col0) - (in1_col4));
        acc.add_constraint(context, constraint0_val);

        // in0 col 1 equals in1 col 5.
        let constraint1_val = eval!(context, (in0_col1) - (in1_col5));
        acc.add_constraint(context, constraint1_val);

        // in0 col 2 equals in1 col 6.
        let constraint2_val = eval!(context, (in0_col2) - (in1_col6));
        acc.add_constraint(context, constraint2_val);

        // in0 col 3 equals in1 col 7.
        let constraint3_val = eval!(context, (in0_col3) - (in1_col7));
        acc.add_constraint(context, constraint3_val);

        acc.add_to_relation(
            context,
            context.one(),
            &[in0_address, in0_col0, in0_col1, in0_col2, in0_col3],
        );
        acc.add_to_relation(
            context,
            context.one(),
            &[in1_address, in1_col4, in1_col5, in1_col6, in1_col7],
        );
    }
}
