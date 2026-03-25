pub mod blake_g_gate;
pub mod eq;
pub mod m_31_to_u_32;
pub mod prelude;
pub mod qm31_ops;
pub mod range_check_15;
pub mod range_check_16;
pub mod subroutines;
pub mod triple_xor;
pub mod verify_bitwise_xor_12;
pub mod verify_bitwise_xor_4;
pub mod verify_bitwise_xor_7;
pub mod verify_bitwise_xor_8;
pub mod verify_bitwise_xor_9;

use crate::{CircuitClaim, CircuitInteractionClaim, CircuitInteractionElements};
use stwo::core::air::Component;
use stwo_constraint_framework::TraceLocationAllocator;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

macro_rules! define_component_list {
    ($($variant:ident),* $(,)?) => {
        pub enum ComponentList {
            $($variant),*
        }
        pub const N_COMPONENTS: usize = [$(stringify!($variant)),*].len();
    };
}

define_component_list! {
    Eq,
    Qm31Ops,
    BlakeGGate,
    M31ToU32,
    TripleXor,
    VerifyBitwiseXor8,
    VerifyBitwiseXor12,
    VerifyBitwiseXor4,
    VerifyBitwiseXor7,
    VerifyBitwiseXor9,
    RangeCheck15,
    RangeCheck16,
}

pub struct CircuitComponents {
    pub eq: eq::Component,
    pub qm31_ops: qm31_ops::Component,
    pub blake_g_gate: blake_g_gate::Component,
    pub m_31_to_u_32: m_31_to_u_32::Component,
    pub triple_xor: triple_xor::Component,
    pub verify_bitwise_xor_8: verify_bitwise_xor_8::Component,
    pub verify_bitwise_xor_12: verify_bitwise_xor_12::Component,
    pub verify_bitwise_xor_4: verify_bitwise_xor_4::Component,
    pub verify_bitwise_xor_7: verify_bitwise_xor_7::Component,
    pub verify_bitwise_xor_9: verify_bitwise_xor_9::Component,
    pub range_check_15: range_check_15::Component,
    pub range_check_16: range_check_16::Component,
}
impl CircuitComponents {
    pub fn new(
        circuit_claim: &CircuitClaim,
        interaction_elements: &CircuitInteractionElements,
        interaction_claim: &CircuitInteractionClaim,
        // Describes the structure of the preprocessed trace. Sensitive to order.
        preprocessed_column_ids: &[PreProcessedColumnId],
    ) -> Self {
        let tree_span_provider =
            &mut TraceLocationAllocator::new_with_preprocessed_columns(preprocessed_column_ids);

        let eq_component = eq::Component::new(
            tree_span_provider,
            eq::Eval {
                log_size: circuit_claim.log_sizes[ComponentList::Eq as usize],
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::Eq as usize],
        );
        let qm31_ops_component = qm31_ops::Component::new(
            tree_span_provider,
            qm31_ops::Eval {
                log_size: circuit_claim.log_sizes[ComponentList::Qm31Ops as usize],
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::Qm31Ops as usize],
        );
        let blake_g_gate_component = blake_g_gate::Component::new(
            tree_span_provider,
            blake_g_gate::Eval {
                claim: blake_g_gate::Claim {
                    log_size: circuit_claim.log_sizes[ComponentList::BlakeGGate as usize],
                },
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::BlakeGGate as usize],
        );
        let m_31_to_u_32_component = m_31_to_u_32::Component::new(
            tree_span_provider,
            m_31_to_u_32::Eval {
                claim: m_31_to_u_32::Claim {
                    log_size: circuit_claim.log_sizes[ComponentList::M31ToU32 as usize],
                },
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::M31ToU32 as usize],
        );
        let triple_xor_component = triple_xor::Component::new(
            tree_span_provider,
            triple_xor::Eval {
                claim: triple_xor::Claim {
                    log_size: circuit_claim.log_sizes[ComponentList::TripleXor as usize],
                },
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::TripleXor as usize],
        );
        let verify_bitwise_xor_8_component = verify_bitwise_xor_8::Component::new(
            tree_span_provider,
            verify_bitwise_xor_8::Eval {
                claim: verify_bitwise_xor_8::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::VerifyBitwiseXor8 as usize],
        );
        let verify_bitwise_xor_12_component = verify_bitwise_xor_12::Component::new(
            tree_span_provider,
            verify_bitwise_xor_12::Eval {
                claim: verify_bitwise_xor_12::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::VerifyBitwiseXor12 as usize],
        );
        let verify_bitwise_xor_4_component = verify_bitwise_xor_4::Component::new(
            tree_span_provider,
            verify_bitwise_xor_4::Eval {
                claim: verify_bitwise_xor_4::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::VerifyBitwiseXor4 as usize],
        );
        let verify_bitwise_xor_7_component = verify_bitwise_xor_7::Component::new(
            tree_span_provider,
            verify_bitwise_xor_7::Eval {
                claim: verify_bitwise_xor_7::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::VerifyBitwiseXor7 as usize],
        );
        let verify_bitwise_xor_9_component = verify_bitwise_xor_9::Component::new(
            tree_span_provider,
            verify_bitwise_xor_9::Eval {
                claim: verify_bitwise_xor_9::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::VerifyBitwiseXor9 as usize],
        );
        let range_check_15_component = range_check_15::Component::new(
            tree_span_provider,
            range_check_15::Eval {
                claim: range_check_15::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::RangeCheck15 as usize],
        );
        let range_check_16_component = range_check_16::Component::new(
            tree_span_provider,
            range_check_16::Eval {
                claim: range_check_16::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::RangeCheck16 as usize],
        );
        Self {
            eq: eq_component,
            qm31_ops: qm31_ops_component,
            blake_g_gate: blake_g_gate_component,
            m_31_to_u_32: m_31_to_u_32_component,
            triple_xor: triple_xor_component,
            verify_bitwise_xor_8: verify_bitwise_xor_8_component,
            verify_bitwise_xor_12: verify_bitwise_xor_12_component,
            verify_bitwise_xor_4: verify_bitwise_xor_4_component,
            verify_bitwise_xor_7: verify_bitwise_xor_7_component,
            verify_bitwise_xor_9: verify_bitwise_xor_9_component,
            range_check_15: range_check_15_component,
            range_check_16: range_check_16_component,
        }
    }

    pub fn components(self) -> Vec<Box<dyn Component>> {
        vec![
            Box::new(self.eq) as Box<dyn Component>,
            Box::new(self.qm31_ops) as Box<dyn Component>,
            Box::new(self.blake_g_gate) as Box<dyn Component>,
            Box::new(self.m_31_to_u_32) as Box<dyn Component>,
            Box::new(self.triple_xor) as Box<dyn Component>,
            Box::new(self.verify_bitwise_xor_8) as Box<dyn Component>,
            Box::new(self.verify_bitwise_xor_12) as Box<dyn Component>,
            Box::new(self.verify_bitwise_xor_4) as Box<dyn Component>,
            Box::new(self.verify_bitwise_xor_7) as Box<dyn Component>,
            Box::new(self.verify_bitwise_xor_9) as Box<dyn Component>,
            Box::new(self.range_check_15) as Box<dyn Component>,
            Box::new(self.range_check_16) as Box<dyn Component>,
        ]
    }
}
