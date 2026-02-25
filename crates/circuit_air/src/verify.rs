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

use crate::statement::{CircuitStatement, INTERACTION_POW_BITS, all_circuit_components};

pub struct CircuitPublicData {
    pub output_addresses: Vec<usize>,
    pub output_values: Vec<QM31>,
    pub n_blake_gates: usize,
    pub preprocessed_column_ids: Vec<PreProcessedColumnId>,
}

pub fn verify_circuit(
    pcs_config: PcsConfig,
    proof: Proof<QM31>,
    public_data: CircuitPublicData,
) -> Result<Context<QM31>, String> {
    let mut context = TraceContext::default();
    let statement = CircuitStatement::new(
        &mut context,
        &public_data.output_addresses,
        &public_data.output_values,
        public_data.n_blake_gates,
        public_data.preprocessed_column_ids.clone(),
    );
    // TODO(Gali): Use a fixed config.
    let config = ProofConfig::from_components(
        &all_circuit_components::<QM31>(),
        public_data.preprocessed_column_ids.len(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );
    let proof_vars = proof.guess(&mut context);

    verify(&mut context, &proof_vars, &config, &statement);
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
