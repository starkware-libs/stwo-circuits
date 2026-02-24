use crate::prover::{CircuitProof, finalize_context, prove_circuit};
use circuit_air::CircuitInteractionElements;
use circuit_air::lookup_sum;
use circuit_air::statement::{CircuitStatement, INTERACTION_POW_BITS};
use circuits::blake::blake;
use circuits::context::{TraceContext, Var};
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{Guess, output, permute};
use circuits::{context::Context, ops::guess};
use circuits_stark_verifier::proof::{Claim, ProofConfig};
use circuits_stark_verifier::proof_from_stark_proof::{
    pack_component_log_sizes, pack_enable_bits, proof_from_stark_proof,
};
use expect_test::expect;
use itertools::Itertools;
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
    output(&mut context, b);

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
    let n_inputs = 9;
    let n_bytes = n_inputs * 16;
    let n_blake_gates = 15;
    for i in 0..n_inputs {
        inputs.push(guess(
            &mut context,
            qm31_from_u32s(4 * i + 82, 4 * i + 83, 4 * i + 84, 4 * i + 85),
        ));
    }
    for _ in 0..n_blake_gates {
        let output = blake(&mut context, &inputs, n_bytes as usize);
        eval!(&mut context, (output.0) + (output.1));
    }

    context
}

#[test]
fn test_prove_and_stark_verify_blake_gate_context() {
    let mut blake_gate_context = build_blake_gate_context();
    blake_gate_context.finalize_guessed_vars();
    blake_gate_context.validate_circuit();

    let CircuitProof {
        components,
        preprocessed_circuit,
        claim,
        interaction_claim,
        pcs_config,
        stark_proof,
        interaction_pow_nonce,
        channel_salt,
    } = prove_circuit(&mut blake_gate_context);
    assert!(stark_proof.is_ok(), "Got error: {}", stark_proof.err().unwrap());
    let proof = stark_proof.unwrap();

    let verifier_channel = &mut Blake2sM31Channel::default();
    verifier_channel.mix_felts(&[channel_salt.into()]);
    pcs_config.mix_into(verifier_channel);
    let commitment_scheme =
        &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(pcs_config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = TreeVec::concat_cols(components.iter().map(|c| c.trace_log_degree_bounds()));

    commitment_scheme.commit(
        proof.proof.commitments[0],
        &preprocessed_circuit.preprocessed_trace.log_sizes(),
        verifier_channel,
    );
    claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[1], &sizes[1], verifier_channel);

    verifier_channel.verify_pow_nonce(INTERACTION_POW_BITS, interaction_pow_nonce);

    verifier_channel.mix_u64(interaction_pow_nonce);
    let interaction_elements = CircuitInteractionElements::draw(verifier_channel);

    interaction_claim.mix_into(verifier_channel);

    commitment_scheme.commit(proof.proof.commitments[2], &sizes[2], verifier_channel);
    stwo::core::verifier::verify_ex(
        &components.iter().map(|c| c.as_ref()).collect::<Vec<&dyn Component>>(),
        verifier_channel,
        commitment_scheme,
        proof.proof,
        true,
    )
    .unwrap();

    assert_eq!(
        lookup_sum(
            &claim,
            &interaction_claim,
            &interaction_elements,
            &preprocessed_circuit.params.output_addresses,
            preprocessed_circuit.params.n_blake_gates
        ),
        QM31::zero()
    );
}

#[test]
fn test_prove_and_stark_verify_permutation_context() {
    let mut permutation_context = build_permutation_context();
    permutation_context.finalize_guessed_vars();
    permutation_context.validate_circuit();

    let CircuitProof {
        pcs_config,
        preprocessed_circuit,
        claim,
        interaction_pow_nonce,
        interaction_claim,
        components,
        stark_proof,
        channel_salt,
    } = prove_circuit(&mut permutation_context);
    assert!(stark_proof.is_ok());
    let proof = stark_proof.unwrap();

    // Verify.
    let verifier_channel = &mut Blake2sM31Channel::default();
    verifier_channel.mix_felts(&[channel_salt.into()]);
    pcs_config.mix_into(verifier_channel);
    let commitment_scheme =
        &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(pcs_config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = TreeVec::concat_cols(components.iter().map(|c| c.trace_log_degree_bounds()));

    commitment_scheme.commit(
        proof.proof.commitments[0],
        &preprocessed_circuit.preprocessed_trace.log_sizes(),
        verifier_channel,
    );
    claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[1], &sizes[1], verifier_channel);
    verifier_channel.verify_pow_nonce(INTERACTION_POW_BITS, interaction_pow_nonce);
    verifier_channel.mix_u64(interaction_pow_nonce);
    let interaction_elements = CircuitInteractionElements::draw(verifier_channel);
    interaction_claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[2], &sizes[2], verifier_channel);
    stwo::core::verifier::verify_ex(
        &components.iter().map(|c| c.as_ref()).collect::<Vec<&dyn Component>>(),
        verifier_channel,
        commitment_scheme,
        proof.proof,
        true,
    )
    .unwrap();

    assert_eq!(
        lookup_sum(
            &claim,
            &interaction_claim,
            &interaction_elements,
            &preprocessed_circuit.params.output_addresses,
            preprocessed_circuit.params.n_blake_gates
        ),
        QM31::zero()
    );
}

