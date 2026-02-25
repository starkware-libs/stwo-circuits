use crate::all_components::all_components;
use crate::statement::{CairoStatement, MEMORY_VALUES_LIMBS};
use cairo_air::CairoProof;
use cairo_air::air::PublicData;
use cairo_air::flat_claims::FlatClaim;
use circuits::context::{Context, TraceContext};
use circuits::ops::Guess;
use circuits_stark_verifier::empty_component::EmptyComponent;
use circuits_stark_verifier::proof::{Claim, Proof, ProofConfig};
use circuits_stark_verifier::proof_from_stark_proof::{
    pack_component_log_sizes, pack_enable_bits, proof_from_stark_proof,
};
use circuits_stark_verifier::verify::verify;
use itertools::{Itertools, zip_eq};
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;

/// Logup security is defined by the `QM31` space (~124 bits) + `INTERACTION_POW_BITS` -
/// log2(number of relation terms).
/// The number of relation terms is defined as n_terms * n_relations * n_uses, where:
/// n_terms = number of terms in each relation (the size of the relation entry) < 2^7,
/// n_relations = number of different relations ids < 2^6,
/// n_uses is bounded by the characteristic of the field = 2^31.
/// E.g. assuming a 100-bit security target, the witness may contain up to
/// 1 << (24 + INTERACTION_POW_BITS) relation terms.
pub const INTERACTION_POW_BITS: u32 = 24;

pub struct CairoVerifierConfig {
    pub proof_config: ProofConfig,
    pub program: Vec<[M31; MEMORY_VALUES_LIMBS]>,
    pub n_outputs: usize,
}

/// Verifies a [CairoProof] for a fixed [CairoVerifierConfig].
pub fn verify_fixed_cairo_circuit(
    verifier_config: CairoVerifierConfig,
    proof: Proof<QM31>,
    public_claim: Vec<u32>,
    outputs: Vec<[M31; MEMORY_VALUES_LIMBS]>,
) -> Result<Context<QM31>, String> {
    if outputs.len() != verifier_config.n_outputs {
        return Err("The proof claim does not match the expected number of outputs.".to_string());
    }

    let config = verifier_config.proof_config;
    let components = zip_eq(all_components().into_values(), config.enabled_components())
        .map(
            |(component, enable_bit)| {
                if enable_bit { component } else { Box::new(EmptyComponent {}) }
            },
        )
        .collect_vec();

    let public_claim = public_claim.iter().map(|u32| M31::from(*u32)).collect_vec();
    let mut context = TraceContext::default();
    let statement = CairoStatement::<QM31>::new_ex(
        &mut context,
        public_claim,
        outputs,
        verifier_config.program,
        components,
    );

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

/// Converts a [CairoProof] to a [Proof] and [PublicData] for the circuit verifier.
pub fn prepare_cairo_proof_for_circuit_verifier(
    proof: &CairoProof<Blake2sM31MerkleHasher>,
    proof_config: &ProofConfig,
) -> (Proof<QM31>, PublicData) {
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
    let component_claimed_sums = interaction_claim.flatten_interaction_claim();

    debug_assert_eq!(component_enable_bits.len(), proof_config.n_components);
    debug_assert_eq!(component_claimed_sums.len(), proof_config.n_components);

    let claim = Claim {
        packed_enable_bits: pack_enable_bits(&component_enable_bits),
        packed_component_log_sizes: pack_component_log_sizes(&component_log_sizes),
        claimed_sums: component_claimed_sums,
    };

    let proof = proof_from_stark_proof(
        extended_stark_proof,
        proof_config,
        claim,
        *interaction_pow,
        *channel_salt,
    );

    (proof, public_data)
}
