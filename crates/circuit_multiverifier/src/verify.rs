use circuit_common::N_RESERVED;
use circuit_verifier::statement::CircuitStatement;
use circuit_verifier::verify::CircuitConfig;
use circuits::context::{Context, FinalizedContext};
use circuits::{
    blake::{HashValue, blake2s_m31},
    ivalue::IValue,
    ops::Guess,
};
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use circuits_stark_verifier::{
    proof::{Proof, ProofConfig},
    verify::verify,
};
use itertools::chain;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

#[cfg(test)]
#[path = "verify_test.rs"]
mod verify_test;

/// A circuit proof together with metadata needed to build the [`CircuitStatement`] against which
/// the proof will be verified. The multiverifier expects that output values of an input circuit
/// consist of 2 QM31 (usually representing an hash) + 1 QM31 equal to
/// [`circuits::context::U_VALUE`] coming from the constant finalization mechanism (see
/// [`circuits::finalize_constants`]).
pub struct MultiverifierInput<Value: IValue> {
    /// A circuit proof.
    pub proof: Proof<Value>,
    /// The preprocessed root of the circuit associated to `proof`.
    pub preprocessed_root: HashValue<QM31>,
    /// The output values of the circuit (excluding the value of the `u` wire at address
    /// [`circuits::context::U_VAR_IDX`]). The multiverifier only supports verification of circuits
    /// with two outputs.
    pub output_values: [QM31; N_RESERVED],
}

/// Configurations shared by the circuits being verified by the multiverifier and their proofs.
/// These values are static and fixed in advance.
pub struct SharedConfig {
    pub pcs_config: PcsConfig,
    pub proof_config: ProofConfig,
    pub preprocessed_column_log_sizes: OrderedHashMap<PreProcessedColumnId, u32>,
}

/// Builds a circuit that verifies two circuit proofs.
///
/// For each [`MultiverifierInput`], the function reconstructs the inner
/// [`CircuitStatement`] from `shared_config` and the per-proof
/// `preprocessed_root` and `outputs`, and runs the STARK verifier.
///
/// After both proofs are verified, the preprocessed roots and the inner
/// circuits' output values are concatenated and hashed. the
/// resulting hash is written into the two reserved output variables
/// of the outer circuit. The circuit is then finalized.
///
/// Both proofs must have been produced with the same [`SharedConfig`].
pub fn build_multiverifier_circuit<Value: IValue>(
    input0: MultiverifierInput<Value>,
    input1: MultiverifierInput<Value>,
    shared_config: &SharedConfig,
) -> FinalizedContext<Value> {
    let mut context = Context::new(N_RESERVED);

    let mut outer_verifier_output_preimage = vec![];
    // Verify sequentially the two proofs.
    for multiverifier_input in [input0, input1] {
        let MultiverifierInput { proof, preprocessed_root, output_values } = multiverifier_input;

        let circuit_config = CircuitConfig {
            config: shared_config.pcs_config,
            n_outputs: N_RESERVED,
            preprocessed_column_log_sizes: shared_config.preprocessed_column_log_sizes.clone(),
            preprocessed_root,
        };
        let statement = CircuitStatement::new(&mut context, &circuit_config, &output_values);
        let proof_vars = proof.guess(&mut context);

        verify(&mut context, &proof_vars, &shared_config.proof_config, &statement);
        outer_verifier_output_preimage.extend(chain!(
            [statement.preprocessed_root.0, statement.preprocessed_root.1],
            statement.get_output_values().iter().copied()
        ));
    }
    // The payload to be hashed is equal to:
    // [
    //      preprocessed_rootA.0, preprocessed_rootA.1, outputsA.0, outputsA.1
    //      preprocessed_rootB.0, preprocessed_rootB.1, outputsB.0, outputsB.1
    // ]
    // where A, B are the two circuits being verified.
    let output_hash = blake2s_m31(
        &mut context,
        &outer_verifier_output_preimage,
        16 * outer_verifier_output_preimage.len(),
    );
    // Copy the resulting hash into the reserved variables
    context.set_outputs(&[output_hash.0, output_hash.1]);

    let context = context.finalize(false);
    #[cfg(test)]
    context.circuit().check_yields();
    context
}
