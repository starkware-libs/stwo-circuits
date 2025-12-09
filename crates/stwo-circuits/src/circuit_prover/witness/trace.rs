use crate::circuit_air::components::CircuitClaim;
use crate::circuit_prover::witness::components::qm31_ops;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use crate::circuit_prover::witness::utils::TreeBuilder;
use crate::circuits::circuit::Circuit;
use stwo::core::fields::qm31::QM31;
use stwo::prover::backend::simd::SimdBackend;

pub fn write_trace(
    circuit: &Circuit,
    values: &[QM31],
    preprocessed_trace: &PreProcessedTrace<SimdBackend>,
    tree_builder: &mut impl TreeBuilder<SimdBackend>,
) -> CircuitClaim {
    let qm31_ops_claim = qm31_ops::write_trace(
        &circuit.add,
        &circuit.sub,
        &circuit.mul,
        &circuit.pointwise_mul,
        values,
        preprocessed_trace,
        tree_builder,
    );

    CircuitClaim { qm31_ops: qm31_ops_claim }
}
