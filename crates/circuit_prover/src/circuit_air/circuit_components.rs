use crate::circuit_air::components::{
    blake_g_gate, eq, m_31_to_u_32, qm31_ops, range_check_16, triple_xor, verify_bitwise_xor_4,
    verify_bitwise_xor_7, verify_bitwise_xor_8, verify_bitwise_xor_9, verify_bitwise_xor_12,
};
use circuit_verifier::circuit_claim::{
    CircuitClaim, CircuitInteractionClaim, CircuitInteractionElements,
};
use circuit_verifier::circuit_components::ComponentList;
use stwo::core::air::Component;
use stwo_constraint_framework::TraceLocationAllocator;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

pub struct CircuitComponents {
    pub eq: eq::Component,
    pub qm31_ops: qm31_ops::Component,
    pub triple_xor: triple_xor::Component,
    pub m_31_to_u_32: m_31_to_u_32::Component,
    pub blake_g_gate: blake_g_gate::Component,
    pub verify_bitwise_xor_8: verify_bitwise_xor_8::Component,
    pub verify_bitwise_xor_12: verify_bitwise_xor_12::Component,
    pub verify_bitwise_xor_4: verify_bitwise_xor_4::Component,
    pub verify_bitwise_xor_7: verify_bitwise_xor_7::Component,
    pub verify_bitwise_xor_9: verify_bitwise_xor_9::Component,
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
            triple_xor: triple_xor_component,
            m_31_to_u_32: m_31_to_u_32_component,
            blake_g_gate: blake_g_gate_component,
            verify_bitwise_xor_8: verify_bitwise_xor_8_component,
            verify_bitwise_xor_12: verify_bitwise_xor_12_component,
            verify_bitwise_xor_4: verify_bitwise_xor_4_component,
            verify_bitwise_xor_7: verify_bitwise_xor_7_component,
            verify_bitwise_xor_9: verify_bitwise_xor_9_component,
            range_check_16: range_check_16_component,
        }
    }

    pub fn components(self) -> Vec<Box<dyn Component>> {
        vec![
            Box::new(self.eq) as Box<dyn Component>,
            Box::new(self.qm31_ops) as Box<dyn Component>,
            Box::new(self.triple_xor) as Box<dyn Component>,
            Box::new(self.m_31_to_u_32) as Box<dyn Component>,
            Box::new(self.blake_g_gate) as Box<dyn Component>,
            Box::new(self.verify_bitwise_xor_8) as Box<dyn Component>,
            Box::new(self.verify_bitwise_xor_12) as Box<dyn Component>,
            Box::new(self.verify_bitwise_xor_4) as Box<dyn Component>,
            Box::new(self.verify_bitwise_xor_7) as Box<dyn Component>,
            Box::new(self.verify_bitwise_xor_9) as Box<dyn Component>,
            Box::new(self.range_check_16) as Box<dyn Component>,
        ]
    }
}