#[test]
fn test_prove_and_stark_verify_fibonacci_context() {
    let mut fibonacci_context = build_fibonacci_context();
    fibonacci_context.finalize_guessed_vars();
    fibonacci_context.validate_circuit();

    let CircuitProof {
        pcs_config,
        preprocessed_circuit,
        claim,
        interaction_pow_nonce,
        interaction_claim,
        components,
        stark_proof,
        channel_salt,
    } = prove_circuit(&mut fibonacci_context);
    assert!(stark_proof.is_ok());
    let proof = stark_proof.unwrap();

    // Verify.
    let verifier_channel = &mut Blake2sM31Channel::default();
    verifier_channel.mix_felts(&[channel_salt.into()]);
    pcs_config.mix_into(verifier_channel);
    let commitment_scheme =
        &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(pcs_config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = TreeVec::concat_cols(components.iter().map(|c| c.trace_log_degree_bounds()));

    commitment_scheme.commit(
        proof.proof.commitments[0],
        &preprocessed_circuit.preprocessed_trace.log_sizes(),
        verifier_channel,
    );
    claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[1], &sizes[1], verifier_channel);
    verifier_channel.verify_pow_nonce(INTERACTION_POW_BITS, interaction_pow_nonce);
    verifier_channel.mix_u64(interaction_pow_nonce);
    let interaction_elements = CircuitInteractionElements::draw(verifier_channel);
    interaction_claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[2], &sizes[2], verifier_channel);
    stwo::core::verifier::verify_ex(
        &components.iter().map(|c| c.as_ref()).collect::<Vec<&dyn Component>>(),
        verifier_channel,
        commitment_scheme,
        proof.proof,
        true,
    )
    .unwrap();

    assert_eq!(
        lookup_sum(
            &claim,
            &interaction_claim,
            &interaction_elements,
            &preprocessed_circuit.params.output_addresses,
            preprocessed_circuit.params.n_blake_gates
        ),
        QM31::zero()
    );
}

pub fn verify_circuit(proof: CircuitProof) {
    let CircuitProof {
        pcs_config,
        preprocessed_circuit,
        claim,
        interaction_pow_nonce,
        interaction_claim,
        components: _,
        stark_proof,
        channel_salt,
    } = proof;
    assert!(stark_proof.is_ok());
    let stark_proof = stark_proof.unwrap();

    // Verify.
    let mut context = TraceContext::default();
    let statement = CircuitStatement::new(
        &mut context,
        &preprocessed_circuit.params.output_addresses,
        &claim.output_values,
        preprocessed_circuit.params.n_blake_gates,
        preprocessed_circuit.preprocessed_trace.ids(),
    );
    let claim = Claim {
        packed_enable_bits: pack_enable_bits(
            &claim.log_sizes.iter().map(|log_size| *log_size > 0).collect_vec(),
        ),
        packed_component_log_sizes: pack_component_log_sizes(&claim.log_sizes),
        claimed_sums: interaction_claim.claimed_sums.to_vec(),
    };
    let config = ProofConfig::from_statement(&statement, &pcs_config, INTERACTION_POW_BITS);

    context.enable_assert_eq_on_eval();
    let proof =
        proof_from_stark_proof(&stark_proof, &config, claim, interaction_pow_nonce, channel_salt);
    let proof_vars = proof.guess(&mut context);

    circuits_stark_verifier::verify::verify(&mut context, &proof_vars, &config, &statement);
    context.check_vars_used();
    #[cfg(test)]
    context.finalize_guessed_vars();
    #[cfg(test)]
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_prove_and_circuit_verify_fibonacci_context() {
    let mut fibonacci_context = build_fibonacci_context();
    fibonacci_context.finalize_guessed_vars();
    fibonacci_context.validate_circuit();

    let proof = prove_circuit(&mut fibonacci_context);
    verify_circuit(proof);
}

#[test]
fn test_finalize_context() {
    let mut context = build_fibonacci_context();
    finalize_context(&mut context);

    assert!(context.circuit.add.len().is_power_of_two());
    context.validate_circuit();
}
