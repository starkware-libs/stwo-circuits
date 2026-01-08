use crate::circuit_air::components::{eq, qm31_ops};
use crate::circuit_air::statement::CircuitStatement;
use crate::circuit_prover::prover::{CircuitProof, finalize_context, prove_circuit};
use crate::circuits::context::TraceContext;
use crate::circuits::ops::Guess;
use crate::circuits::{context::Context, ops::guess};
use crate::eval;
use crate::stark_verifier::proof::ProofConfig;
use crate::stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use expect_test::expect;
use num_traits::{One, Zero};
use stwo::core::air::Component;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::{CommitmentSchemeVerifier, TreeVec};
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;

// Not a power of 2 so that we can test component padding.
const N: usize = 1030;

pub fn build_fibonacci_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    let (mut a, mut b) = (guess(&mut context, QM31::zero()), guess(&mut context, QM31::one()));
    for _ in 2..N {
        (a, b) = (b, eval!(&mut context, (a) + (b)));
    }

    expect![[r#"
        (809871181 + 0i) + (0 + 0i)u
    "#]]
    .assert_debug_eq(&context.get(b));

    context
}

#[test]
fn test_prove_and_stark_verify_fibonacci_context() {
    let mut fibonacci_context = build_fibonacci_context();
    fibonacci_context.finalize_guessed_vars();
    fibonacci_context.validate_circuit();

    let CircuitProof { components, claim, interaction_claim, pcs_config, stark_proof } =
        prove_circuit(&mut fibonacci_context);
    assert!(stark_proof.is_ok());
    let proof = stark_proof.unwrap();

    // Verify.
    let verifier_channel = &mut Blake2sM31Channel::default();
    let commitment_scheme =
        &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(pcs_config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = TreeVec::concat_cols(components.iter().map(|c| c.trace_log_degree_bounds()));

    commitment_scheme.commit(proof.proof.commitments[0], &sizes[0], verifier_channel);
    claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[1], &sizes[1], verifier_channel);
    // TODO(Gali): Draw interaction element?
    interaction_claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[2], &sizes[2], verifier_channel);
    stwo::core::verifier::verify(
        &components.iter().map(|c| c.as_ref()).collect::<Vec<&dyn Component>>(),
        verifier_channel,
        commitment_scheme,
        proof.proof,
    )
    .unwrap();
}

#[test]
fn test_prove_and_circuit_verify_fibonacci_context() {
    let mut fibonacci_context = build_fibonacci_context();
    fibonacci_context.finalize_guessed_vars();
    fibonacci_context.validate_circuit();

    let CircuitProof { components, claim, interaction_claim, pcs_config, stark_proof } =
        prove_circuit(&mut fibonacci_context);
    assert!(stark_proof.is_ok());
    let proof = stark_proof.unwrap();

    // Verify.
    let config = ProofConfig::new(&components, &pcs_config);

    let mut context = TraceContext::default();
    let proof = proof_from_stark_proof(
        &proof,
        &config,
        claim.log_sizes.to_vec(),
        interaction_claim.claimed_sums.to_vec(),
    );
    let proof_vars = proof.guess(&mut context);
    crate::stark_verifier::verify::verify(
        &mut context,
        &proof_vars,
        &config,
        &CircuitStatement {
            components: vec![
                Box::new(qm31_ops::CircuitQm31OpsComponent {
                    preprocessed_column_indices: [2, 3, 4, 5, 6, 7, 8, 9],
                }),
                Box::new(eq::CircuitEqComponent { preprocessed_column_indices: [0, 1] }),
            ],
        },
    );
    context.check_vars_used();
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_finalize_context() {
    let mut context = build_fibonacci_context();
    finalize_context(&mut context);

    assert!(context.circuit.add.len().is_power_of_two());
    context.validate_circuit();
}
