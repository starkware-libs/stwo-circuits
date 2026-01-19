use stwo::core::pcs::PcsConfig;

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

    let statement = CairoStatement::default();

    let config = ProofConfig::from_statement(&statement, 4, &pcs_config);

    let empty_proof = empty_proof(&config);
    let mut novalue_context = Context::<NoValue>::default();
    let proof_vars = empty_proof.guess(&mut novalue_context);
    verify(&mut novalue_context, &proof_vars, &config, &statement);
    novalue_context.finalize_guessed_vars();
    novalue_context.check_vars_used();
    novalue_context.circuit.check_yields();

    println!("Stats: {:?}", novalue_context.stats);
}
