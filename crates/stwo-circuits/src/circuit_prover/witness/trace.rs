use crate::circuit_air::components::CircuitClaim;
use crate::circuit_prover::witness::components::qm31_ops;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::TreeBuilder;
use stwo::prover::backend::simd::SimdBackend;

pub fn write_trace(
    context_values: &[QM31],
    preprocessed_trace: &PreProcessedTrace,
    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sM31MerkleChannel>,
) -> CircuitClaim {
    let qm31_ops_claim = qm31_ops::write_trace(context_values, preprocessed_trace, tree_builder);

    CircuitClaim { qm31_ops: qm31_ops_claim }
}
