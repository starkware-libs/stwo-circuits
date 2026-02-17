use std::fs::{File, OpenOptions};

use cairo_air::utils::{binary_deserialize_from_file, binary_serialize_to_file};
use num_traits::Zero;
use std::path::PathBuf;
use stwo::core::fields::m31::M31;

use crate::cairo_air::preprocessed_columns::MAX_SEQUENCE_LOG_SIZE;
use crate::cairo_air::statement::{MEMORY_VALUES_LIMBS, PUBLIC_DATA_LEN};
use crate::cairo_air::verify::verify_cairo;
use crate::{
    cairo_air::statement::CairoStatement,
    circuits::{context::Context, ivalue::NoValue, ops::Guess},
    stark_verifier::{
        proof::{ProofConfig, empty_proof},
        verify::verify,
    },
};
use cairo_air::PreProcessedTraceVariant;
use dev_utils::utils::get_compiled_cairo_program_path;
use dev_utils::vm_utils::{ProgramType, run_and_adapt};
use stwo::core::fri::FriConfig;
use stwo::core::{pcs::PcsConfig, vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel};
use stwo_cairo_prover::prover::{ChannelHash, ProverParameters, prove_cairo};

/// Logup security is defined by the `QM31` space (~124 bits) + `INTERACTION_POW_BITS` -
/// log2(number of relation terms).
/// The number of relation terms is defined as n_terms * n_relations * n_uses, where:
/// n_terms = number of terms in each relation (the size of the relation entry) < 2^7,
/// n_relations = number of different relations ids < 2^6,
/// n_uses is bounded by the characteristic of the field = 2^31.
/// E.g. assuming a 100-bit security target, the witness may contain up to
/// 1 << (24 + INTERACTION_POW_BITS) relation terms.
pub const INTERACTION_POW_BITS: u32 = 24;

#[test]
fn test_verify() {
    let mut pcs_config = PcsConfig::default();
    pcs_config.lifting_log_size =
        Some(MAX_SEQUENCE_LOG_SIZE as u32 + pcs_config.fri_config.log_blowup_factor);

    let mut novalue_context = Context::<NoValue>::default();
    let output_len = 1;
    let program_len = 128;
    let flat_claim = vec![M31::zero(); PUBLIC_DATA_LEN + output_len + program_len];
    let outputs = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; output_len];
    let program = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; program_len];
    let statement = CairoStatement::new(&mut novalue_context, flat_claim, outputs, program);

    let config = ProofConfig::from_statement(&statement, &pcs_config, INTERACTION_POW_BITS);

    let empty_proof = empty_proof(&config);

    let proof_vars = empty_proof.guess(&mut novalue_context);
    verify(&mut novalue_context, &proof_vars, &config, &statement);
    novalue_context.finalize_guessed_vars();
    novalue_context.check_vars_used();
    novalue_context.circuit.check_yields();

    println!("Stats: {:?}", novalue_context.stats);
}

pub fn get_proof_file_path(test_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test_data/")
        .join(test_name)
        .join("proof.bin")
}

#[test]
fn test_verify_all_opcodes() {
    let proof_path = get_proof_file_path("all_opcode_components");
    let low_blowup_factor = 1;

    if std::env::var("FIX_PROOF").is_ok() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_all_opcode_components");
        let input = run_and_adapt(&compiled_program, ProgramType::Json, None).unwrap();
        let prover_params = ProverParameters {
            channel_hash: ChannelHash::Blake2sM31,
            pcs_config: PcsConfig {
                pow_bits: 26,
                fri_config: FriConfig::new(0, low_blowup_factor, 70, 1),
                lifting_log_size: Some(20 + low_blowup_factor),
            },
            preprocessed_trace: PreProcessedTraceVariant::CanonicalSmall,
            channel_salt: 0,
            store_polynomials_coefficients: false,
            include_all_preprocessed_columns: true,
        };
        let cairo_proof = prove_cairo::<Blake2sM31MerkleChannel>(input, prover_params).unwrap();

        let proof_file =
            OpenOptions::new().create(true).write(true).truncate(true).open(&proof_path).unwrap();
        binary_serialize_to_file(&cairo_proof, &proof_file).unwrap();
    }

    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();

    let mut context = verify_cairo(&cairo_proof);
    context.check_vars_used();
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
    println!("Stats: {:?}", context.stats);
}

#[test]
fn test_verify_privacy() {
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();

    let mut context = verify_cairo(&cairo_proof);
    context.check_vars_used();
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}
