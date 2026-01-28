use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::pcs::PcsConfig;

use crate::cairo_air::statement::{MEMORY_VALUES_LIMBS, OUTPUT_LEN, PROGRAM_LEN, PUBLIC_DATA_LEN};
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
    let flat_claim = vec![M31::zero(); PUBLIC_DATA_LEN];
    let program = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; PROGRAM_LEN];
    let statement = CairoStatement::new(&mut novalue_context, flat_claim, OUTPUT_LEN, program);

    let config = ProofConfig::from_statement(&statement, 20, &pcs_config);

    let empty_proof = empty_proof(&config);

    let proof_vars = empty_proof.guess(&mut novalue_context);
    verify(&mut novalue_context, &proof_vars, &config, &statement);
    novalue_context.finalize_guessed_vars();
    novalue_context.check_vars_used();
    novalue_context.circuit.check_yields();

    println!("Stats: {:?}", novalue_context.stats);
}
