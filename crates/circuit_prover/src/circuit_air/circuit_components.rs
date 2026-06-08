use crate::circuit_air::components::{
    blake_g_gate, eq, m_31_to_u_32, qm31_ops, range_check_16, triple_xor, verify_bitwise_xor_4,
    verify_bitwise_xor_7, verify_bitwise_xor_8, verify_bitwise_xor_9, verify_bitwise_xor_12,
};
use circuit_verifier::circuit_claim::{CircuitInteractionClaim, CircuitInteractionElements};
use circuit_verifier::circuit_components::{ComponentList, N_COMPONENTS};
use stwo::core::air::Component;
use stwo::prover::ComponentProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo_constraint_framework::TraceLocationAllocator;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

pub struct CircuitComponents {
    components: Vec<Box<dyn ComponentProver<SimdBackend>>>,
}
impl CircuitComponents {
    pub fn new(
        interaction_elements: &CircuitInteractionElements,
        interaction_claim: &CircuitInteractionClaim,
        component_log_sizes: &[u32; N_COMPONENTS],
        // Describes the structure of the preprocessed trace. Sensitive to order.
        preprocessed_column_ids: &[PreProcessedColumnId],
    ) -> Self {
        let tree_span_provider =
            &mut TraceLocationAllocator::new_with_preprocessed_columns(preprocessed_column_ids);

        let eq_component = eq::Component::new(
            tree_span_provider,
            eq::Eval {
                log_size: component_log_sizes[ComponentList::Eq as usize],
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::Eq as usize],
        );
        let qm31_ops_component = qm31_ops::Component::new(
            tree_span_provider,
            qm31_ops::Eval {
                log_size: component_log_sizes[ComponentList::Qm31Ops as usize],
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::Qm31Ops as usize],
        );
        let triple_xor_component = triple_xor::Component::new(
            tree_span_provider,
            triple_xor::Eval {
                claim: triple_xor::Claim {
                    log_size: component_log_sizes[ComponentList::TripleXor as usize],
                },
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::TripleXor as usize],
        );
        let m_31_to_u_32_component = m_31_to_u_32::Component::new(
            tree_span_provider,
            m_31_to_u_32::Eval {
                claim: m_31_to_u_32::Claim {
                    log_size: component_log_sizes[ComponentList::M31ToU32 as usize],
                },
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::M31ToU32 as usize],
        );
        let blake_g_gate_component = blake_g_gate::Component::new(
            tree_span_provider,
            blake_g_gate::Eval {
                claim: blake_g_gate::Claim {
                    log_size: component_log_sizes[ComponentList::BlakeGGate as usize],
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
            components: vec![
                Box::new(eq_component) as Box<dyn ComponentProver<SimdBackend>>,
                Box::new(qm31_ops_component) as Box<dyn ComponentProver<SimdBackend>>,
                Box::new(triple_xor_component) as Box<dyn ComponentProver<SimdBackend>>,
                Box::new(m_31_to_u_32_component) as Box<dyn ComponentProver<SimdBackend>>,
                Box::new(blake_g_gate_component) as Box<dyn ComponentProver<SimdBackend>>,
                Box::new(verify_bitwise_xor_8_component) as Box<dyn ComponentProver<SimdBackend>>,
                Box::new(verify_bitwise_xor_12_component) as Box<dyn ComponentProver<SimdBackend>>,
                Box::new(verify_bitwise_xor_4_component) as Box<dyn ComponentProver<SimdBackend>>,
                Box::new(verify_bitwise_xor_7_component) as Box<dyn ComponentProver<SimdBackend>>,
                Box::new(verify_bitwise_xor_9_component) as Box<dyn ComponentProver<SimdBackend>>,
                Box::new(range_check_16_component) as Box<dyn ComponentProver<SimdBackend>>,
            ],
        }
    }

    pub fn component_provers(&self) -> Vec<&dyn ComponentProver<SimdBackend>> {
        self.components.iter().map(|c| c.as_ref()).collect()
    }

    pub fn components(self) -> Vec<Box<dyn Component>> {
        self.components.into_iter().map(|c| c as Box<dyn Component>).collect()
    }
}
