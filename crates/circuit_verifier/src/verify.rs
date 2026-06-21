use circuit_common::N_RESERVED;
use circuits::{
    blake::{HashValue, blake2s_m31},
    context::{Context, FinalizedContext},
    ivalue::IValue,
    ops::Guess,
};
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use circuits_stark_verifier::{
    proof::{Proof, ProofConfig},
    statement::Statement,
    verify::verify,
};
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::statement::{CircuitStatement, INTERACTION_POW_BITS};

pub struct CircuitPublicData {
    pub output_values: Vec<QM31>,
}

#[derive(Debug, PartialEq)]
pub struct CircuitConfig {
    pub config: PcsConfig,
    /// The number of output gates of the circuit, excluding the output gate of the `u` constant
    /// (at address [`circuits::context::U_VAR_IDX`]).
    pub n_outputs: usize,
    pub preprocessed_column_log_sizes: OrderedHashMap<PreProcessedColumnId, u32>,
    pub preprocessed_root: HashValue<QM31>,
}

/// Builds the circuit that verifies a proof of execution of another circuit.
///
/// The circuit:
/// 1. Builds a [`CircuitStatement`] from the output addresses, output values, preprocessed column
///    log sizes, and preprocessed root.
/// 2. Guesses the proof values into the circuit and runs the STARK verification.
/// 3. Hashes the preprocessed root together with all output values except the last (`u`) via Blake,
///    and copies the resulting hash into the reserved output wires (3 and 4). To ensure soundness
///    in a recursive setup, the outer-most verifier (assumed honest) must reconstruct the whole
///    chain of output hashes computed during the recursive steps.
/// 4. Finalizes constants and guessed variables.
pub fn build_verification_circuit<Value: IValue>(
    circuit_config: CircuitConfig,
    proof: Proof<Value>,
    public_data: CircuitPublicData,
) -> Result<FinalizedContext<Value>, String> {
    let mut context = Context::new(N_RESERVED);
    let statement =
        CircuitStatement::new(&mut context, &circuit_config, &public_data.output_values);

    let proof_config = ProofConfig::new(
        statement.get_components(),
        circuit_config.preprocessed_column_log_sizes.len(),
        &circuit_config.config,
        INTERACTION_POW_BITS,
    );
    let proof_vars = proof.guess(&mut context);

    verify(&mut context, &proof_vars, &proof_config, &statement);

    // Deal with the outputs: hash the preprocessed root and all the output values except `u` (= the
    // last one). This is fine for soundness because `u` is checked as part of the logup sum.
    let preprocessed_root = statement.get_preprocessed_root(&mut context);
    let output_preimage: Vec<_> = [preprocessed_root.0, preprocessed_root.1]
        .into_iter()
        .chain(statement.get_output_values().iter().copied())
        .collect();
    let output_hash = blake2s_m31(&mut context, &output_preimage, 16 * output_preimage.len());
    // Copy the resulting hash into the wires 3 and 4, and mark them as outputs.
    context.set_outputs(&[output_hash.0, output_hash.1]);

    let context = context.finalize(false);
    #[cfg(test)]
    context.circuit().check_yields();

    Ok(context)
}

pub fn verify_circuit(
    circuit_config: CircuitConfig,
    proof: Proof<QM31>,
    public_data: CircuitPublicData,
) -> Result<FinalizedContext<QM31>, String> {
    let context = build_verification_circuit(circuit_config, proof, public_data)?;
    #[cfg(test)]
    context.check_vars_used();

    if !context.is_circuit_valid() {
        return Err("Verification failed".to_string());
    }
    Ok(context)
}
