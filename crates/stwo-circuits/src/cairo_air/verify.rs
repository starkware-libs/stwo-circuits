use crate::circuits::context::Context;
use cairo_air::air::{CairoClaim, CairoInteractionClaim};
use stwo::core::fields::qm31::QM31;
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
pub fn verify_cairo(_proof: &ExtendedCairoProof<Blake2sM31MerkleHasher>) -> Context<QM31> {
    unimplemented!()
}
