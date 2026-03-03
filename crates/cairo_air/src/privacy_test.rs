use std::fs::File;

use cairo_air::utils::binary_deserialize_from_file;
use cairo_air::verifier::INTERACTION_POW_BITS;
use circuit_air::statement::all_circuit_components;
use circuit_air::verify::{CircuitConfig, verify_circuit};
use circuit_prover::prover::{preprare_circuit_proof_for_circuit_verifier, prove_circuit};
use circuits::blake::HashValue;
use circuits::ivalue::qm31_from_u32s;
use circuits_stark_verifier::proof::ProofConfig;
use itertools::Itertools;
use stwo::core::fields::qm31::QM31;

use crate::privacy::{
    PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT, privacy_cairo_verifier_config, privacy_components,
};
use crate::test::{verify_cairo, verify_cairo_with_component_set};
use crate::utils::get_proof_file_path;
use crate::verify::build_cairo_verifier_circuit;

#[test]
fn test_verify_privacy() {
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();

    // Verify the proof.
    let context = verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();

    // Build the verifier circuit via NoValue.
    let const_config = privacy_cairo_verifier_config();
    let novalue_context = build_cairo_verifier_circuit(&const_config);

    // Check that building the verifier circuit via NoValue produces the same topology.
    // TODO(Gali): Consider comparing unused and maybe unused vars.
    assert_eq!(context.circuit, novalue_context.circuit);
    assert_eq!(context.stats, novalue_context.stats);
    assert_eq!(context.guessed_vars, novalue_context.guessed_vars);
    let constants_a = context.constants().iter().map(|(k, v)| (*k, v.idx)).collect_vec();
    let constants_b = novalue_context.constants().iter().map(|(k, v)| (*k, v.idx)).collect_vec();
    assert_eq!(constants_a, constants_b);
}

#[test]
fn test_verify_privacy_with_recursion() {
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();

    let mut context = verify_cairo(&cairo_proof).unwrap();
    let circuit_proof = prove_circuit(&mut context);
    let preprocessed_column_ids = circuit_proof.preprocessed_circuit.preprocessed_trace.ids();
    let proof_config = ProofConfig::from_components(
        &all_circuit_components::<QM31>(),
        preprocessed_column_ids.len(),
        &circuit_proof.pcs_config,
        INTERACTION_POW_BITS,
    );
    let preprocessed_root = PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT;
    let preprocessed_root = HashValue(
        qm31_from_u32s(
            preprocessed_root[0],
            preprocessed_root[1],
            preprocessed_root[2],
            preprocessed_root[3],
        ),
        qm31_from_u32s(
            preprocessed_root[4],
            preprocessed_root[5],
            preprocessed_root[6],
            preprocessed_root[7],
        ),
    );
    let circuit_config = CircuitConfig {
        config: circuit_proof.pcs_config,
        output_addresses: circuit_proof.preprocessed_circuit.params.output_addresses.clone(),
        n_blake_gates: circuit_proof.preprocessed_circuit.params.n_blake_gates,
        preprocessed_column_ids,
        preprocessed_root,
    };
    let (proof, public_data) =
        preprare_circuit_proof_for_circuit_verifier(circuit_proof, proof_config);
    verify_circuit(circuit_config, proof, public_data).unwrap();
}
