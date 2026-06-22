use stwo::core::channel::{Blake2sM31Channel, MerkleChannel};
use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasherGeneric};
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleHasher;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::backend::{BackendForChannel, CpuBackend};

/// A [`MerkleChannel`] that commits with the **standard** (non-`M31`-reduced) Blake2s Merkle
/// hasher, while running Fiat-Shamir through the **`M31`** Blake2s channel.
///
/// All challenge and query derivation in the reduced field.
/// This avoids the standard channel's rejection sampling and raw-word query draws
/// (`Channel::draw_u32s`).
#[derive(Default)]
pub struct MerkleChannelForCircuit;

impl MerkleChannel for MerkleChannelForCircuit {
    type C = Blake2sM31Channel;
    type H = Blake2sMerkleHasher;

    fn mix_root(channel: &mut Self::C, root: Blake2sHash) {
        // Mix the *unreduced* Merkle root into the channel.
        channel
            .update_digest(Blake2sHasherGeneric::<true>::concat_and_hash(&channel.digest(), &root));
    }
}

// `BackendForChannel` is a marker trait; the backends already implement both supertraits for this
// hasher/channel pair (`MerkleOpsLifted<Blake2sMerkleHasher>` and `GrindOps<Blake2sM31Channel>`).
impl BackendForChannel<MerkleChannelForCircuit> for SimdBackend {}
impl BackendForChannel<MerkleChannelForCircuit> for CpuBackend {}
