use stwo::core::pcs::PcsConfig;

use crate::cairo_air::statement::{PUBLIC_DATA_LEN, PublicData};
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
    let flat_claim = PublicData::default().pack::<NoValue>().guess(&mut novalue_context);
    let statement = CairoStatement::new(&mut novalue_context, flat_claim, PUBLIC_DATA_LEN);

    let config = ProofConfig::from_statement(&statement, 4, 0, &pcs_config);

    let empty_proof = empty_proof(&config);

    let proof_vars = empty_proof.guess(&mut novalue_context);
    verify(&mut novalue_context, &proof_vars, &config, &statement);
    novalue_context.finalize_guessed_vars();
    novalue_context.check_vars_used();
    novalue_context.circuit.check_yields();

    println!("Stats: {:?}", novalue_context.stats);
}
