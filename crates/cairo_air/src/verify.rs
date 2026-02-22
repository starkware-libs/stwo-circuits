use crate::all_components::all_components;
use crate::statement::{CairoStatement, MEMORY_VALUES_LIMBS};
use cairo_air::CairoProof;
use cairo_air::flat_claims::FlatClaim;
use circuits::context::{Context, TraceContext};
use circuits::ops::Guess;
use circuits_stark_verifier::empty_component::EmptyComponent;
use circuits_stark_verifier::proof::{Claim, ProofConfig};
use circuits_stark_verifier::proof_from_stark_proof::{
    pack_component_log_sizes, pack_enable_bits, proof_from_stark_proof,
};
use circuits_stark_verifier::verify::verify;
use itertools::{Itertools, zip_eq};
use std::array;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;

/// Circuit Verifies a [CairoProof].
pub fn verify_cairo(proof: &CairoProof<Blake2sM31MerkleHasher>) -> Result<Context<QM31>, String> {
    let mut context = TraceContext::default();
    let CairoProof {
        claim,
        interaction_pow,
        interaction_claim,
        extended_stark_proof,
        channel_salt,
        preprocessed_trace_variant: _,
    } = proof;

    let FlatClaim { component_enable_bits, component_log_sizes, public_data } =
        claim.flatten_claim();

    let (public_claim, outputs, program) = public_data.pack_into_u32s();

    let public_claim = public_claim.iter().map(|u32| M31::from_u32_unchecked(*u32)).collect_vec();

    debug_assert!(outputs.len().is_multiple_of(MEMORY_VALUES_LIMBS));
    let outputs = outputs
        .chunks_exact(MEMORY_VALUES_LIMBS)
        .map(|chunk| array::from_fn(|i| M31::from_u32_unchecked(chunk[i])))
        .collect_vec();
    debug_assert!(program.len().is_multiple_of(MEMORY_VALUES_LIMBS));
    let program = program
        .chunks_exact(MEMORY_VALUES_LIMBS)
        .map(|chunk| array::from_fn(|i| M31::from_u32_unchecked(chunk[i])))
        .collect_vec();

    let components = zip_eq(all_components().into_values().collect_vec(), &component_enable_bits)
        .map(
            |(component, enable_bit)| {
                if *enable_bit { component } else { Box::new(EmptyComponent {}) }
            },
        )
        .collect_vec();

    let statement =
        CairoStatement::<QM31>::new_ex(&mut context, public_claim, outputs, program, components);
    let config =
        ProofConfig::from_statement(&statement, &proof.extended_stark_proof.proof.config, 24);
    assert_eq!(component_enable_bits.len(), config.n_components);
    let component_claimed_sums = interaction_claim.flatten_interaction_claim();
    assert_eq!(component_claimed_sums.len(), config.n_components);
    let claim = Claim {
        packed_enable_bits: pack_enable_bits(&component_enable_bits),
        packed_component_log_sizes: pack_component_log_sizes(&component_log_sizes),
        claimed_sums: component_claimed_sums,
    };

    let proof_vars = proof_from_stark_proof(
        extended_stark_proof,
        &config,
        claim,
        *interaction_pow,
        *channel_salt,
    )
    .guess(&mut context);

    verify(&mut context, &proof_vars, &config, &statement);

    // Check the verifier circuit gates topology only in test mode.
    #[cfg(test)]
    context.check_vars_used();
    context.finalize_guessed_vars();
    #[cfg(test)]
    context.circuit.check_yields();
    // Always validate the circuit values.
    if !context.is_circuit_valid() {
        return Err("Verification failed".to_string());
    }
    Ok(context)
}
