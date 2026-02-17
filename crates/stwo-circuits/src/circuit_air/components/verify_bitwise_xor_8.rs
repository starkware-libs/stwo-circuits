// This file was created by the AIR team.

use crate::circuit_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 2;
pub const LOG_SIZE: u32 = 16;
pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

pub struct Eval {
    pub claim: Claim,
    pub common_lookup_elements: relations::CommonLookupElements,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Claim {}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let trace_log_sizes = vec![LOG_SIZE; N_TRACE_COLUMNS];
        let interaction_log_sizes = vec![LOG_SIZE; SECURE_EXTENSION_DEGREE];
        TreeVec::new(vec![vec![], trace_log_sizes, interaction_log_sizes])
    }

    pub fn mix_into(&self, _channel: &mut impl Channel) {}
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
        LOG_SIZE
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let relation_id = E::F::from(M31::from(112558620));
        let relation_id_b = E::F::from(M31::from(521092554));
        let bitwise_xor_8_0 =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "bitwise_xor_8_0".to_owned() });
        let bitwise_xor_8_1 =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "bitwise_xor_8_1".to_owned() });
        let bitwise_xor_8_2 =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "bitwise_xor_8_2".to_owned() });
        let multiplicity_0 = eval.next_trace_mask();
        let multiplicity_1 = eval.next_trace_mask();

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(multiplicity_0),
            &[
                relation_id.clone(),
                bitwise_xor_8_0.clone(),
                bitwise_xor_8_1.clone(),
                bitwise_xor_8_2.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(multiplicity_1),
            &[
                relation_id_b.clone(),
                bitwise_xor_8_0.clone(),
                bitwise_xor_8_1.clone(),
                bitwise_xor_8_2.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}
