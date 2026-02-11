// This file was created by the AIR team.

use crate::circuit_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 148;
pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "BlakeG", uses: 8 },
    RelationUse { relation_id: "BlakeMessage", uses: 16 },
    RelationUse { relation_id: "BlakeRound", uses: 1 },
    RelationUse { relation_id: "BlakeRoundSigma", uses: 1 },
];

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
        let interaction_log_sizes = vec![self.log_size; SECURE_EXTENSION_DEGREE * 14];
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
        let M31_1 = E::F::from(M31::from(1));
        let M31_1139985212 = E::F::from(M31::from(1139985212));
        let M31_1492981981 = E::F::from(M31::from(1492981981));
        let M31_1805967942 = E::F::from(M31::from(1805967942));
        let M31_40528774 = E::F::from(M31::from(40528774));
        let input_limb_0_col0 = eval.next_trace_mask();
        let input_limb_1_col1 = eval.next_trace_mask();
        let input_limb_2_col2 = eval.next_trace_mask();
        let input_limb_3_col3 = eval.next_trace_mask();
        let input_limb_4_col4 = eval.next_trace_mask();
        let input_limb_5_col5 = eval.next_trace_mask();
        let input_limb_6_col6 = eval.next_trace_mask();
        let input_limb_7_col7 = eval.next_trace_mask();
        let input_limb_8_col8 = eval.next_trace_mask();
        let input_limb_9_col9 = eval.next_trace_mask();
        let input_limb_10_col10 = eval.next_trace_mask();
        let input_limb_11_col11 = eval.next_trace_mask();
        let input_limb_12_col12 = eval.next_trace_mask();
        let input_limb_13_col13 = eval.next_trace_mask();
        let input_limb_14_col14 = eval.next_trace_mask();
        let input_limb_15_col15 = eval.next_trace_mask();
        let input_limb_16_col16 = eval.next_trace_mask();
        let input_limb_17_col17 = eval.next_trace_mask();
        let input_limb_18_col18 = eval.next_trace_mask();
        let input_limb_19_col19 = eval.next_trace_mask();
        let input_limb_20_col20 = eval.next_trace_mask();
        let input_limb_21_col21 = eval.next_trace_mask();
        let input_limb_22_col22 = eval.next_trace_mask();
        let input_limb_23_col23 = eval.next_trace_mask();
        let input_limb_24_col24 = eval.next_trace_mask();
        let input_limb_25_col25 = eval.next_trace_mask();
        let input_limb_26_col26 = eval.next_trace_mask();
        let input_limb_27_col27 = eval.next_trace_mask();
        let input_limb_28_col28 = eval.next_trace_mask();
        let input_limb_29_col29 = eval.next_trace_mask();
        let input_limb_30_col30 = eval.next_trace_mask();
        let input_limb_31_col31 = eval.next_trace_mask();
        let input_limb_32_col32 = eval.next_trace_mask();
        let input_limb_33_col33 = eval.next_trace_mask();
        let input_limb_34_col34 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_0_col35 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_1_col36 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_2_col37 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_3_col38 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_4_col39 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_5_col40 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_6_col41 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_7_col42 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_8_col43 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_9_col44 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_10_col45 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_11_col46 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_12_col47 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_13_col48 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_14_col49 = eval.next_trace_mask();
        let blake_round_sigma_output_limb_15_col50 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col51 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col52 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col53 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col54 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col55 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col56 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col57 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col58 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col59 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col60 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col61 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col62 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col63 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col64 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col65 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col66 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col67 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col68 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col69 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col70 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col71 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col72 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col73 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col74 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col75 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col76 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col77 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col78 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col79 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col80 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_0_col81 = eval.next_trace_mask();
        let blake_message_output_message_limb_limb_1_col82 = eval.next_trace_mask();
        let blake_g_output_limb_0_col83 = eval.next_trace_mask();
        let blake_g_output_limb_1_col84 = eval.next_trace_mask();
        let blake_g_output_limb_2_col85 = eval.next_trace_mask();
        let blake_g_output_limb_3_col86 = eval.next_trace_mask();
        let blake_g_output_limb_4_col87 = eval.next_trace_mask();
        let blake_g_output_limb_5_col88 = eval.next_trace_mask();
        let blake_g_output_limb_6_col89 = eval.next_trace_mask();
        let blake_g_output_limb_7_col90 = eval.next_trace_mask();
        let blake_g_output_limb_0_col91 = eval.next_trace_mask();
        let blake_g_output_limb_1_col92 = eval.next_trace_mask();
        let blake_g_output_limb_2_col93 = eval.next_trace_mask();
        let blake_g_output_limb_3_col94 = eval.next_trace_mask();
        let blake_g_output_limb_4_col95 = eval.next_trace_mask();
        let blake_g_output_limb_5_col96 = eval.next_trace_mask();
        let blake_g_output_limb_6_col97 = eval.next_trace_mask();
        let blake_g_output_limb_7_col98 = eval.next_trace_mask();
        let blake_g_output_limb_0_col99 = eval.next_trace_mask();
        let blake_g_output_limb_1_col100 = eval.next_trace_mask();
        let blake_g_output_limb_2_col101 = eval.next_trace_mask();
        let blake_g_output_limb_3_col102 = eval.next_trace_mask();
        let blake_g_output_limb_4_col103 = eval.next_trace_mask();
        let blake_g_output_limb_5_col104 = eval.next_trace_mask();
        let blake_g_output_limb_6_col105 = eval.next_trace_mask();
        let blake_g_output_limb_7_col106 = eval.next_trace_mask();
        let blake_g_output_limb_0_col107 = eval.next_trace_mask();
        let blake_g_output_limb_1_col108 = eval.next_trace_mask();
        let blake_g_output_limb_2_col109 = eval.next_trace_mask();
        let blake_g_output_limb_3_col110 = eval.next_trace_mask();
        let blake_g_output_limb_4_col111 = eval.next_trace_mask();
        let blake_g_output_limb_5_col112 = eval.next_trace_mask();
        let blake_g_output_limb_6_col113 = eval.next_trace_mask();
        let blake_g_output_limb_7_col114 = eval.next_trace_mask();
        let blake_g_output_limb_0_col115 = eval.next_trace_mask();
        let blake_g_output_limb_1_col116 = eval.next_trace_mask();
        let blake_g_output_limb_2_col117 = eval.next_trace_mask();
        let blake_g_output_limb_3_col118 = eval.next_trace_mask();
        let blake_g_output_limb_4_col119 = eval.next_trace_mask();
        let blake_g_output_limb_5_col120 = eval.next_trace_mask();
        let blake_g_output_limb_6_col121 = eval.next_trace_mask();
        let blake_g_output_limb_7_col122 = eval.next_trace_mask();
        let blake_g_output_limb_0_col123 = eval.next_trace_mask();
        let blake_g_output_limb_1_col124 = eval.next_trace_mask();
        let blake_g_output_limb_2_col125 = eval.next_trace_mask();
        let blake_g_output_limb_3_col126 = eval.next_trace_mask();
        let blake_g_output_limb_4_col127 = eval.next_trace_mask();
        let blake_g_output_limb_5_col128 = eval.next_trace_mask();
        let blake_g_output_limb_6_col129 = eval.next_trace_mask();
        let blake_g_output_limb_7_col130 = eval.next_trace_mask();
        let blake_g_output_limb_0_col131 = eval.next_trace_mask();
        let blake_g_output_limb_1_col132 = eval.next_trace_mask();
        let blake_g_output_limb_2_col133 = eval.next_trace_mask();
        let blake_g_output_limb_3_col134 = eval.next_trace_mask();
        let blake_g_output_limb_4_col135 = eval.next_trace_mask();
        let blake_g_output_limb_5_col136 = eval.next_trace_mask();
        let blake_g_output_limb_6_col137 = eval.next_trace_mask();
        let blake_g_output_limb_7_col138 = eval.next_trace_mask();
        let blake_g_output_limb_0_col139 = eval.next_trace_mask();
        let blake_g_output_limb_1_col140 = eval.next_trace_mask();
        let blake_g_output_limb_2_col141 = eval.next_trace_mask();
        let blake_g_output_limb_3_col142 = eval.next_trace_mask();
        let blake_g_output_limb_4_col143 = eval.next_trace_mask();
        let blake_g_output_limb_5_col144 = eval.next_trace_mask();
        let blake_g_output_limb_6_col145 = eval.next_trace_mask();
        let blake_g_output_limb_7_col146 = eval.next_trace_mask();
        let enabler = eval.next_trace_mask();

        eval.add_constraint(enabler.clone() * enabler.clone() - enabler.clone());

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                M31_1805967942.clone(),
                input_limb_1_col1.clone(),
                blake_round_sigma_output_limb_0_col35.clone(),
                blake_round_sigma_output_limb_1_col36.clone(),
                blake_round_sigma_output_limb_2_col37.clone(),
                blake_round_sigma_output_limb_3_col38.clone(),
                blake_round_sigma_output_limb_4_col39.clone(),
                blake_round_sigma_output_limb_5_col40.clone(),
                blake_round_sigma_output_limb_6_col41.clone(),
                blake_round_sigma_output_limb_7_col42.clone(),
                blake_round_sigma_output_limb_8_col43.clone(),
                blake_round_sigma_output_limb_9_col44.clone(),
                blake_round_sigma_output_limb_10_col45.clone(),
                blake_round_sigma_output_limb_11_col46.clone(),
                blake_round_sigma_output_limb_12_col47.clone(),
                blake_round_sigma_output_limb_13_col48.clone(),
                blake_round_sigma_output_limb_14_col49.clone(),
                blake_round_sigma_output_limb_15_col50.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_0_col35.clone(),
                blake_message_output_message_limb_limb_0_col51.clone(),
                blake_message_output_message_limb_limb_1_col52.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_1_col36.clone(),
                blake_message_output_message_limb_limb_0_col53.clone(),
                blake_message_output_message_limb_limb_1_col54.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_2_col37.clone(),
                blake_message_output_message_limb_limb_0_col55.clone(),
                blake_message_output_message_limb_limb_1_col56.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_3_col38.clone(),
                blake_message_output_message_limb_limb_0_col57.clone(),
                blake_message_output_message_limb_limb_1_col58.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_4_col39.clone(),
                blake_message_output_message_limb_limb_0_col59.clone(),
                blake_message_output_message_limb_limb_1_col60.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_5_col40.clone(),
                blake_message_output_message_limb_limb_0_col61.clone(),
                blake_message_output_message_limb_limb_1_col62.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_6_col41.clone(),
                blake_message_output_message_limb_limb_0_col63.clone(),
                blake_message_output_message_limb_limb_1_col64.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_7_col42.clone(),
                blake_message_output_message_limb_limb_0_col65.clone(),
                blake_message_output_message_limb_limb_1_col66.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_8_col43.clone(),
                blake_message_output_message_limb_limb_0_col67.clone(),
                blake_message_output_message_limb_limb_1_col68.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_9_col44.clone(),
                blake_message_output_message_limb_limb_0_col69.clone(),
                blake_message_output_message_limb_limb_1_col70.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_10_col45.clone(),
                blake_message_output_message_limb_limb_0_col71.clone(),
                blake_message_output_message_limb_limb_1_col72.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_11_col46.clone(),
                blake_message_output_message_limb_limb_0_col73.clone(),
                blake_message_output_message_limb_limb_1_col74.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_12_col47.clone(),
                blake_message_output_message_limb_limb_0_col75.clone(),
                blake_message_output_message_limb_limb_1_col76.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_13_col48.clone(),
                blake_message_output_message_limb_limb_0_col77.clone(),
                blake_message_output_message_limb_limb_1_col78.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_14_col49.clone(),
                blake_message_output_message_limb_limb_0_col79.clone(),
                blake_message_output_message_limb_limb_1_col80.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_1492981981.clone(),
                input_limb_34_col34.clone(),
                blake_round_sigma_output_limb_15_col50.clone(),
                blake_message_output_message_limb_limb_0_col81.clone(),
                blake_message_output_message_limb_limb_1_col82.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                M31_1139985212.clone(),
                input_limb_2_col2.clone(),
                input_limb_3_col3.clone(),
                input_limb_10_col10.clone(),
                input_limb_11_col11.clone(),
                input_limb_18_col18.clone(),
                input_limb_19_col19.clone(),
                input_limb_26_col26.clone(),
                input_limb_27_col27.clone(),
                blake_message_output_message_limb_limb_0_col51.clone(),
                blake_message_output_message_limb_limb_1_col52.clone(),
                blake_message_output_message_limb_limb_0_col53.clone(),
                blake_message_output_message_limb_limb_1_col54.clone(),
                blake_g_output_limb_0_col83.clone(),
                blake_g_output_limb_1_col84.clone(),
                blake_g_output_limb_2_col85.clone(),
                blake_g_output_limb_3_col86.clone(),
                blake_g_output_limb_4_col87.clone(),
                blake_g_output_limb_5_col88.clone(),
                blake_g_output_limb_6_col89.clone(),
                blake_g_output_limb_7_col90.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                M31_1139985212.clone(),
                input_limb_4_col4.clone(),
                input_limb_5_col5.clone(),
                input_limb_12_col12.clone(),
                input_limb_13_col13.clone(),
                input_limb_20_col20.clone(),
                input_limb_21_col21.clone(),
                input_limb_28_col28.clone(),
                input_limb_29_col29.clone(),
                blake_message_output_message_limb_limb_0_col55.clone(),
                blake_message_output_message_limb_limb_1_col56.clone(),
                blake_message_output_message_limb_limb_0_col57.clone(),
                blake_message_output_message_limb_limb_1_col58.clone(),
                blake_g_output_limb_0_col91.clone(),
                blake_g_output_limb_1_col92.clone(),
                blake_g_output_limb_2_col93.clone(),
                blake_g_output_limb_3_col94.clone(),
                blake_g_output_limb_4_col95.clone(),
                blake_g_output_limb_5_col96.clone(),
                blake_g_output_limb_6_col97.clone(),
                blake_g_output_limb_7_col98.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                M31_1139985212.clone(),
                input_limb_6_col6.clone(),
                input_limb_7_col7.clone(),
                input_limb_14_col14.clone(),
                input_limb_15_col15.clone(),
                input_limb_22_col22.clone(),
                input_limb_23_col23.clone(),
                input_limb_30_col30.clone(),
                input_limb_31_col31.clone(),
                blake_message_output_message_limb_limb_0_col59.clone(),
                blake_message_output_message_limb_limb_1_col60.clone(),
                blake_message_output_message_limb_limb_0_col61.clone(),
                blake_message_output_message_limb_limb_1_col62.clone(),
                blake_g_output_limb_0_col99.clone(),
                blake_g_output_limb_1_col100.clone(),
                blake_g_output_limb_2_col101.clone(),
                blake_g_output_limb_3_col102.clone(),
                blake_g_output_limb_4_col103.clone(),
                blake_g_output_limb_5_col104.clone(),
                blake_g_output_limb_6_col105.clone(),
                blake_g_output_limb_7_col106.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                M31_1139985212.clone(),
                input_limb_8_col8.clone(),
                input_limb_9_col9.clone(),
                input_limb_16_col16.clone(),
                input_limb_17_col17.clone(),
                input_limb_24_col24.clone(),
                input_limb_25_col25.clone(),
                input_limb_32_col32.clone(),
                input_limb_33_col33.clone(),
                blake_message_output_message_limb_limb_0_col63.clone(),
                blake_message_output_message_limb_limb_1_col64.clone(),
                blake_message_output_message_limb_limb_0_col65.clone(),
                blake_message_output_message_limb_limb_1_col66.clone(),
                blake_g_output_limb_0_col107.clone(),
                blake_g_output_limb_1_col108.clone(),
                blake_g_output_limb_2_col109.clone(),
                blake_g_output_limb_3_col110.clone(),
                blake_g_output_limb_4_col111.clone(),
                blake_g_output_limb_5_col112.clone(),
                blake_g_output_limb_6_col113.clone(),
                blake_g_output_limb_7_col114.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                M31_1139985212.clone(),
                blake_g_output_limb_0_col83.clone(),
                blake_g_output_limb_1_col84.clone(),
                blake_g_output_limb_2_col93.clone(),
                blake_g_output_limb_3_col94.clone(),
                blake_g_output_limb_4_col103.clone(),
                blake_g_output_limb_5_col104.clone(),
                blake_g_output_limb_6_col113.clone(),
                blake_g_output_limb_7_col114.clone(),
                blake_message_output_message_limb_limb_0_col67.clone(),
                blake_message_output_message_limb_limb_1_col68.clone(),
                blake_message_output_message_limb_limb_0_col69.clone(),
                blake_message_output_message_limb_limb_1_col70.clone(),
                blake_g_output_limb_0_col115.clone(),
                blake_g_output_limb_1_col116.clone(),
                blake_g_output_limb_2_col117.clone(),
                blake_g_output_limb_3_col118.clone(),
                blake_g_output_limb_4_col119.clone(),
                blake_g_output_limb_5_col120.clone(),
                blake_g_output_limb_6_col121.clone(),
                blake_g_output_limb_7_col122.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                M31_1139985212.clone(),
                blake_g_output_limb_0_col91.clone(),
                blake_g_output_limb_1_col92.clone(),
                blake_g_output_limb_2_col101.clone(),
                blake_g_output_limb_3_col102.clone(),
                blake_g_output_limb_4_col111.clone(),
                blake_g_output_limb_5_col112.clone(),
                blake_g_output_limb_6_col89.clone(),
                blake_g_output_limb_7_col90.clone(),
                blake_message_output_message_limb_limb_0_col71.clone(),
                blake_message_output_message_limb_limb_1_col72.clone(),
                blake_message_output_message_limb_limb_0_col73.clone(),
                blake_message_output_message_limb_limb_1_col74.clone(),
                blake_g_output_limb_0_col123.clone(),
                blake_g_output_limb_1_col124.clone(),
                blake_g_output_limb_2_col125.clone(),
                blake_g_output_limb_3_col126.clone(),
                blake_g_output_limb_4_col127.clone(),
                blake_g_output_limb_5_col128.clone(),
                blake_g_output_limb_6_col129.clone(),
                blake_g_output_limb_7_col130.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                M31_1139985212.clone(),
                blake_g_output_limb_0_col99.clone(),
                blake_g_output_limb_1_col100.clone(),
                blake_g_output_limb_2_col109.clone(),
                blake_g_output_limb_3_col110.clone(),
                blake_g_output_limb_4_col87.clone(),
                blake_g_output_limb_5_col88.clone(),
                blake_g_output_limb_6_col97.clone(),
                blake_g_output_limb_7_col98.clone(),
                blake_message_output_message_limb_limb_0_col75.clone(),
                blake_message_output_message_limb_limb_1_col76.clone(),
                blake_message_output_message_limb_limb_0_col77.clone(),
                blake_message_output_message_limb_limb_1_col78.clone(),
                blake_g_output_limb_0_col131.clone(),
                blake_g_output_limb_1_col132.clone(),
                blake_g_output_limb_2_col133.clone(),
                blake_g_output_limb_3_col134.clone(),
                blake_g_output_limb_4_col135.clone(),
                blake_g_output_limb_5_col136.clone(),
                blake_g_output_limb_6_col137.clone(),
                blake_g_output_limb_7_col138.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                M31_1139985212.clone(),
                blake_g_output_limb_0_col107.clone(),
                blake_g_output_limb_1_col108.clone(),
                blake_g_output_limb_2_col85.clone(),
                blake_g_output_limb_3_col86.clone(),
                blake_g_output_limb_4_col95.clone(),
                blake_g_output_limb_5_col96.clone(),
                blake_g_output_limb_6_col105.clone(),
                blake_g_output_limb_7_col106.clone(),
                blake_message_output_message_limb_limb_0_col79.clone(),
                blake_message_output_message_limb_limb_1_col80.clone(),
                blake_message_output_message_limb_limb_0_col81.clone(),
                blake_message_output_message_limb_limb_1_col82.clone(),
                blake_g_output_limb_0_col139.clone(),
                blake_g_output_limb_1_col140.clone(),
                blake_g_output_limb_2_col141.clone(),
                blake_g_output_limb_3_col142.clone(),
                blake_g_output_limb_4_col143.clone(),
                blake_g_output_limb_5_col144.clone(),
                blake_g_output_limb_6_col145.clone(),
                blake_g_output_limb_7_col146.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_40528774.clone(),
                input_limb_0_col0.clone(),
                input_limb_1_col1.clone(),
                input_limb_2_col2.clone(),
                input_limb_3_col3.clone(),
                input_limb_4_col4.clone(),
                input_limb_5_col5.clone(),
                input_limb_6_col6.clone(),
                input_limb_7_col7.clone(),
                input_limb_8_col8.clone(),
                input_limb_9_col9.clone(),
                input_limb_10_col10.clone(),
                input_limb_11_col11.clone(),
                input_limb_12_col12.clone(),
                input_limb_13_col13.clone(),
                input_limb_14_col14.clone(),
                input_limb_15_col15.clone(),
                input_limb_16_col16.clone(),
                input_limb_17_col17.clone(),
                input_limb_18_col18.clone(),
                input_limb_19_col19.clone(),
                input_limb_20_col20.clone(),
                input_limb_21_col21.clone(),
                input_limb_22_col22.clone(),
                input_limb_23_col23.clone(),
                input_limb_24_col24.clone(),
                input_limb_25_col25.clone(),
                input_limb_26_col26.clone(),
                input_limb_27_col27.clone(),
                input_limb_28_col28.clone(),
                input_limb_29_col29.clone(),
                input_limb_30_col30.clone(),
                input_limb_31_col31.clone(),
                input_limb_32_col32.clone(),
                input_limb_33_col33.clone(),
                input_limb_34_col34.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(enabler.clone()),
            &[
                M31_40528774.clone(),
                input_limb_0_col0.clone(),
                (input_limb_1_col1.clone() + M31_1.clone()),
                blake_g_output_limb_0_col115.clone(),
                blake_g_output_limb_1_col116.clone(),
                blake_g_output_limb_0_col123.clone(),
                blake_g_output_limb_1_col124.clone(),
                blake_g_output_limb_0_col131.clone(),
                blake_g_output_limb_1_col132.clone(),
                blake_g_output_limb_0_col139.clone(),
                blake_g_output_limb_1_col140.clone(),
                blake_g_output_limb_2_col141.clone(),
                blake_g_output_limb_3_col142.clone(),
                blake_g_output_limb_2_col117.clone(),
                blake_g_output_limb_3_col118.clone(),
                blake_g_output_limb_2_col125.clone(),
                blake_g_output_limb_3_col126.clone(),
                blake_g_output_limb_2_col133.clone(),
                blake_g_output_limb_3_col134.clone(),
                blake_g_output_limb_4_col135.clone(),
                blake_g_output_limb_5_col136.clone(),
                blake_g_output_limb_4_col143.clone(),
                blake_g_output_limb_5_col144.clone(),
                blake_g_output_limb_4_col119.clone(),
                blake_g_output_limb_5_col120.clone(),
                blake_g_output_limb_4_col127.clone(),
                blake_g_output_limb_5_col128.clone(),
                blake_g_output_limb_6_col129.clone(),
                blake_g_output_limb_7_col130.clone(),
                blake_g_output_limb_6_col137.clone(),
                blake_g_output_limb_7_col138.clone(),
                blake_g_output_limb_6_col145.clone(),
                blake_g_output_limb_7_col146.clone(),
                blake_g_output_limb_6_col121.clone(),
                blake_g_output_limb_7_col122.clone(),
                input_limb_34_col34.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}

