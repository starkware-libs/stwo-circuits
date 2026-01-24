use crate::circuit_air::statement::CircuitStatement;
use crate::circuit_prover::prover::{CircuitProof, finalize_context, prove_circuit};
use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::{IValue, qm31_from_u32s};
use crate::circuits::ops::{Guess, permute};
use crate::circuits::{context::Context, ops::guess};
use crate::eval;
use crate::stark_verifier::proof::{Claim, ProofConfig};
use crate::stark_verifier::proof_from_stark_proof::{
    pack_component_log_sizes, pack_enable_bits, proof_from_stark_proof,
};
use crate::stark_verifier::verify::INTERACTION_POW_BITS;
use expect_test::expect;
use num_traits::{One, Zero};
use stwo::core::air::Component;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::channel::Channel;
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

pub fn build_permutation_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    let a = guess(&mut context, qm31_from_u32s(0, 2, 0, 2));
    let b = guess(&mut context, qm31_from_u32s(1, 1, 1, 1));

    let outputs = permute(&mut context, &[a, b], IValue::sort_by_u_coordinate);
    let _outputs = permute(&mut context, &outputs, IValue::sort_by_u_coordinate);

    context
}

#[test]
fn test_prove_and_stark_verify_permutation_context() {
    let mut permutation_context = build_permutation_context();
    permutation_context.finalize_guessed_vars();
    permutation_context.validate_circuit();

    let CircuitProof {
        pcs_config,
        claim,
        interaction_pow,
        interaction_claim,
        components,
        stark_proof,
    } = prove_circuit(&mut permutation_context);
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
    verifier_channel.verify_pow_nonce(INTERACTION_POW_BITS, interaction_pow);
    verifier_channel.mix_u64(interaction_pow);
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
fn test_prove_and_stark_verify_fibonacci_context() {
    let mut fibonacci_context = build_fibonacci_context();
    fibonacci_context.finalize_guessed_vars();
    fibonacci_context.validate_circuit();

    let CircuitProof {
        pcs_config,
        claim,
        interaction_pow,
        interaction_claim,
        components,
        stark_proof,
    } = prove_circuit(&mut fibonacci_context);
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
    verifier_channel.verify_pow_nonce(INTERACTION_POW_BITS, interaction_pow);
    verifier_channel.mix_u64(interaction_pow);
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

    let CircuitProof {
        pcs_config,
        claim,
        interaction_pow,
        interaction_claim,
        components: _,
        stark_proof,
    } = prove_circuit(&mut fibonacci_context);
    assert!(stark_proof.is_ok());
    let proof = stark_proof.unwrap();

    // Verify.
    let log_trace_size = claim.log_sizes.iter().max().unwrap();
    let statement = CircuitStatement::default();
    let claim = Claim {
        packed_enable_bits: pack_enable_bits(&[true, true]),
        packed_component_log_sizes: pack_component_log_sizes(&claim.log_sizes),
        claimed_sums: interaction_claim.claimed_sums.to_vec(),
        public_claim: vec![],
    };
    let config = ProofConfig::from_statement(
        &statement,
        *log_trace_size as usize,
        claim.public_claim.len(),
        &pcs_config,
    );

    let mut context = TraceContext::default();
    let proof = proof_from_stark_proof(&proof, &config, claim, interaction_pow);
    let proof_vars = proof.guess(&mut context);

    crate::stark_verifier::verify::verify(&mut context, &proof_vars, &config, &statement);
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
