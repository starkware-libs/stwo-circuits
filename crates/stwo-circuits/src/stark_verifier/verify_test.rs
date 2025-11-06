use crate::circuits::context::{Context, TraceContext};
use crate::circuits::ivalue::NoValue;
use crate::circuits::ops::Guess;
use crate::examples::simple_air::create_proof;
use crate::stark_verifier::proof::{ProofConfig, empty_proof};
use crate::stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use crate::stark_verifier::verify::verify;

#[test]
fn test_verify() {
    let config = ProofConfig {
        n_proof_of_work_bits: 10,
        n_preprocessed_columns: 1,
        n_trace_columns: 4,
        n_interaction_columns: 4,
    };

    // Create a NoValue version.
    let novalue_circuit = {
        let empty_proof = empty_proof(&config);
        let mut novalue_context = Context::<NoValue>::default();
        let proof_vars = empty_proof.guess(&mut novalue_context);
        verify(&mut novalue_context, &proof_vars, &config);
        novalue_context.circuit
    };

    // Create a context with values from the proof.
    let (_component, proof) = create_proof();
    let mut context = TraceContext::default();
    let proof = proof_from_stark_proof(&proof, &config);
    let proof_vars = proof.guess(&mut context);
    verify(&mut context, &proof_vars, &config);

    // Make sure we got the same circuit.
    assert_eq!(context.circuit, novalue_circuit);

    novalue_circuit.check(context.values()).unwrap();
    println!("Stats: {:?}", context.stats);
}
