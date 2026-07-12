use circuit_verifier::circuit_hash::config_words;
use circuits::blake::BLAKE2S_DIGEST_N_WORDS;
use circuits::utils::le_u32s_from_bytes;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasher};

/// Computes the circuit hash. This is the non-circuit version of
/// [`circuit_verifier::circuit_hash::compute_circuit_hash`]. Used by the prover to mix the circuit
/// hash into the channel.
pub fn compute_circuit_hash(
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
    log_blowup_factor: u32,
    preprocessed_root: Blake2sHash,
) -> Blake2sHash {
    let config_words = config_words(log_blowup_factor, component_log_sizes);
    let root_words: [u32; BLAKE2S_DIGEST_N_WORDS] = le_u32s_from_bytes(preprocessed_root.0);

    let mut hasher = Blake2sHasher::new();
    for word in config_words.into_iter().chain(root_words) {
        hasher.update(&word.to_le_bytes());
    }
    hasher.finalize()
}
