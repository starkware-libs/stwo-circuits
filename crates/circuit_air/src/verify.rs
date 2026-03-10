use circuits::{blake::HashValue, context::Context, ivalue::IValue, ops::Guess};
use circuits_stark_verifier::{
    proof::{Proof, ProofConfig},
    verify::verify,
};
use itertools::izip;
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::statement::{CircuitStatement, INTERACTION_POW_BITS};

pub struct CircuitPublicData {
    pub output_values: Vec<QM31>,
}

#[derive(Debug, PartialEq)]
pub struct CircuitConfig {
    pub config: PcsConfig,
    pub output_addresses: Vec<usize>,
    pub n_blake_gates: usize,
    pub preprocessed_column_ids: Vec<PreProcessedColumnId>,
    pub preprocessed_root: HashValue<QM31>,
}

pub fn build_verification_circuit<Value: IValue>(
    circuit_config_vec: Vec<CircuitConfig>,
    proofs_vec: Vec<Proof<Value>>,
    public_data_vec: Vec<CircuitPublicData>,
) -> Result<Context<Value>, String> {
    let mut context = Context::default();
    assert_eq!(public_data_vec.len(), proofs_vec.len());
    assert_eq!(circuit_config_vec.len(), proofs_vec.len());
    for (proof, public_data, circuit_config) in
        izip!(proofs_vec, public_data_vec, circuit_config_vec)
    {
        verify_circuit_single_proof(&mut context, proof, &circuit_config, &public_data);
    }
    context.finalize_guessed_vars();
    #[cfg(test)]
    context.circuit.check_yields();

    Ok(context)
}

pub fn verify_circuit(
    circuit_config_vec: Vec<CircuitConfig>,
    proofs_vec: Vec<Proof<QM31>>,
    public_data_vec: Vec<CircuitPublicData>,
) -> Result<Context<QM31>, String> {
    let context = build_verification_circuit(circuit_config_vec, proofs_vec, public_data_vec)?;
    #[cfg(test)]
    context.check_vars_used();

    if !context.is_circuit_valid() {
        return Err("Verification failed".to_string());
    }
    Ok(context)
}

pub fn verify_circuit_single_proof<Value: IValue>(
    context: &mut Context<Value>,
    proof: Proof<Value>,
    circuit_config: &CircuitConfig,
    public_data: &CircuitPublicData,
) {
    let statement = CircuitStatement::new(
        context,
        &circuit_config.output_addresses,
        &public_data.output_values,
        circuit_config.n_blake_gates,
        circuit_config.preprocessed_column_ids.clone(),
        circuit_config.preprocessed_root,
    );

    let proof_config =
        ProofConfig::from_statement(&statement, &circuit_config.config, INTERACTION_POW_BITS);
    let proof_vars = proof.guess(context);

    verify(context, &proof_vars, &proof_config, &statement);
}
