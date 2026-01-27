use cairo_air::PreProcessedTraceVariant;
use dev_utils::utils::get_compiled_cairo_program_path;
use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;
use stwo::core::fri::FriConfig;
use stwo::core::{pcs::PcsConfig, vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel};
use stwo_cairo_prover::prover::{prove_cairo_ex, ChannelHash, ProverParameters};
use stwo_cairo_utils::vm_utils::{run_and_adapt, ProgramType};

use crate::cairo_air::statement::PUBLIC_DATA_LEN;
use crate::{
    cairo_air::{statement::CairoStatement, verify::verify_cairo},
    circuits::{context::Context, ivalue::NoValue, ops::Guess},
    stark_verifier::{
        proof::{empty_proof, ProofConfig},
        verify::verify,
    },
};

#[test]
fn test_verify() {
    let pcs_config = PcsConfig::default();

    let mut novalue_context = Context::<NoValue>::default();
    let flat_claim = vec![NoValue; PUBLIC_DATA_LEN.div_ceil(SECURE_EXTENSION_DEGREE)]
        .guess(&mut novalue_context);
    let statement = CairoStatement::new(&mut novalue_context, flat_claim, PUBLIC_DATA_LEN);

    let config = ProofConfig::from_statement(&statement, 20, &pcs_config);

    let empty_proof = empty_proof(&config);

    let proof_vars = empty_proof.guess(&mut novalue_context);
    verify(&mut novalue_context, &proof_vars, &config, &statement);
    novalue_context.finalize_guessed_vars();
    novalue_context.check_vars_used();
    novalue_context.circuit.check_yields();

    println!("Stats: {:?}", novalue_context.stats);
}

#[test]
fn test_verify_cairo() {
    let compiled_program = get_compiled_cairo_program_path("test_prove_verify_ret_opcode");
    let input = run_and_adapt(&compiled_program, ProgramType::Json, None).unwrap();
    let prover_params = ProverParameters {
        channel_hash: ChannelHash::Blake2s,
        pcs_config: PcsConfig { pow_bits: 26, fri_config: FriConfig::new(0, 1, 70) },
        preprocessed_trace: PreProcessedTraceVariant::Canonical,
        channel_salt: None,
        store_polynomials_coefficients: false,
    };
    let cairo_proof = prove_cairo_ex::<Blake2sM31MerkleChannel>(input, prover_params).unwrap();

    let mut context = verify_cairo(&cairo_proof);
    context.check_vars_used();
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    // context.validate_circuit();
    println!("Stats: {:?}", context.stats);
}
