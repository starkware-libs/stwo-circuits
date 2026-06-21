use stwo::core::channel::{Blake2sM31Channel, MerkleChannel};
use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasherGeneric, reduce_to_m31};
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleHasher;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::backend::{BackendForChannel, CpuBackend};

/// A [`MerkleChannel`] that commits with the non-`M31`-reduced Blake2s Merkle hasher,
/// while running Fiat-Shamir through [`Blake2sM31Channel`].
///
/// Blake2s keeps each 32-bit output word in full (as two 16-bit
/// limbs), while [`Blake2sM31Channel`] keeps all challenge and query derivation in the reduced
/// field. This avoids `Blake2sChannel`'s rejection sampling and raw-word query draws
/// (`Channel::draw_u32s`), so the in-circuit Fiat-Shamir logic stays in `M31` exactly as before.
#[derive(Default)]
pub struct MerkleChannelForCircuit;

impl MerkleChannel for MerkleChannelForCircuit {
    type C = Blake2sM31Channel;
    type H = Blake2sMerkleHasher;

    fn mix_root(channel: &mut Self::C, root: Blake2sHash) {
        // Reduce the Merkle root mod `M31::P` before mixing. The
        // root's words can exceed `M31::P`, but the in-circuit channel represents the root
        // as `M31` limbs, so reducing here keeps the out-of-circuit Fiat-Shamir consistent
        // with the in-circuit logic.
        let root = Blake2sHash(reduce_to_m31(root.0));
        // Mix the reduced root into the channel via
        // `Blake2sHasherGeneric::<true>::concat_and_hash`, which reduces the result mod
        // `M31::P`.
        channel
            .update_digest(Blake2sHasherGeneric::<true>::concat_and_hash(&channel.digest(), &root));
    }
}

// `BackendForChannel` is a marker trait; the backends already implement both supertraits for this
// hasher/channel pair (`MerkleOpsLifted<Blake2sMerkleHasher>` and `GrindOps<Blake2sM31Channel>`).
impl BackendForChannel<MerkleChannelForCircuit> for SimdBackend {}
impl BackendForChannel<MerkleChannelForCircuit> for CpuBackend {}
