use crate::circuit_air::CircuitInteractionElements;
use crate::circuit_air::statement::CircuitStatement;
use crate::circuit_prover::prover::{CircuitProof, finalize_context, prove_circuit};
use crate::circuit_prover::witness::components::blake_gate::blake2s_initial_state;
use crate::circuits::blake::blake;
use crate::circuits::context::{TraceContext, Var};
use crate::circuits::ivalue::{IValue, qm31_from_u32s};
use crate::circuits::ops::{Guess, permute};
use crate::circuits::{context::Context, ops::guess};
use crate::eval;
use crate::stark_verifier::proof::ProofConfig;
use crate::stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use expect_test::expect;
use num_traits::{One, Zero};
use stwo::core::air::Component;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::fields::FieldExpOps;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::{CommitmentSchemeVerifier, TreeVec};
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo_constraint_framework::Relation;

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

pub fn build_blake_gate_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();
    context.enable_assert_eq_on_eval();

    let mut inputs: Vec<Var> = vec![];
    let n_inputs = 4;
    let n_bytes = n_inputs * 16;
    let n_blake_gates = 3;
    for _ in 0..n_inputs {
        inputs.push(guess(&mut context, qm31_from_u32s(0, 1, 2, 3)));
    }
    for _ in 0..n_blake_gates {
        let _output = blake(&mut context, &inputs, n_bytes);
    }

    context
}

#[test]
fn test_prove_and_stark_verify_blake_gate_context() {
    let mut blake_gate_context = build_blake_gate_context();
    blake_gate_context.finalize_guessed_vars();
    blake_gate_context.validate_circuit();

    let CircuitProof { components, claim, interaction_claim, pcs_config, stark_proof } =
        prove_circuit(&mut blake_gate_context);
    assert!(stark_proof.is_ok(), "Got error: {}", stark_proof.err().unwrap());
    let stark_proof = stark_proof.unwrap();

    // Compute the expected logup term. In this case it's only the terms corresponding to blake's
    // IV.
    let verifier_channel = &mut Blake2sM31Channel::default();
    let commitment_scheme =
        &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(pcs_config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = TreeVec::concat_cols(components.iter().map(|c| c.trace_log_degree_bounds()));

    commitment_scheme.commit(stark_proof.proof.commitments[0], &sizes[0], verifier_channel);
    claim.mix_into(verifier_channel);
    commitment_scheme.commit(stark_proof.proof.commitments[1], &sizes[1], verifier_channel);
    let CircuitInteractionElements { common_lookup_elements } =
        CircuitInteractionElements::draw(verifier_channel);

    let mut yield_sum: QM31 = QM31::zero();
    let limbs = compute_initial_state_limbs();
    for limb in limbs {
        let denom: QM31 = common_lookup_elements.combine(&limb);
        yield_sum += denom.inverse();
    }

    let total_claim_sum: QM31 = interaction_claim.claimed_sums.iter().sum();
    println!("{:?}", total_claim_sum);
    println!("{:?}", yield_sum);
    println!("{:?}", total_claim_sum - yield_sum);
}

fn compute_initial_state_limbs() -> Vec<[M31; 18]> {
    let state_id = M31::from(1061955672);
    let initial_state = blake2s_initial_state();
    let initial_state_limbs = [
        M31::from(initial_state[0] & 0xffff),
        M31::from((initial_state[0] >> 16) & 0xffff),
        M31::from(initial_state[1] & 0xffff),
        M31::from((initial_state[1] >> 16) & 0xffff),
        M31::from(initial_state[2] & 0xffff),
        M31::from((initial_state[2] >> 16) & 0xffff),
        M31::from(initial_state[3] & 0xffff),
        M31::from((initial_state[3] >> 16) & 0xffff),
        M31::from(initial_state[4] & 0xffff),
        M31::from((initial_state[4] >> 16) & 0xffff),
        M31::from(initial_state[5] & 0xffff),
        M31::from((initial_state[5] >> 16) & 0xffff),
        M31::from(initial_state[6] & 0xffff),
        M31::from((initial_state[6] >> 16) & 0xffff),
        M31::from(initial_state[7] & 0xffff),
        M31::from((initial_state[7] >> 16) & 0xffff),
    ];
    let mut res = vec![];
    for i in 0..3 {
        let mut tmp = vec![];
        tmp.push(state_id);
        tmp.push(M31::from(i));
        tmp.extend_from_slice(&initial_state_limbs);
        res.push(tmp.try_into().unwrap());
    }
    res
}

#[test]
fn test_prove_and_stark_verify_permutation_context() {
    let mut permutation_context = build_permutation_context();
    permutation_context.finalize_guessed_vars();
    permutation_context.validate_circuit();

    let CircuitProof { components, claim, interaction_claim, pcs_config, stark_proof } =
        prove_circuit(&mut permutation_context);
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

    let CircuitProof { components: _, claim, interaction_claim, pcs_config, stark_proof } =
        prove_circuit(&mut fibonacci_context);
    assert!(stark_proof.is_ok());
    let proof = stark_proof.unwrap();

    // Verify.
    let log_trace_size = claim.log_sizes.iter().max().unwrap();
    let statement = CircuitStatement::default();
    let config = ProofConfig::from_statement(&statement, *log_trace_size as usize, &pcs_config);

    let mut context = TraceContext::default();
    let proof = proof_from_stark_proof(
        &proof,
        &config,
        claim.log_sizes.to_vec(),
        interaction_claim.claimed_sums.to_vec(),
    );
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
