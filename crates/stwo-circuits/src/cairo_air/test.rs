use stwo::core::pcs::PcsConfig;

use crate::{
    cairo_air::{components, statement::CairoStatement},
    circuits::{context::Context, ivalue::NoValue, ops::Guess},
    stark_verifier::{
        constraint_eval::CircuitEval,
        proof::{ProofConfig, empty_proof},
        verify::verify,
    },
};

#[test]
fn test_verify() {
    let pcs_config = PcsConfig::default();
    let components = vec![
        Box::new(components::jnz_opcode_taken::Component {}) as Box<dyn CircuitEval<NoValue>>,
        Box::new(components::jnz_opcode_non_taken::Component {}) as Box<dyn CircuitEval<NoValue>>,
    ];

    let config = ProofConfig::from_circuit_components(&components, 0, 4, &pcs_config);

    let empty_proof = empty_proof(&config);
    let mut novalue_context = Context::<NoValue>::default();
    let proof_vars = empty_proof.guess(&mut novalue_context);
    verify(&mut novalue_context, &proof_vars, &config, &CairoStatement { components });
    novalue_context.finalize_guessed_vars();
    novalue_context.check_vars_used();
    println!("Stats: {:?}", novalue_context.stats);
}
