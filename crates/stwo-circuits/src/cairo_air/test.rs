use num_traits::Zero;
use stwo::core::fields::m31::M31;

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
use stwo::core::fri::FriConfig;
use stwo::core::{pcs::PcsConfig, vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel};
use stwo_cairo_prover::prover::{ChannelHash, ProverParameters, prove_cairo};
use stwo_cairo_utils::vm_utils::{ProgramType, run_and_adapt};

#[test]
fn test_verify() {
    let pcs_config = PcsConfig::default();

    let mut novalue_context = Context::<NoValue>::default();
    let output_len = 1;
    let program_len = 128;
    let flat_claim = vec![M31::zero(); PUBLIC_DATA_LEN + output_len + program_len];
    let outputs = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; output_len];
    let program = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; program_len];
    let statement = CairoStatement::new(&mut novalue_context, flat_claim, outputs, program);

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
    let compiled_program =
        get_compiled_cairo_program_path("test_prove_verify_all_opcode_components");
    let input = run_and_adapt(&compiled_program, ProgramType::Json, None).unwrap();
    let prover_params = ProverParameters {
        channel_hash: ChannelHash::Blake2s,
        pcs_config: PcsConfig { pow_bits: 26, fri_config: FriConfig::new(0, 1, 70) },
        preprocessed_trace: PreProcessedTraceVariant::CanonicalWithoutPedersen,
        channel_salt: 0,
        store_polynomials_coefficients: false,
    };

    let cairo_proof = prove_cairo::<Blake2sM31MerkleChannel>(input, prover_params).unwrap();

    let mut context = verify_cairo(&cairo_proof);
    context.check_vars_used();
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
    println!("Stats: {:?}", context.stats);
}
