use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::pcs::PcsConfig;

use crate::cairo_air::statement::{MEMORY_VALUES_LIMBS, PUB_MEMORY_VALUE_LEN, PUBLIC_DATA_LEN};
use crate::{
    cairo_air::statement::CairoStatement,
    circuits::{context::Context, ivalue::NoValue, ops::Guess},
    stark_verifier::{
        proof::{ProofConfig, empty_proof},
        verify::verify,
    },
};

#[test]
fn test_verify() {
    let pcs_config = PcsConfig::default();

    let mut novalue_context = Context::<NoValue>::default();
    let output_len = 1;
    let program_len = 128;
    let flat_claim =
        vec![M31::zero(); PUBLIC_DATA_LEN + output_len * PUB_MEMORY_VALUE_LEN + program_len];
    let program = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; program_len];
    let statement = CairoStatement::new(&mut novalue_context, flat_claim, output_len, program);

    let config = ProofConfig::from_statement(&statement, 20, &pcs_config);

    let empty_proof = empty_proof(&config);

    let proof_vars = empty_proof.guess(&mut novalue_context);
    verify(&mut novalue_context, &proof_vars, &config, &statement);
    novalue_context.finalize_guessed_vars();
    novalue_context.check_vars_used();
    novalue_context.circuit.check_yields();

    println!("Stats: {:?}", novalue_context.stats);
}
