use crate::circuit_air::components::{
    blake_g_gate, eq, m_31_to_u_32, qm_31_ops, range_check_16, triple_xor, verify_bitwise_xor_4,
    verify_bitwise_xor_7, verify_bitwise_xor_8, verify_bitwise_xor_9, verify_bitwise_xor_12,
};
use circuit_verifier::circuit_claim::{
    CircuitInteractionClaim, CircuitInteractionElements, ClaimedSum,
};
use circuit_verifier::circuit_components::{COMPONENT_NAMES, N_COMPONENTS};
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use itertools::{Itertools, zip_eq};
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

        let lookup_elements = &interaction_elements.common_lookup_elements;
        let claimed_sums = &interaction_claim.claimed_sums;

        // A constructor closure per component, indexed by `ComponentList`.
        //
        // The `TraceLocationAllocator` assigns each component a contiguous range of committed
        // columns in the order the constructors are called. Calling them in size-sorted order
        // therefore makes committed order size-sorted, and the verifier can skip the query-column
        // sort.
        #[allow(clippy::type_complexity)]
        let constructors: [Box<
            dyn FnMut(
                &mut TraceLocationAllocator,
                u32,
                ClaimedSum,
            ) -> Box<dyn ComponentProver<SimdBackend>>,
        >; N_COMPONENTS] = [
            Box::new(|tsp, log_size, claimed_sum| {
                Box::new(eq::Component::new(
                    tsp,
                    eq::Eval { log_size, common_lookup_elements: lookup_elements.clone() },
                    claimed_sum,
                )) as Box<dyn ComponentProver<SimdBackend>>
            }),
            Box::new(|tsp, log_size, claimed_sum| {
                Box::new(qm_31_ops::Component::new(
                    tsp,
                    qm_31_ops::Eval {
                        claim: qm_31_ops::Claim { log_size },
                        common_lookup_elements: lookup_elements.clone(),
                    },
                    claimed_sum,
                )) as Box<dyn ComponentProver<SimdBackend>>
            }),
            Box::new(|tsp, log_size, claimed_sum| {
                Box::new(triple_xor::Component::new(
                    tsp,
                    triple_xor::Eval {
                        claim: triple_xor::Claim { log_size },
                        common_lookup_elements: lookup_elements.clone(),
                    },
                    claimed_sum,
                )) as Box<dyn ComponentProver<SimdBackend>>
            }),
            Box::new(|tsp, log_size, claimed_sum| {
                Box::new(m_31_to_u_32::Component::new(
                    tsp,
                    m_31_to_u_32::Eval {
                        claim: m_31_to_u_32::Claim { log_size },
                        common_lookup_elements: lookup_elements.clone(),
                    },
                    claimed_sum,
                )) as Box<dyn ComponentProver<SimdBackend>>
            }),
            Box::new(|tsp, log_size, claimed_sum| {
                Box::new(blake_g_gate::Component::new(
                    tsp,
                    blake_g_gate::Eval {
                        claim: blake_g_gate::Claim { log_size },
                        common_lookup_elements: lookup_elements.clone(),
                    },
                    claimed_sum,
                )) as Box<dyn ComponentProver<SimdBackend>>
            }),
            Box::new(|tsp, _log_size, claimed_sum| {
                Box::new(verify_bitwise_xor_8::Component::new(
                    tsp,
                    verify_bitwise_xor_8::Eval {
                        claim: verify_bitwise_xor_8::Claim {},
                        common_lookup_elements: lookup_elements.clone(),
                    },
                    claimed_sum,
                )) as Box<dyn ComponentProver<SimdBackend>>
            }),
            Box::new(|tsp, _log_size, claimed_sum| {
                Box::new(verify_bitwise_xor_12::Component::new(
                    tsp,
                    verify_bitwise_xor_12::Eval {
                        claim: verify_bitwise_xor_12::Claim {},
                        common_lookup_elements: lookup_elements.clone(),
                    },
                    claimed_sum,
                )) as Box<dyn ComponentProver<SimdBackend>>
            }),
            Box::new(|tsp, _log_size, claimed_sum| {
                Box::new(verify_bitwise_xor_4::Component::new(
                    tsp,
                    verify_bitwise_xor_4::Eval {
                        claim: verify_bitwise_xor_4::Claim {},
                        common_lookup_elements: lookup_elements.clone(),
                    },
                    claimed_sum,
                )) as Box<dyn ComponentProver<SimdBackend>>
            }),
            Box::new(|tsp, _log_size, claimed_sum| {
                Box::new(verify_bitwise_xor_7::Component::new(
                    tsp,
                    verify_bitwise_xor_7::Eval {
                        claim: verify_bitwise_xor_7::Claim {},
                        common_lookup_elements: lookup_elements.clone(),
                    },
                    claimed_sum,
                )) as Box<dyn ComponentProver<SimdBackend>>
            }),
            Box::new(|tsp, _log_size, claimed_sum| {
                Box::new(verify_bitwise_xor_9::Component::new(
                    tsp,
                    verify_bitwise_xor_9::Eval {
                        claim: verify_bitwise_xor_9::Claim {},
                        common_lookup_elements: lookup_elements.clone(),
                    },
                    claimed_sum,
                )) as Box<dyn ComponentProver<SimdBackend>>
            }),
            Box::new(|tsp, _log_size, claimed_sum| {
                Box::new(range_check_16::Component::new(
                    tsp,
                    range_check_16::Eval {
                        claim: range_check_16::Claim {},
                        common_lookup_elements: lookup_elements.clone(),
                    },
                    claimed_sum,
                )) as Box<dyn ComponentProver<SimdBackend>>
            }),
        ];

        // `constructors` and `COMPONENT_NAMES` are both indexed by `ComponentList`, so look up each
        // component's log size by name rather than relying on the map's iteration order.
        let components: Vec<Box<dyn ComponentProver<SimdBackend>>> =
            zip_eq(COMPONENT_NAMES, constructors)
                .map(|(name, constructor)| (component_log_sizes[name], constructor))
                .sorted_by_key(|(log_size, _)| *log_size)
                .zip_eq(claimed_sums)
                .map(|((log_size, mut constructor), claimed_sum)| {
                    constructor(&mut *tree_span_provider, log_size, *claimed_sum)
                })
                .collect();

        Self { components }
    }

    pub fn component_provers(&self) -> Vec<&dyn ComponentProver<SimdBackend>> {
        self.components.iter().map(|c| c.as_ref()).collect()
    }

    pub fn components(&self) -> Vec<&dyn Component> {
        self.components.iter().map(|c| c.as_ref() as &dyn Component).collect()
    }
}
