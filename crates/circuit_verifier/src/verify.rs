use circuit_common::{
    order_hash_map::OrderedHashMap,
    outputs::{copy_hash_into_reserved, new_verifier_context},
};
use circuits::{
    blake::{HashValue, blake},
    context::Context,
    finalize_constants::finalize_constants,
    ivalue::IValue,
    ops::Guess,
};
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
    pub output_addresses: Vec<usize>,
    pub preprocessed_column_log_sizes: OrderedHashMap<PreProcessedColumnId, u32>,
    pub preprocessed_root: HashValue<QM31>,
}

pub fn build_verification_circuit<Value: IValue>(
    circuit_config: CircuitConfig,
    proof: Proof<Value>,
    public_data: CircuitPublicData,
) -> Result<Context<Value>, String> {
    let mut context = new_verifier_context();
    let statement = CircuitStatement::new(
        &mut context,
        &circuit_config.output_addresses,
        &public_data.output_values,
        circuit_config.preprocessed_column_log_sizes.clone(),
        circuit_config.preprocessed_root,
    );

    let proof_config = ProofConfig::new(
        statement.get_components(),
        vec![true; statement.get_components().len()],
        circuit_config.preprocessed_column_log_sizes.len(),
        &circuit_config.config,
        INTERACTION_POW_BITS,
    );
    let proof_vars = proof.guess(&mut context);

    verify(&mut context, &proof_vars, &proof_config, &statement);

    // Deal with the outputs: hash the preprocessed root and all the output values except `u` (= the
    // last one).
    let preprocessed_root = statement.get_preprocessed_root(&mut context);
    let (_, output_preimage_skip_last) = statement.get_output_values().split_last().unwrap();
    let output_preimage: Vec<_> = [preprocessed_root.0, preprocessed_root.1]
        .into_iter()
        .chain(output_preimage_skip_last.iter().copied())
        .collect();
    let output_hash = blake(&mut context, &output_preimage, 16 * output_preimage.len());
    // Copy the resulting hash into the wires 3 and 4, and mark them as outputs.
    copy_hash_into_reserved(&mut context, output_hash);

    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    #[cfg(test)]
    context.circuit.check_yields();

    Ok(context)
}

pub fn verify_circuit(
    circuit_config: CircuitConfig,
    proof: Proof<QM31>,
    public_data: CircuitPublicData,
) -> Result<Context<QM31>, String> {
    let context = build_verification_circuit(circuit_config, proof, public_data)?;
    #[cfg(test)]
    context.check_vars_used();

    if !context.is_circuit_valid() {
        return Err("Verification failed".to_string());
    }
    Ok(context)
}
