use crate::all_components::all_components;
use crate::preprocessed_columns::PREPROCESSED_COLUMNS_ORDER;
use crate::statement::{CairoStatement, MEMORY_VALUES_LIMBS};
use cairo_air::CairoProof;
use cairo_air::air::PublicData;
use cairo_air::flat_claims::FlatClaim;
use circuits::context::{Context, TraceContext};
use circuits::ops::Guess;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::empty_component::EmptyComponent;
use circuits_stark_verifier::proof::{Claim, Proof, ProofConfig};
use circuits_stark_verifier::proof_from_stark_proof::{
    pack_component_log_sizes, pack_enable_bits, proof_from_stark_proof,
};
use circuits_stark_verifier::verify::verify;
use itertools::{Itertools, zip_eq};
use std::array;
use std::collections::HashSet;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;

pub struct CairoVerifierConfig {
    pub proof_config: ProofConfig,
    pub n_outputs: usize,
    pub program_len: usize,
}

/// Verifies a [CairoProof] for a fixed [CairoVerifierConfig].
pub fn verify_fixed_cairo_circuit(
    verifier_config: CairoVerifierConfig,
    proof: Proof<QM31>,
    public_data: PublicData,
) -> Result<Context<QM31>, String> {
    let mut context = TraceContext::default();

    let config = verifier_config.proof_config;

    let components = zip_eq(all_components().into_values(), config.enabled_components())
        .map(
            |(component, enable_bit)| {
                if enable_bit { component } else { Box::new(EmptyComponent {}) }
            },
        )
        .collect_vec();

    let (public_claim, outputs, program) = public_data.pack_into_u32s();
    if outputs.len() != verifier_config.n_outputs * MEMORY_VALUES_LIMBS {
        return Err("The proof claim does not match the expected number of outputs.".to_string());
    }
    if program.len() != verifier_config.program_len * MEMORY_VALUES_LIMBS {
        return Err("The proof claim does not match the expected program length.".to_string());
    }

    let public_claim = public_claim.iter().map(|u32| M31::from_u32_unchecked(*u32)).collect_vec();

    let outputs = outputs
        .chunks_exact(MEMORY_VALUES_LIMBS)
        .map(|chunk| array::from_fn(|i| M31::from_u32_unchecked(chunk[i])))
        .collect_vec();
    let program = program
        .chunks_exact(MEMORY_VALUES_LIMBS)
        .map(|chunk| array::from_fn(|i| M31::from_u32_unchecked(chunk[i])))
        .collect_vec();

    let statement =
        CairoStatement::<QM31>::new_ex(&mut context, public_claim, outputs, program, components);

    let proof_vars = proof.guess(&mut context);
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

/// Circuit Verifies a [CairoProof].
pub fn verify_cairo(proof: &CairoProof<Blake2sM31MerkleHasher>) -> Result<Context<QM31>, String> {
    let FlatClaim { component_enable_bits, component_log_sizes: _, public_data: _ } =
        proof.claim.flatten_claim();

    let components = HashSet::from_iter(
        zip_eq(all_components::<QM31>().into_keys(), &component_enable_bits)
            .filter(|(_, enable_bit)| **enable_bit)
            .map(|(component_name, _)| component_name),
    );

    verify_cairo_with_component_set(proof, components)
}

/// Verifies a [CairoProof] with a given set of components.
pub fn verify_cairo_with_component_set(
    proof: &CairoProof<Blake2sM31MerkleHasher>,
    component_set: HashSet<&str>,
) -> Result<Context<QM31>, String> {
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

    let components: Vec<Box<dyn CircuitEval<QM31>>> =
        zip_eq(all_components::<QM31>().into_iter(), &component_enable_bits)
            .map(|((component_name, component), &enable_bit)| {
                let component_in_set = component_set.contains(component_name);
                if component_in_set != enable_bit {
                    return Err(format!(
                        "Proof was produced with the wrong components set: expected the component '{}' to be {}abled according to the component set, but it is {}abled in the proof.",
                        component_name,
                        if component_in_set { "en" } else { "dis" },
                        if enable_bit { "en" } else { "dis" }
                    ));
                }
                Ok(if enable_bit { component } else { Box::new(EmptyComponent {}) })
            })
            .try_collect()?;

    let proof_config = ProofConfig::from_components(
        &components,
        PREPROCESSED_COLUMNS_ORDER.len(),
        &proof.extended_stark_proof.proof.config,
        24,
    );
    let verifier_config = CairoVerifierConfig {
        proof_config,
        n_outputs: claim.public_data.public_memory.output.len(),
        program_len: claim.public_data.public_memory.program.len(),
    };

    let component_claimed_sums = interaction_claim.flatten_interaction_claim();
    assert_eq!(component_claimed_sums.len(), verifier_config.proof_config.n_components);
    let claim = Claim {
        packed_enable_bits: pack_enable_bits(&component_enable_bits),
        packed_component_log_sizes: pack_component_log_sizes(&component_log_sizes),
        claimed_sums: component_claimed_sums,
    };
    let proof = proof_from_stark_proof(
        extended_stark_proof,
        &verifier_config.proof_config,
        claim,
        *interaction_pow,
        *channel_salt,
    );

    verify_fixed_cairo_circuit(verifier_config, proof, public_data)
}
