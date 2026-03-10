use itertools::Itertools;
use std::fs::File;
use std::path::PathBuf;
use stwo::core::fields::m31::M31;
use stwo_cairo_common::prover_types::felt::split_f252;

use crate::statement::MEMORY_VALUES_LIMBS;

use circuit_air::statement::all_circuit_components;
use circuit_air::verify::{CircuitConfig, CircuitPublicData, verify_circuit};
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{CircuitProof, preprare_circuit_proof_for_circuit_verifier};
use circuits::blake::HashValue;
use circuits::context::Context;
use circuits_stark_verifier::proof::{Proof, ProofConfig};
use num_traits::Zero;
use stwo::core::fields::qm31::QM31;

pub fn get_test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test_data/")
}

pub fn get_proof_file_path(test_name: &str) -> PathBuf {
    get_test_data_dir().join(test_name).join("proof.bin")
}

/// Loads a compiled Cairo program from a JSON file and converts each felt252
/// into 28 9-bit M31 limbs using [split_f252].
pub fn load_program(json_path: &std::path::Path) -> Vec<[M31; MEMORY_VALUES_LIMBS]> {
    let json: serde_json::Value = serde_json::from_reader(File::open(json_path).unwrap()).unwrap();
    json["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hex_str| {
            let s = hex_str.as_str().unwrap().strip_prefix("0x").unwrap();
            let padded = format!("{s:0>64}");
            let hi = u128::from_str_radix(&padded[..32], 16).unwrap();
            let lo = u128::from_str_radix(&padded[32..], 16).unwrap();
            let mut words = [0u32; 8];
            for i in 0..4 {
                words[i] = (lo >> (32 * i)) as u32;
                words[i + 4] = (hi >> (32 * i)) as u32;
            }
            split_f252(words)
        })
        .collect_vec()
}

/// Verifies with a circuit a proof of execution of another circuit.
///
/// If `preprocessed_root` is `None`, the verifier takes the preprocessed root from the input proof.
/// This is unsound and is only done to make testing easier.
pub fn verify_circuit_proof(
    preprocessed_circuit: &PreprocessedCircuit,
    circuit_proof: CircuitProof,
    preprocessed_root: Option<HashValue<QM31>>,
) -> Context<QM31> {
    verify_circuit_proofs(preprocessed_circuit, vec![circuit_proof], preprocessed_root)
}

/// Verifies with a circuit a vector of proofs of execution of other circuits.
///
/// If `preprocessed_root` is `None`, the verifier takes the preprocessed root from the first input
/// proof. This is unsound and is only done to make testing easier.
pub fn verify_circuit_proofs(
    preprocessed_circuit: &PreprocessedCircuit,
    circuit_proofs: Vec<CircuitProof>,
    preprocessed_root: Option<HashValue<QM31>>,
) -> Context<QM31> {
    let preprocessed_column_ids = preprocessed_circuit.preprocessed_trace.ids();
    let preprocessed_column_ids_len = preprocessed_column_ids.len();
    let pcs_config = circuit_proofs[0].pcs_config;
    // TODO(Noamp): Add `Eq` trait to `PcsConfig` in stwo to avoid serializing and deserializing.
    let serialized_pcs_config = serde_json::to_string(&pcs_config).unwrap();
    assert!(circuit_proofs.iter().all(|proof| serde_json::to_string(&proof.pcs_config).unwrap() == serialized_pcs_config));
    let mut circuit_config = CircuitConfig {
        config: pcs_config,
        output_addresses: preprocessed_circuit.params.output_addresses.clone(),
        n_blake_gates: preprocessed_circuit.params.n_blake_gates,
        preprocessed_column_ids,
        preprocessed_root: HashValue(QM31::zero(), QM31::zero()),
    };
    let (proofs_vec, public_data_vec): (Vec<Proof<QM31>>, Vec<CircuitPublicData>) = circuit_proofs
        .into_iter()
        .map(|circuit_proof| {
            let proof_config = ProofConfig::from_components(
                &all_circuit_components::<QM31>(),
                preprocessed_column_ids_len,
                &circuit_proof.pcs_config,
                circuit_air::statement::INTERACTION_POW_BITS,
            );
            preprare_circuit_proof_for_circuit_verifier(circuit_proof, proof_config)
        })
        .unzip();
    circuit_config.preprocessed_root = preprocessed_root.unwrap_or(proofs_vec[0].preprocessed_root);
    verify_circuit(circuit_config, proofs_vec, public_data_vec).unwrap()
}
