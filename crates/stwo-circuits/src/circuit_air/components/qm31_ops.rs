use crate::circuit_air::components::prelude::*;

pub const N_PREPROCESSED_COLUMNS: usize = 8;
pub const N_TRACE_COLUMNS: usize = 12;
pub const N_INTERACTION_COLUMNS: usize = 2;

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
        let add_flag = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "qm31_ops_add_flag".to_owned() });
        let sub_flag = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "qm31_ops_sub_flag".to_owned() });
        // TODO(Gali): Add constraints for mul.
        let _mul_flag = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "qm31_ops_mul_flag".to_owned() });
        let pointwise_mul_flag = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "qm31_ops_pointwise_mul_flag".to_owned(),
        });
        let in0_address = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "qm31_ops_in0_address".to_owned(),
        });
        let in1_address = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "qm31_ops_in1_address".to_owned(),
        });
        let out_address = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "qm31_ops_out_address".to_owned(),
        });
        let mults =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "qm31_ops_mults".to_owned() });

        let in0_limb0_col0 = eval.next_trace_mask();
        let in0_limb1_col1 = eval.next_trace_mask();
        let in0_limb2_col2 = eval.next_trace_mask();
        let in0_limb3_col3 = eval.next_trace_mask();
        let in1_limb0_col4 = eval.next_trace_mask();
        let in1_limb1_col5 = eval.next_trace_mask();
        let in1_limb2_col6 = eval.next_trace_mask();
        let in1_limb3_col7 = eval.next_trace_mask();
        let out_limb0_col8 = eval.next_trace_mask();
        let out_limb1_col9 = eval.next_trace_mask();
        let out_limb2_col10 = eval.next_trace_mask();
        let out_limb3_col11 = eval.next_trace_mask();

        // out col 8.
        eval.add_constraint(
            ((add_flag.clone()) * (in0_limb0_col0.clone() + in1_limb0_col4.clone())
                + (sub_flag.clone()) * (in0_limb0_col0.clone() - in1_limb0_col4.clone())
                + (pointwise_mul_flag.clone()) * (in0_limb0_col0.clone() * in1_limb0_col4.clone()))
                - out_limb0_col8.clone(),
        );

        // out col 9.
        eval.add_constraint(
            ((add_flag.clone()) * (in0_limb1_col1.clone() + in1_limb1_col5.clone())
                + (sub_flag.clone()) * (in0_limb1_col1.clone() - in1_limb1_col5.clone())
                + (pointwise_mul_flag.clone()) * (in0_limb1_col1.clone() * in1_limb1_col5.clone()))
                - out_limb1_col9.clone(),
        );

        // out col 10.
        eval.add_constraint(
            ((add_flag.clone()) * (in0_limb2_col2.clone() + in1_limb2_col6.clone())
                + (sub_flag.clone()) * (in0_limb2_col2.clone() - in1_limb2_col6.clone())
                + (pointwise_mul_flag.clone()) * (in0_limb2_col2.clone() * in1_limb2_col6.clone()))
                - out_limb2_col10.clone(),
        );

        // out col 11.
        eval.add_constraint(
            ((add_flag.clone()) * (in0_limb3_col3.clone() + in1_limb3_col7.clone())
                + (sub_flag.clone()) * (in0_limb3_col3.clone() - in1_limb3_col7.clone())
                + (pointwise_mul_flag.clone()) * (in0_limb3_col3.clone() * in1_limb3_col7.clone()))
                - out_limb3_col11.clone(),
        );

        eval.add_to_relation(RelationEntry::new(
            &self.gate_lookup_elements,
            E::EF::one(),
            &[
                in0_address.clone(),
                in0_limb0_col0.clone(),
                in0_limb1_col1.clone(),
                in0_limb2_col2.clone(),
                in0_limb3_col3.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.gate_lookup_elements,
            E::EF::one(),
            &[
                in1_address.clone(),
                in1_limb0_col4.clone(),
                in1_limb1_col5.clone(),
                in1_limb2_col6.clone(),
                in1_limb3_col7.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.gate_lookup_elements,
            -E::EF::from(mults),
            &[
                out_address.clone(),
                out_limb0_col8.clone(),
                out_limb1_col9.clone(),
                out_limb2_col10.clone(),
                out_limb3_col11.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}

impl Statement for Eval {
    fn evaluate(
        &self,
        context: &mut Context<impl IValue>,
        acc: &mut CompositionConstraintAccumulator<'_>,
    ) {
        // Preprocessed columns.
        let mut preprocessed_idx_iter = self.preprocessed_column_indices.iter();
        let add_flag =
            acc.oods_samples.preprocessed_columns[*preprocessed_idx_iter.next().unwrap()];
        let sub_flag =
            acc.oods_samples.preprocessed_columns[*preprocessed_idx_iter.next().unwrap()];
            // TODO(Gali): Add constraints for mul.
        let _mul_flag =
            acc.oods_samples.preprocessed_columns[*preprocessed_idx_iter.next().unwrap()];
        let pointwise_mul_flag =
            acc.oods_samples.preprocessed_columns[*preprocessed_idx_iter.next().unwrap()];
        let in0_address =
            acc.oods_samples.preprocessed_columns[*preprocessed_idx_iter.next().unwrap()];
        let in1_address =
            acc.oods_samples.preprocessed_columns[*preprocessed_idx_iter.next().unwrap()];
        let out_address =
            acc.oods_samples.preprocessed_columns[*preprocessed_idx_iter.next().unwrap()];
        let mults = acc.oods_samples.preprocessed_columns[*preprocessed_idx_iter.next().unwrap()];
        let neg_mults = eval!(context, (context.zero()) - (mults));

        // Trace columns.
        let Some(
            &[in0_limb0_col0, in0_limb1_col1, in0_limb2_col2, in0_limb3_col3, in1_limb0_col4, in1_limb1_col5, in1_limb2_col6, in1_limb3_col7, out_limb0_col8, out_limb1_col9, out_limb2_col10, out_limb3_col11],
        ) = acc.oods_samples.trace.split_off(..N_TRACE_COLUMNS)
        else {
            panic!("Expected {} trace values", N_TRACE_COLUMNS);
        };

        // Interaction columns.
        let Some(
            [_interaction_0_limb0, _interaction_0_limb1, _interaction_0_limb2, _interaction_0_limb3, interaction_1_limb0, interaction_1_limb1, interaction_1_limb2, interaction_1_limb3],
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

        // Claimed sum.
        let Some([claimed_sum]) = acc.claimed_sums.split_off(..1) else {
            panic!("Expected 1 claimed sum");
        };

        // Constraints.
        let constraint0_val = eval!(
            context,
            ((((add_flag) * ((in0_limb0_col0) + (in1_limb0_col4)))
                + ((sub_flag) * ((in0_limb0_col0) - (in1_limb0_col4))))
                + ((pointwise_mul_flag) * ((in0_limb0_col0) * (in1_limb0_col4))))
                - (out_limb0_col8)
        );
        acc.accumulate(context, constraint0_val);

        let constraint1_val = eval!(
            context,
            ((((add_flag) * ((in0_limb1_col1) + (in1_limb1_col5)))
                + ((sub_flag) * ((in0_limb1_col1) - (in1_limb1_col5))))
                + ((pointwise_mul_flag) * ((in0_limb1_col1) * (in1_limb1_col5))))
                - (out_limb1_col9)
        );
        acc.accumulate(context, constraint1_val);

        let constraint2_val = eval!(
            context,
            ((((add_flag) * ((in0_limb2_col2) + (in1_limb2_col6)))
                + ((sub_flag) * ((in0_limb2_col2) - (in1_limb2_col6))))
                + ((pointwise_mul_flag) * ((in0_limb2_col2) * (in1_limb2_col6))))
                - (out_limb2_col10)
        );
        acc.accumulate(context, constraint2_val);

        let constraint3_val = eval!(
            context,
            ((((add_flag) * ((in0_limb3_col3) + (in1_limb3_col7)))
                + ((sub_flag) * ((in0_limb3_col3) - (in1_limb3_col7))))
                + ((pointwise_mul_flag) * ((in0_limb3_col3) * (in1_limb3_col7))))
                - (out_limb3_col11)
        );
        acc.accumulate(context, constraint3_val);

        // Last logup constraint.
        let _in0_gate_lookup_frac = get_frac(
            context,
            acc.interaction_elements,
            context.one(),
            &[in0_address, in0_limb0_col0, in0_limb1_col1, in0_limb2_col2, in0_limb3_col3],
        );
        let _in1_gate_lookup_frac = get_frac(
            context,
            acc.interaction_elements,
            context.one(),
            &[in1_address, in1_limb0_col4, in1_limb1_col5, in1_limb2_col6, in1_limb3_col7],
        );
        let out_gate_lookup_frac = get_frac(
            context,
            acc.interaction_elements,
            neg_mults,
            &[out_address, out_limb0_col8, out_limb1_col9, out_limb2_col10, out_limb3_col11],
        );

        let prev_logup_sum = from_partial_evals(
            context,
            [
                interaction_1_limb0.at_prev,
                interaction_1_limb1.at_prev,
                interaction_1_limb2.at_prev,
                interaction_1_limb3.at_prev,
            ],
        );
        let cur_logup_sum = from_partial_evals(
            context,
            [
                interaction_1_limb0.at_oods,
                interaction_1_limb1.at_oods,
                interaction_1_limb2.at_oods,
                interaction_1_limb3.at_oods,
            ],
        );
        let n_instances = context.constant((1 << self.log_size).into());
        let cumsum_shift = div(context, *claimed_sum, n_instances);
        let diff = eval!(context, (cur_logup_sum) - (prev_logup_sum));
        let shifted_diff = eval!(context, (diff) + (cumsum_shift));
        let logup_constraint_val = single_logup_term(context, out_gate_lookup_frac, shifted_diff);
        acc.accumulate(context, logup_constraint_val);
    }
}
