use crate::components::prelude::*;
use circuits_stark_verifier::constraint_eval::RelationUse;

pub const N_TRACE_COLUMNS: usize = 12;
pub const N_INTERACTION_COLUMNS: usize = 8;
use circuits_stark_verifier::constraint_eval::ComponentDataTrait;

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

pub struct CircuitQm31OpsComponent {}

impl<Value: IValue> CircuitEval<Value> for CircuitQm31OpsComponent {
    fn name(&self) -> String {
        "qm31_ops".to_string()
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        let m31_gate_relation_id = context.constant(SecureField::from(M31::from(378353459)));
        let add_flag = acc
            .get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_add_flag".to_owned() });
        let sub_flag = acc
            .get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_sub_flag".to_owned() });
        let mul_flag = acc
            .get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_mul_flag".to_owned() });
        let pointwise_mul_flag = acc.get_preprocessed_column(&PreProcessedColumnId {
            id: "qm31_ops_pointwise_mul_flag".to_owned(),
        });
        let op0_addr = acc.get_preprocessed_column(&PreProcessedColumnId {
            id: "qm31_ops_in0_address".to_owned(),
        });
        let op1_addr = acc.get_preprocessed_column(&PreProcessedColumnId {
            id: "qm31_ops_in1_address".to_owned(),
        });
        let dst_addr = acc.get_preprocessed_column(&PreProcessedColumnId {
            id: "qm31_ops_out_address".to_owned(),
        });
        let qm31_ops_multiplicity =
            acc.get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_mults".to_owned() });

        let [
            input_op0_limb0_col0,
            input_op0_limb1_col1,
            input_op0_limb2_col2,
            input_op0_limb3_col3,
            input_op1_limb0_col4,
            input_op1_limb1_col5,
            input_op1_limb2_col6,
            input_op1_limb3_col7,
            input_dst_limb0_col8,
            input_dst_limb1_col9,
            input_dst_limb2_col10,
            input_dst_limb3_col11,
        ] = *component_data.trace_columns()
        else {
            panic!("Expected {N_TRACE_COLUMNS} trace columns")
        };

        // all flags sum to 1.
        let constraint0_val =
            eval!(context, ((((add_flag) + (sub_flag)) + (mul_flag)) + (pointwise_mul_flag)) - (1));
        acc.add_constraint(context, constraint0_val);
        // each flag is a bit.
        let constraint1_val = eval!(context, (add_flag) * ((add_flag) - (1)));
        acc.add_constraint(context, constraint1_val);
        let constraint2_val = eval!(context, (sub_flag) * ((sub_flag) - (1)));
        acc.add_constraint(context, constraint2_val);
        let constraint3_val = eval!(context, (mul_flag) * ((mul_flag) - (1)));
        acc.add_constraint(context, constraint3_val);
        let constraint4_val = eval!(context, (pointwise_mul_flag) * ((pointwise_mul_flag) - (1)));
        acc.add_constraint(context, constraint4_val);

        // out col 8.
        let constraint5_val = eval!(
            context,
            (input_dst_limb0_col8)
                - ((((((((((input_op0_limb0_col0) * (input_op1_limb0_col4))
                    - ((input_op0_limb1_col1) * (input_op1_limb1_col5)))
                    + ((2)
                        * (((input_op0_limb2_col2) * (input_op1_limb2_col6))
                            - ((input_op0_limb3_col3) * (input_op1_limb3_col7)))))
                    - ((input_op0_limb2_col2) * (input_op1_limb3_col7)))
                    - ((input_op0_limb3_col3) * (input_op1_limb2_col6)))
                    * (mul_flag))
                    + (((input_op0_limb0_col0) + (input_op1_limb0_col4)) * (add_flag)))
                    + (((input_op0_limb0_col0) - (input_op1_limb0_col4)) * (sub_flag)))
                    + (((input_op0_limb0_col0) * (input_op1_limb0_col4)) * (pointwise_mul_flag)))
        );
        acc.add_constraint(context, constraint5_val);

        // out col 9.
        let constraint6_val = eval!(
            context,
            (input_dst_limb1_col9)
                - ((((((((((input_op0_limb0_col0) * (input_op1_limb1_col5))
                    + ((input_op0_limb1_col1) * (input_op1_limb0_col4)))
                    + ((2)
                        * (((input_op0_limb2_col2) * (input_op1_limb3_col7))
                            + ((input_op0_limb3_col3) * (input_op1_limb2_col6)))))
                    + ((input_op0_limb2_col2) * (input_op1_limb2_col6)))
                    - ((input_op0_limb3_col3) * (input_op1_limb3_col7)))
                    * (mul_flag))
                    + (((input_op0_limb1_col1) + (input_op1_limb1_col5)) * (add_flag)))
                    + (((input_op0_limb1_col1) - (input_op1_limb1_col5)) * (sub_flag)))
                    + (((input_op0_limb1_col1) * (input_op1_limb1_col5)) * (pointwise_mul_flag)))
        );
        acc.add_constraint(context, constraint6_val);

        // out col 10.
        let constraint7_val = eval!(
            context,
            (input_dst_limb2_col10)
                - (((((((((input_op0_limb0_col0) * (input_op1_limb2_col6))
                    - ((input_op0_limb1_col1) * (input_op1_limb3_col7)))
                    + ((input_op0_limb2_col2) * (input_op1_limb0_col4)))
                    - ((input_op0_limb3_col3) * (input_op1_limb1_col5)))
                    * (mul_flag))
                    + (((input_op0_limb2_col2) + (input_op1_limb2_col6)) * (add_flag)))
                    + (((input_op0_limb2_col2) - (input_op1_limb2_col6)) * (sub_flag)))
                    + (((input_op0_limb2_col2) * (input_op1_limb2_col6)) * (pointwise_mul_flag)))
        );
        acc.add_constraint(context, constraint7_val);

        // out col 11.
        let constraint8_val = eval!(
            context,
            (input_dst_limb3_col11)
                - (((((((((input_op0_limb0_col0) * (input_op1_limb3_col7))
                    + ((input_op0_limb1_col1) * (input_op1_limb2_col6)))
                    + ((input_op0_limb2_col2) * (input_op1_limb1_col5)))
                    + ((input_op0_limb3_col3) * (input_op1_limb0_col4)))
                    * (mul_flag))
                    + (((input_op0_limb3_col3) + (input_op1_limb3_col7)) * (add_flag)))
                    + (((input_op0_limb3_col3) - (input_op1_limb3_col7)) * (sub_flag)))
                    + (((input_op0_limb3_col3) * (input_op1_limb3_col7)) * (pointwise_mul_flag)))
        );
        acc.add_constraint(context, constraint8_val);

        acc.add_to_relation(
            context,
            context.one(),
            &[
                m31_gate_relation_id,
                op0_addr,
                input_op0_limb0_col0,
                input_op0_limb1_col1,
                input_op0_limb2_col2,
                input_op0_limb3_col3,
            ],
        );

        acc.add_to_relation(
            context,
            context.one(),
            &[
                m31_gate_relation_id,
                op1_addr,
                input_op1_limb0_col4,
                input_op1_limb1_col5,
                input_op1_limb2_col6,
                input_op1_limb3_col7,
            ],
        );

        let neg_mults = eval!(context, (context.zero()) - (qm31_ops_multiplicity));
        acc.add_to_relation(
            context,
            neg_mults,
            &[
                m31_gate_relation_id,
                dst_addr,
                input_dst_limb0_col8,
                input_dst_limb1_col9,
                input_dst_limb2_col10,
                input_dst_limb3_col11,
            ],
        );
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &[RelationUse { relation_id: "gate", uses: 2 }]
    }
}
