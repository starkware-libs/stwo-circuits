// This file was created by the AIR team.

use crate::circuit_air::{components::prelude::*, relations::GATE_RELATION_ID};

pub const N_TRACE_COLUMNS: usize = 24;
pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "BlakeOutput", uses: 1 }];

pub struct Eval {
    pub claim: Claim,
    pub common_lookup_elements: relations::CommonLookupElements,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub log_size: u32,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let trace_log_sizes = vec![self.log_size; N_TRACE_COLUMNS];
        let interaction_log_sizes = vec![self.log_size; SECURE_EXTENSION_DEGREE * 2];
        TreeVec::new(vec![vec![], trace_log_sizes, interaction_log_sizes])
    }

    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_u64(self.log_size as u64);
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct InteractionClaim {
    pub claimed_sum: SecureField,
}
impl InteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_felts(&[self.claimed_sum]);
    }
}

pub type Component = FrameworkComponent<Eval>;

impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        self.claim.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let M31_1061955672 = E::F::from(M31::from(1061955672));
        let M31_65536 = E::F::from(M31::from(65536));
        let final_state_addr = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "final_state_addr".to_owned() });
        let blake_output0_addr = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "blake_output0_addr".to_owned() });
        let blake_output1_addr = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "blake_output1_addr".to_owned() });
        let blake_output0_mults = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "blake_output0_mults".to_owned() });
        let blake_output1_mults = eval
            .get_preprocessed_column(PreProcessedColumnId { id: "blake_output1_mults".to_owned() });

        let input_final_state_limb0_limb_0_col0 = eval.next_trace_mask();
        let input_final_state_limb0_limb_1_col1 = eval.next_trace_mask();
        let input_final_state_limb1_limb_0_col2 = eval.next_trace_mask();
        let input_final_state_limb1_limb_1_col3 = eval.next_trace_mask();
        let input_final_state_limb2_limb_0_col4 = eval.next_trace_mask();
        let input_final_state_limb2_limb_1_col5 = eval.next_trace_mask();
        let input_final_state_limb3_limb_0_col6 = eval.next_trace_mask();
        let input_final_state_limb3_limb_1_col7 = eval.next_trace_mask();
        let input_final_state_limb4_limb_0_col8 = eval.next_trace_mask();
        let input_final_state_limb4_limb_1_col9 = eval.next_trace_mask();
        let input_final_state_limb5_limb_0_col10 = eval.next_trace_mask();
        let input_final_state_limb5_limb_1_col11 = eval.next_trace_mask();
        let input_final_state_limb6_limb_0_col12 = eval.next_trace_mask();
        let input_final_state_limb6_limb_1_col13 = eval.next_trace_mask();
        let input_final_state_limb7_limb_0_col14 = eval.next_trace_mask();
        let input_final_state_limb7_limb_1_col15 = eval.next_trace_mask();
        let output_limb0_col16 = eval.next_trace_mask();
        let output_limb1_col17 = eval.next_trace_mask();
        let output_limb2_col18 = eval.next_trace_mask();
        let output_limb3_col19 = eval.next_trace_mask();
        let output_limb4_col20 = eval.next_trace_mask();
        let output_limb5_col21 = eval.next_trace_mask();
        let output_limb6_col22 = eval.next_trace_mask();
        let output_limb7_col23 = eval.next_trace_mask();

        // output_limb0.
        eval.add_constraint(
            (output_limb0_col16.clone()
                - (input_final_state_limb0_limb_0_col0.clone()
                    + (input_final_state_limb0_limb_1_col1.clone() * M31_65536.clone()))),
        );
        // output_limb1.
        eval.add_constraint(
            (output_limb1_col17.clone()
                - (input_final_state_limb1_limb_0_col2.clone()
                    + (input_final_state_limb1_limb_1_col3.clone() * M31_65536.clone()))),
        );
        // output_limb2.
        eval.add_constraint(
            (output_limb2_col18.clone()
                - (input_final_state_limb2_limb_0_col4.clone()
                    + (input_final_state_limb2_limb_1_col5.clone() * M31_65536.clone()))),
        );
        // output_limb3.
        eval.add_constraint(
            (output_limb3_col19.clone()
                - (input_final_state_limb3_limb_0_col6.clone()
                    + (input_final_state_limb3_limb_1_col7.clone() * M31_65536.clone()))),
        );
        // output_limb4.
        eval.add_constraint(
            (output_limb4_col20.clone()
                - (input_final_state_limb4_limb_0_col8.clone()
                    + (input_final_state_limb4_limb_1_col9.clone() * M31_65536.clone()))),
        );
        // output_limb5.
        eval.add_constraint(
            (output_limb5_col21.clone()
                - (input_final_state_limb5_limb_0_col10.clone()
                    + (input_final_state_limb5_limb_1_col11.clone() * M31_65536.clone()))),
        );
        // output_limb6.
        eval.add_constraint(
            (output_limb6_col22.clone()
                - (input_final_state_limb6_limb_0_col12.clone()
                    + (input_final_state_limb6_limb_1_col13.clone() * M31_65536.clone()))),
        );
        // output_limb7.
        eval.add_constraint(
            (output_limb7_col23.clone()
                - (input_final_state_limb7_limb_0_col14.clone()
                    + (input_final_state_limb7_limb_1_col15.clone() * M31_65536.clone()))),
        );

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                M31_1061955672.clone(),
                final_state_addr.clone(),
                input_final_state_limb0_limb_0_col0.clone(),
                input_final_state_limb0_limb_1_col1.clone(),
                input_final_state_limb1_limb_0_col2.clone(),
                input_final_state_limb1_limb_1_col3.clone(),
                input_final_state_limb2_limb_0_col4.clone(),
                input_final_state_limb2_limb_1_col5.clone(),
                input_final_state_limb3_limb_0_col6.clone(),
                input_final_state_limb3_limb_1_col7.clone(),
                input_final_state_limb4_limb_0_col8.clone(),
                input_final_state_limb4_limb_1_col9.clone(),
                input_final_state_limb5_limb_0_col10.clone(),
                input_final_state_limb5_limb_1_col11.clone(),
                input_final_state_limb6_limb_0_col12.clone(),
                input_final_state_limb6_limb_1_col13.clone(),
                input_final_state_limb7_limb_0_col14.clone(),
                input_final_state_limb7_limb_1_col15.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(blake_output0_mults),
            &[
                E::F::from(GATE_RELATION_ID),
                blake_output0_addr.clone(),
                output_limb0_col16.clone(),
                output_limb1_col17.clone(),
                output_limb2_col18.clone(),
                output_limb3_col19.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(blake_output1_mults),
            &[
                E::F::from(GATE_RELATION_ID),
                blake_output1_addr.clone(),
                output_limb4_col20.clone(),
                output_limb5_col21.clone(),
                output_limb6_col22.clone(),
                output_limb7_col23.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}
