use crate::circuit_air::components::{
    blake_g_gate, eq, m_31_to_u_32, qm_31_ops, range_check_16, triple_xor, verify_bitwise_xor_4,
    verify_bitwise_xor_7, verify_bitwise_xor_8, verify_bitwise_xor_9, verify_bitwise_xor_12,
};
use circuit_verifier::circuit_claim::{CircuitInteractionClaim, CircuitInteractionElements};
use circuit_verifier::circuit_components::ComponentList;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
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
        component_log_sizes: &OrderedHashMap<&'static str, u32>,
        // Describes the structure of the preprocessed trace. Sensitive to order.
        preprocessed_column_ids: &[PreProcessedColumnId],
    ) -> Self {
        let tree_span_provider =
            &mut TraceLocationAllocator::new_with_preprocessed_columns(preprocessed_column_ids);

        let eq_component = eq::Component::new(
            tree_span_provider,
            eq::Eval {
                log_size: component_log_sizes[ComponentList::Eq.name()],
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::Eq.idx()],
        );
        let qm31_ops_component = qm_31_ops::Component::new(
            tree_span_provider,
            qm_31_ops::Eval {
                claim: qm_31_ops::Claim {
                    log_size: component_log_sizes[ComponentList::Qm31Ops.name()],
                },
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::Qm31Ops.idx()],
        );
        let triple_xor_component = triple_xor::Component::new(
            tree_span_provider,
            triple_xor::Eval {
                claim: triple_xor::Claim {
                    log_size: component_log_sizes[ComponentList::TripleXor.name()],
                },
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::TripleXor.idx()],
        );
        let m_31_to_u_32_component = m_31_to_u_32::Component::new(
            tree_span_provider,
            m_31_to_u_32::Eval {
                claim: m_31_to_u_32::Claim {
                    log_size: component_log_sizes[ComponentList::M31ToU32.name()],
                },
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::M31ToU32.idx()],
        );
        let blake_g_gate_component = blake_g_gate::Component::new(
            tree_span_provider,
            blake_g_gate::Eval {
                claim: blake_g_gate::Claim {
                    log_size: component_log_sizes[ComponentList::BlakeGGate.name()],
                },
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::BlakeGGate.idx()],
        );
        let verify_bitwise_xor_8_component = verify_bitwise_xor_8::Component::new(
            tree_span_provider,
            verify_bitwise_xor_8::Eval {
                claim: verify_bitwise_xor_8::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::VerifyBitwiseXor8.idx()],
        );
        let verify_bitwise_xor_12_component = verify_bitwise_xor_12::Component::new(
            tree_span_provider,
            verify_bitwise_xor_12::Eval {
                claim: verify_bitwise_xor_12::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::VerifyBitwiseXor12.idx()],
        );
        let verify_bitwise_xor_4_component = verify_bitwise_xor_4::Component::new(
            tree_span_provider,
            verify_bitwise_xor_4::Eval {
                claim: verify_bitwise_xor_4::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::VerifyBitwiseXor4.idx()],
        );
        let verify_bitwise_xor_7_component = verify_bitwise_xor_7::Component::new(
            tree_span_provider,
            verify_bitwise_xor_7::Eval {
                claim: verify_bitwise_xor_7::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::VerifyBitwiseXor7.idx()],
        );
        let verify_bitwise_xor_9_component = verify_bitwise_xor_9::Component::new(
            tree_span_provider,
            verify_bitwise_xor_9::Eval {
                claim: verify_bitwise_xor_9::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::VerifyBitwiseXor9.idx()],
        );
        let range_check_16_component = range_check_16::Component::new(
            tree_span_provider,
            range_check_16::Eval {
                claim: range_check_16::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::RangeCheck16.idx()],
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

    pub fn components(&self) -> Vec<&dyn Component> {
        self.components.iter().map(|c| c.as_ref() as &dyn Component).collect()
    }
}
