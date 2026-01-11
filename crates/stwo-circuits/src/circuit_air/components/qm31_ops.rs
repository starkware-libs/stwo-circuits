use crate::circuit_air::components::prelude::*;

pub const N_PREPROCESSED_COLUMNS: usize = 8;
pub const N_TRACE_COLUMNS: usize = 12;

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

        let in0_col0 = eval.next_trace_mask();
        let in0_col1 = eval.next_trace_mask();
        let in0_col2 = eval.next_trace_mask();
        let in0_col3 = eval.next_trace_mask();
        let in1_col4 = eval.next_trace_mask();
        let in1_col5 = eval.next_trace_mask();
        let in1_col6 = eval.next_trace_mask();
        let in1_col7 = eval.next_trace_mask();
        let out_col8 = eval.next_trace_mask();
        let out_col9 = eval.next_trace_mask();
        let out_col10 = eval.next_trace_mask();
        let out_col11 = eval.next_trace_mask();

        // out col 8.
        eval.add_constraint(
            ((add_flag.clone()) * (in0_col0.clone() + in1_col4.clone())
                + (sub_flag.clone()) * (in0_col0.clone() - in1_col4.clone())
                + (pointwise_mul_flag.clone()) * (in0_col0.clone() * in1_col4.clone()))
                - out_col8.clone(),
        );

        // out col 9.
        eval.add_constraint(
            ((add_flag.clone()) * (in0_col1.clone() + in1_col5.clone())
                + (sub_flag.clone()) * (in0_col1.clone() - in1_col5.clone())
                + (pointwise_mul_flag.clone()) * (in0_col1.clone() * in1_col5.clone()))
                - out_col9.clone(),
        );

        // out col 10.
        eval.add_constraint(
            ((add_flag.clone()) * (in0_col2.clone() + in1_col6.clone())
                + (sub_flag.clone()) * (in0_col2.clone() - in1_col6.clone())
                + (pointwise_mul_flag.clone()) * (in0_col2.clone() * in1_col6.clone()))
                - out_col10.clone(),
        );

        // out col 11.
        eval.add_constraint(
            ((add_flag.clone()) * (in0_col3.clone() + in1_col7.clone())
                + (sub_flag.clone()) * (in0_col3.clone() - in1_col7.clone())
                + (pointwise_mul_flag.clone()) * (in0_col3.clone() * in1_col7.clone()))
                - out_col11.clone(),
        );

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

        eval.add_to_relation(RelationEntry::new(
            &self.gate_lookup_elements,
            -E::EF::from(mults),
            &[
                out_address.clone(),
                out_col8.clone(),
                out_col9.clone(),
                out_col10.clone(),
                out_col11.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}

pub struct CircuitQm31OpsComponent {
    pub preprocessed_column_indices: [usize; N_PREPROCESSED_COLUMNS],
}

impl CircuitEval for CircuitQm31OpsComponent {
    fn evaluate(
        &self,
        context: &mut Context<impl IValue>,
        acc: &mut CompositionConstraintAccumulator<'_>,
    ) {
        let [
            add_flag,
            sub_flag,
            _mul_flag,
            pointwise_mul_flag,
            in0_address,
            in1_address,
            out_address,
            mults,
        ] = acc
            .get_preprocessed_columns::<N_PREPROCESSED_COLUMNS>(self.preprocessed_column_indices);

        let [
            in0_col0,
            in0_col1,
            in0_col2,
            in0_col3,
            in1_col4,
            in1_col5,
            in1_col6,
            in1_col7,
            out_col8,
            out_col9,
            out_col10,
            out_col11,
        ] = acc.get_trace(N_TRACE_COLUMNS).try_into().unwrap();

        // out col 8.
        let constraint0_val = eval!(
            context,
            ((((add_flag) * ((in0_col0) + (in1_col4))) + ((sub_flag) * ((in0_col0) - (in1_col4))))
                + ((pointwise_mul_flag) * ((in0_col0) * (in1_col4))))
                - (out_col8)
        );
        acc.add_constraint(context, constraint0_val);

        // out col 9.
        let constraint1_val = eval!(
            context,
            ((((add_flag) * ((in0_col1) + (in1_col5))) + ((sub_flag) * ((in0_col1) - (in1_col5))))
                + ((pointwise_mul_flag) * ((in0_col1) * (in1_col5))))
                - (out_col9)
        );
        acc.add_constraint(context, constraint1_val);

        // out col 10.
        let constraint2_val = eval!(
            context,
            ((((add_flag) * ((in0_col2) + (in1_col6))) + ((sub_flag) * ((in0_col2) - (in1_col6))))
                + ((pointwise_mul_flag) * ((in0_col2) * (in1_col6))))
                - (out_col10)
        );
        acc.add_constraint(context, constraint2_val);

        // out col 11.
        let constraint3_val = eval!(
            context,
            ((((add_flag) * ((in0_col3) + (in1_col7))) + ((sub_flag) * ((in0_col3) - (in1_col7))))
                + ((pointwise_mul_flag) * ((in0_col3) * (in1_col7))))
                - (out_col11)
        );
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

        let neg_mults = eval!(context, (context.zero()) - (mults));
        acc.add_to_relation(
            context,
            neg_mults,
            &[out_address, out_col8, out_col9, out_col10, out_col11],
        );

        acc.finalize_logup_in_pairs(context);
    }
}
