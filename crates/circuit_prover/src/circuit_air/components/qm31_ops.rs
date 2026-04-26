use crate::circuit_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 12;
pub const N_INTERACTION_COLUMNS: usize = 8;

pub struct Eval {
    pub log_size: u32,
    pub common_lookup_elements: relations::CommonLookupElements,
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
        let m31_gate_relation_id = E::F::from(M31::from(378353459));
        let add_flag = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "qm31_ops_add_flag".to_owned() });
        let sub_flag = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "qm31_ops_sub_flag".to_owned() });
        let mul_flag = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "qm31_ops_mul_flag".to_owned() });
        let pointwise_mul_flag = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "qm31_ops_pointwise_mul_flag".to_owned(),
        });
        let op0_addr = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "qm31_ops_in0_address".to_owned(),
        });
        let op1_addr = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "qm31_ops_in1_address".to_owned(),
        });
        let dst_addr = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "qm31_ops_out_address".to_owned(),
        });
        let qm31_ops_multiplicity =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "qm31_ops_mults".to_owned() });

        let input_op0_limb0_col0 = eval.next_trace_mask();
        let input_op0_limb1_col1 = eval.next_trace_mask();
        let input_op0_limb2_col2 = eval.next_trace_mask();
        let input_op0_limb3_col3 = eval.next_trace_mask();
        let input_op1_limb0_col4 = eval.next_trace_mask();
        let input_op1_limb1_col5 = eval.next_trace_mask();
        let input_op1_limb2_col6 = eval.next_trace_mask();
        let input_op1_limb3_col7 = eval.next_trace_mask();
        let input_dst_limb0_col8 = eval.next_trace_mask();
        let input_dst_limb1_col9 = eval.next_trace_mask();
        let input_dst_limb2_col10 = eval.next_trace_mask();
        let input_dst_limb3_col11 = eval.next_trace_mask();

        // all flags sum to 1.
        eval.add_constraint(
            add_flag.clone() + sub_flag.clone() + mul_flag.clone() + pointwise_mul_flag.clone()
                - E::F::one(),
        );
        // each flag is a bit.
        eval.add_constraint(add_flag.clone() * (add_flag.clone() - E::F::one()));
        eval.add_constraint(sub_flag.clone() * (sub_flag.clone() - E::F::one()));
        eval.add_constraint(mul_flag.clone() * (mul_flag.clone() - E::F::one()));
        eval.add_constraint(
            pointwise_mul_flag.clone() * (pointwise_mul_flag.clone() - E::F::one()),
        );

        eval.add_constraint(
            input_dst_limb0_col8.clone()
                - (mul_flag.clone()
                    * (input_op0_limb0_col0.clone() * input_op1_limb0_col4.clone()
                        - input_op0_limb1_col1.clone() * input_op1_limb1_col5.clone()
                        + E::F::from(M31::from(2))
                            * (input_op0_limb2_col2.clone() * input_op1_limb2_col6.clone()
                                - input_op0_limb3_col3.clone() * input_op1_limb3_col7.clone())
                        - input_op0_limb2_col2.clone() * input_op1_limb3_col7.clone()
                        - input_op0_limb3_col3.clone() * input_op1_limb2_col6.clone())
                    + add_flag.clone()
                        * (input_op0_limb0_col0.clone() + input_op1_limb0_col4.clone())
                    + sub_flag.clone()
                        * (input_op0_limb0_col0.clone() - input_op1_limb0_col4.clone())
                    + pointwise_mul_flag.clone()
                        * (input_op0_limb0_col0.clone() * input_op1_limb0_col4.clone())),
        );

        eval.add_constraint(
            input_dst_limb1_col9.clone()
                - (mul_flag.clone()
                    * (input_op0_limb0_col0.clone() * input_op1_limb1_col5.clone()
                        + input_op0_limb1_col1.clone() * input_op1_limb0_col4.clone()
                        + E::F::from(M31::from(2))
                            * (input_op0_limb2_col2.clone() * input_op1_limb3_col7.clone()
                                + input_op0_limb3_col3.clone() * input_op1_limb2_col6.clone())
                        + input_op0_limb2_col2.clone() * input_op1_limb2_col6.clone()
                        - input_op0_limb3_col3.clone() * input_op1_limb3_col7.clone())
                    + add_flag.clone()
                        * (input_op0_limb1_col1.clone() + input_op1_limb1_col5.clone())
                    + sub_flag.clone()
                        * (input_op0_limb1_col1.clone() - input_op1_limb1_col5.clone())
                    + pointwise_mul_flag.clone()
                        * (input_op0_limb1_col1.clone() * input_op1_limb1_col5.clone())),
        );

        eval.add_constraint(
            input_dst_limb2_col10.clone()
                - (mul_flag.clone()
                    * (input_op0_limb0_col0.clone() * input_op1_limb2_col6.clone()
                        - input_op0_limb1_col1.clone() * input_op1_limb3_col7.clone()
                        + input_op0_limb2_col2.clone() * input_op1_limb0_col4.clone()
                        - input_op0_limb3_col3.clone() * input_op1_limb1_col5.clone())
                    + add_flag.clone()
                        * (input_op0_limb2_col2.clone() + input_op1_limb2_col6.clone())
                    + sub_flag.clone()
                        * (input_op0_limb2_col2.clone() - input_op1_limb2_col6.clone())
                    + pointwise_mul_flag.clone()
                        * (input_op0_limb2_col2.clone() * input_op1_limb2_col6.clone())),
        );

        eval.add_constraint(
            input_dst_limb3_col11.clone()
                - (mul_flag.clone()
                    * (input_op0_limb0_col0.clone() * input_op1_limb3_col7.clone()
                        + input_op0_limb1_col1.clone() * input_op1_limb2_col6.clone()
                        + input_op0_limb2_col2.clone() * input_op1_limb1_col5.clone()
                        + input_op0_limb3_col3.clone() * input_op1_limb0_col4.clone())
                    + add_flag.clone()
                        * (input_op0_limb3_col3.clone() + input_op1_limb3_col7.clone())
                    + sub_flag.clone()
                        * (input_op0_limb3_col3.clone() - input_op1_limb3_col7.clone())
                    + pointwise_mul_flag.clone()
                        * (input_op0_limb3_col3.clone() * input_op1_limb3_col7.clone())),
        );

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                m31_gate_relation_id.clone(),
                op0_addr.clone(),
                input_op0_limb0_col0.clone(),
                input_op0_limb1_col1.clone(),
                input_op0_limb2_col2.clone(),
                input_op0_limb3_col3.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                m31_gate_relation_id.clone(),
                op1_addr.clone(),
                input_op1_limb0_col4.clone(),
                input_op1_limb1_col5.clone(),
                input_op1_limb2_col6.clone(),
                input_op1_limb3_col7.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(qm31_ops_multiplicity),
            &[
                m31_gate_relation_id.clone(),
                dst_addr.clone(),
                input_dst_limb0_col8.clone(),
                input_dst_limb1_col9.clone(),
                input_dst_limb2_col10.clone(),
                input_dst_limb3_col11.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}
