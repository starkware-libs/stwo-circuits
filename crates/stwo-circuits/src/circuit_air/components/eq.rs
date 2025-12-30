use crate::circuit_air::components::prelude::*;

pub const N_PREPROCESSED_COLUMNS: usize = 2;
pub const N_TRACE_COLUMNS: usize = 8;
pub const N_INTERACTION_COLUMNS: usize = 1;

pub struct Eval {
    pub log_size: u32,
    pub gate_lookup_elements: relations::Gate,
    pub preprocessed_column_indices: [usize; N_PREPROCESSED_COLUMNS],
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

pub struct Statement {
    pub preprocessed_column_indices: [usize; N_PREPROCESSED_COLUMNS],
    pub log_size: u32,
}
impl crate::stark_verifier::statement::Statement for Statement {
    fn evaluate(
        &self,
        context: &mut Context<impl IValue>,
        acc: &mut CompositionConstraintAccumulator<'_>,
    ) {
        let in0_address =
            acc.oods_samples.preprocessed_columns[self.preprocessed_column_indices[0]];
        let in1_address =
            acc.oods_samples.preprocessed_columns[self.preprocessed_column_indices[1]];

        let Some(&[in0_col0, in0_col1, in0_col2, in0_col3, in1_col4, in1_col5, in1_col6, in1_col7]) =
            acc.oods_samples.trace.split_off(..N_TRACE_COLUMNS)
        else {
            panic!("Expected {} trace values", N_TRACE_COLUMNS);
        };
        let Some(
            [interaction_0_limb0, interaction_0_limb1, interaction_0_limb2, interaction_0_limb3],
        ) = acc
            .oods_samples
            .interaction
            .split_off(..N_INTERACTION_COLUMNS * SECURE_EXTENSION_DEGREE)
        else {
            panic!(
                "Expected {} interaction values",
                N_INTERACTION_COLUMNS * SECURE_EXTENSION_DEGREE
            );
        };
        let Some([claimed_sum]) = acc.claimed_sums.split_off(..1) else {
            panic!("Expected 1 claimed sum");
        };

        // Constraints.
        let constraint0_val = eval!(context, (in0_col0) - (in1_col4));
        acc.accumulate(context, constraint0_val);

        let constraint1_val = eval!(context, (in0_col1) - (in1_col5));
        acc.accumulate(context, constraint1_val);

        let constraint2_val = eval!(context, (in0_col2) - (in1_col6));
        acc.accumulate(context, constraint2_val);

        let constraint3_val = eval!(context, (in0_col3) - (in1_col7));
        acc.accumulate(context, constraint3_val);

        // Logup constraints.
        let prev_logup_sum = from_partial_evals(
            context,
            [
                interaction_0_limb0.at_prev,
                interaction_0_limb1.at_prev,
                interaction_0_limb2.at_prev,
                interaction_0_limb3.at_prev,
            ],
        );
        let cur_logup_sum = from_partial_evals(
            context,
            [
                interaction_0_limb0.at_oods,
                interaction_0_limb1.at_oods,
                interaction_0_limb2.at_oods,
                interaction_0_limb3.at_oods,
            ],
        );
        let n_instances = context.constant((1 << self.log_size).into());
        let cumsum_shift = div(context, *claimed_sum, n_instances);
        let diff = eval!(context, (cur_logup_sum) - (prev_logup_sum));
        let shifted_diff = eval!(context, (diff) + (cumsum_shift));

        let in0_gate_lookup_frac = get_frac(context, acc.interaction_elements, context.one(), &[in0_address, in0_col0, in0_col1, in0_col2, in0_col3]);
        let in1_gate_lookup_frac = get_frac(context, acc.interaction_elements, context.one(), &[in1_address, in1_col4, in1_col5, in1_col6, in1_col7]);

        let logup_constraint_val = pair_logup_term(
            context,
            in0_gate_lookup_frac,
            in1_gate_lookup_frac,
            shifted_diff,
        );
        acc.accumulate(context, logup_constraint_val);
    }
}
