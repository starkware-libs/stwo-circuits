use crate::cairo_air::statement::CairoStatement;
use crate::circuits::context::{Context, TraceContext};
use crate::circuits::ops::Guess;
use crate::stark_verifier::proof::{Claim, Proof, ProofConfig};
use crate::stark_verifier::proof_from_stark_proof::{
    pack_component_log_sizes, pack_enable_bits, pack_public_claim, proof_from_stark_proof,
};
use crate::stark_verifier::verify::verify;
use cairo_air::air::{CairoClaim, CairoInteractionClaim};
use cairo_air::combined_claim::CombinedClaim;
use stwo::core::fields::qm31::{QM31, SecureField};
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::vcs_lifted::MerkleHasherLifted;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;

/// [cairo_air::air::CairoProof] with [ExtendedStarkProof] instead of
/// [stwo::core::proof::StarkProof].
// TODO(Gali): Move to stwo_cairo.
pub struct ExtendedCairoProof<H: MerkleHasherLifted> {
    pub claim: CairoClaim,
    pub interaction_pow: u64,
    pub interaction_claim: CairoInteractionClaim,
    pub stark_proof: ExtendedStarkProof<H>,
    /// Optional salt used in the channel initialization.
    pub channel_salt: Option<u64>,
}

/// Circuit Verifies an [ExtendedCairoProof].
// TODO(Gali): Add test.
pub fn verify_cairo(proof: &ExtendedCairoProof<Blake2sM31MerkleHasher>) -> Context<QM31> {
    let statement = CairoStatement::<QM31>::default();

    let (config, proof) = proof_from_cairo_proof(proof, &statement);

    let mut context = TraceContext::default();
    let proof_vars = proof.guess(&mut context);

    verify(&mut context, &proof_vars, &config, &statement);

    context
}

/// Prepares the input for the circuit verifier by converting the [ExtendedCairoProof] to a
/// [ProofConfig] and a [Proof].
pub fn proof_from_cairo_proof(
    proof: &ExtendedCairoProof<Blake2sM31MerkleHasher>,
    statement: &CairoStatement<QM31>,
) -> (ProofConfig, Proof<SecureField>) {
    let ExtendedCairoProof {
        claim,
        // TODO(Gali): Add interaction pow to the config.
        interaction_pow: _,
        interaction_claim,
        stark_proof,
        // TODO(Gali): Add channel salt to the config.
        channel_salt: _,
    } = proof;

    let CombinedClaim {
        component_enable_bits,
        public_claim,
        component_log_sizes,
        component_claimed_sums,
    } = CombinedClaim::from_cairo_claims(claim, interaction_claim);

    let log_trace_size = component_log_sizes.iter().max().unwrap();
    let config = ProofConfig::from_statement(
        statement,
        *log_trace_size as usize,
        public_claim.len(),
        &proof.stark_proof.proof.config,
    );

    let claim = Claim {
        packed_enable_bits: pack_enable_bits(&component_enable_bits),
        packed_component_log_sizes: pack_component_log_sizes(&component_log_sizes),
        claimed_sums: component_claimed_sums,
        public_claim: pack_public_claim(&public_claim),
    };

    let proof = proof_from_stark_proof(stark_proof, &config, claim);
    (config, proof)
}
