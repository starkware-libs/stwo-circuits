pub mod blake_g;
pub mod blake_gate;
pub mod blake_output;
pub mod blake_round;
pub mod blake_round_sigma;
pub mod eq;
pub mod prelude;
pub mod qm31_ops;
pub mod range_check_15;
pub mod range_check_16;
pub mod subroutines;
pub mod triple_xor_32;
pub mod verify_bitwise_xor_12;
pub mod verify_bitwise_xor_4;
pub mod verify_bitwise_xor_7;
pub mod verify_bitwise_xor_8;
pub mod verify_bitwise_xor_9;

#[cfg(test)]
pub mod constraints_regression_test_values;

use crate::circuit_air::{CircuitClaim, CircuitInteractionClaim, CircuitInteractionElements};
use itertools::chain;
use stwo::core::air::Component;
use stwo::prover::ComponentProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo_constraint_framework::TraceLocationAllocator;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

pub enum ComponentList {
    Eq,
    Qm31Ops,
    BlakeGate,
    BlakeRound,
    BlakeRoundSigma,
    BlakeG,
}
pub const N_COMPONENTS: usize = std::mem::variant_count::<ComponentList>();

pub struct CircuitComponents {
    pub eq: eq::Component,
    pub qm31_ops: qm31_ops::Component,
    pub blake_gate: blake_gate::Component,
    pub blake_round: blake_round::Component,
    pub blake_round_sigma: blake_round_sigma::Component,
    pub blake_g: blake_g::Component,
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
        let blake_gate_component = blake_gate::Component::new(
            tree_span_provider,
            blake_gate::Eval {
                claim: blake_gate::Claim {
                    log_size: circuit_claim.log_sizes[ComponentList::BlakeGate as usize],
                },
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::BlakeGate as usize],
        );
        let blake_round_component = blake_round::Component::new(
            tree_span_provider,
            blake_round::Eval {
                claim: blake_round::Claim {
                    log_size: circuit_claim.log_sizes[ComponentList::BlakeRound as usize],
                },
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::BlakeRound as usize],
        );
        let blake_round_sigma_component = blake_round_sigma::Component::new(
            tree_span_provider,
            blake_round_sigma::Eval {
                claim: blake_round_sigma::Claim {},
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::BlakeRoundSigma as usize],
        );
        let blake_g_component = blake_g::Component::new(
            tree_span_provider,
            blake_g::Eval {
                claim: blake_g::Claim {
                    log_size: circuit_claim.log_sizes[ComponentList::BlakeG as usize],
                },
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::BlakeG as usize],
        );
        Self {
            eq: eq_component,
            qm31_ops: qm31_ops_component,
            blake_gate: blake_gate_component,
            blake_round: blake_round_component,
            blake_round_sigma: blake_round_sigma_component,
            blake_g: blake_g_component,
        }
    }

    pub fn provers(&self) -> Vec<&dyn ComponentProver<SimdBackend>> {
        chain!([
            &self.eq as &dyn ComponentProver<SimdBackend>,
            &self.qm31_ops as &dyn ComponentProver<SimdBackend>,
            &self.blake_gate as &dyn ComponentProver<SimdBackend>,
            &self.blake_round as &dyn ComponentProver<SimdBackend>,
            &self.blake_round_sigma as &dyn ComponentProver<SimdBackend>,
            &self.blake_g as &dyn ComponentProver<SimdBackend>,
        ])
        .collect()
    }

    pub fn components(self) -> Vec<Box<dyn Component>> {
        vec![
            Box::new(self.eq) as Box<dyn Component>,
            Box::new(self.qm31_ops) as Box<dyn Component>,
            Box::new(self.blake_gate) as Box<dyn Component>,
            Box::new(self.blake_round) as Box<dyn Component>,
            Box::new(self.blake_round_sigma) as Box<dyn Component>,
            Box::new(self.blake_g) as Box<dyn Component>,
        ]
    }
}
