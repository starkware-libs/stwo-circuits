use crate::circuits::context::Context;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::TreeBuilder;
use stwo::prover::backend::simd::SimdBackend;

/// Responsible for generating the CircuitClaim and writing the trace.
/// NOTE: Order of writing the trace is important, and should be consistent with [`CircuitClaim`],
/// [`CircuitInteractionClaimGenerator`].
// TODO(Gali): Consider adding Claim, ClaimGenerator, InteractionClaim and InteractionClaimGenerator
// traits.
pub struct CircuitClaimGenerator {
    // TODO(Gali): Add components.
}

impl CircuitClaimGenerator {
    pub fn new(_context: Context<QM31>) -> Self {
        Self {}
    }
    pub fn write_trace(
        &self,
        _tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sM31MerkleChannel>,
    ) -> (CircuitClaim, CircuitInteractionClaimGenerator) {
        (CircuitClaim {}, CircuitInteractionClaimGenerator {})
    }
}

pub struct CircuitClaim {}

impl CircuitClaim {
    pub fn mix_into(&self, _channel: &mut Blake2sM31Channel) {}
}

pub struct CircuitInteractionClaimGenerator {}
