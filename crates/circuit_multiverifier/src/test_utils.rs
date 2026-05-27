use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_verifier::{
    statement::{INTERACTION_POW_BITS, all_circuit_components},
    verify::CircuitConfig,
};
use circuits::{blake::HashValue, context::Context, ivalue::NoValue};
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};

use crate::verify::{MultiverifierInput, SharedConfig, build_multiverifier_circuit};

/// Builds a `NoValue` multiverifier and preprocesses it. The multiverifier is build by feeding it
/// two identical proofs of a circuit.
#[expect(dead_code)]
pub fn get_preprocessed_multiverifier_from_circuit(
    preprocessed_leaf_circuit: &PreprocessedCircuit,
    pcs_config: PcsConfig,
) -> (PreprocessedCircuit, Context<NoValue>) {
    let all_circuit_components = &all_circuit_components::<NoValue>();
    let proof_config = ProofConfig::new(
        all_circuit_components,
        vec![true; all_circuit_components.len()],
        preprocessed_leaf_circuit.preprocessed_trace.n_columns(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );
    let subcircuit_config = CircuitConfig {
        config: pcs_config,
        output_addresses: preprocessed_leaf_circuit.params.output_addresses.clone(),
        preprocessed_column_log_sizes: preprocessed_leaf_circuit.preprocessed_trace.log_sizes(),
        preprocessed_root: HashValue(QM31::from(0), QM31::from(0)),
    };
    let empty_input = || MultiverifierInput {
        proof: empty_proof(&proof_config),
        preprocessed_root: subcircuit_config.preprocessed_root,
        hashed_outputs: [QM31::from(0); 2],
    };
    let shared_config = SharedConfig {
        pcs_config,
        proof_config: ProofConfig::new(
            all_circuit_components,
            vec![true; all_circuit_components.len()],
            proof_config.n_preprocessed_columns,
            &subcircuit_config.config,
            INTERACTION_POW_BITS,
        ),
        preprocessed_column_log_sizes: subcircuit_config.preprocessed_column_log_sizes.clone(),
        output_addresses: subcircuit_config.output_addresses,
    };
    let mut multiverifier_context =
        build_multiverifier_circuit::<NoValue>(empty_input(), empty_input(), &shared_config);
    let preprocessed_multiverifier_circuit =
        PreprocessedCircuit::preprocess_circuit(&mut multiverifier_context);
    (preprocessed_multiverifier_circuit, multiverifier_context)
}
