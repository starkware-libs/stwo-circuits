use crate::utils::get_proof_file_path;
use crate::{test::verify_cairo, utils::verify_circuit_proofs};
use cairo_air::utils::binary_deserialize_from_file;
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{BaseColumnPool, SimdBackend, prove_circuit_assignment};
use std::fs::File;

#[test]
fn test_verify_privacy_with_2_to_1_recursion() {
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();

    let mut context = verify_cairo(&cairo_proof).unwrap();
    let preprocessed = PreprocessedCircuit::preprocess_circuit(&mut context);
    let circuit_proof_1 = prove_circuit_assignment(
        context.values(),
        &preprocessed,
        &BaseColumnPool::<SimdBackend>::new(),
    );
    let circuit_proof_2 = prove_circuit_assignment(
        context.values(),
        &preprocessed,
        &BaseColumnPool::<SimdBackend>::new(),
    );
    // To test with a precomputed preprocessed root, change `None` to
    // `Some(privacy_circuit_preprocessed_root())`.
    verify_circuit_proofs(&preprocessed, vec![circuit_proof_1, circuit_proof_2], None);
}
