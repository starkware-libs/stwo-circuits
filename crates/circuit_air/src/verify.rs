use circuits::{
    context::{Context, TraceContext},
    ops::Guess,
};
use circuits_stark_verifier::{
    proof::{Proof, ProofConfig},
    verify::verify,
};
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::statement::{CircuitStatement, INTERACTION_POW_BITS};

pub struct CircuitPublicData {
    pub output_values: Vec<QM31>,
}

pub struct CircuitConfig {
    pub config: PcsConfig,
    pub output_addresses: Vec<usize>,
    pub n_blake_gates: usize,
    pub preprocessed_column_ids: Vec<PreProcessedColumnId>,
}

pub fn verify_circuit(
    circuit_config: CircuitConfig,
    proof: Proof<QM31>,
    public_data: CircuitPublicData,
) -> Result<Context<QM31>, String> {
    let mut context = TraceContext::default();
    let statement = CircuitStatement::new(
        &mut context,
        &circuit_config.output_addresses,
        &public_data.output_values,
        circuit_config.n_blake_gates,
        circuit_config.preprocessed_column_ids.clone(),
    );

    let proof_config =
        ProofConfig::from_statement(&statement, &circuit_config.config, INTERACTION_POW_BITS);
    let proof_vars = proof.guess(&mut context);

    verify(&mut context, &proof_vars, &proof_config, &statement);
    context.check_vars_used();
    #[cfg(test)]
    context.finalize_guessed_vars();
    #[cfg(test)]
    context.circuit.check_yields();
    if !context.is_circuit_valid() {
        return Err("Verification failed".to_string());
    }
    Ok(context)
}
