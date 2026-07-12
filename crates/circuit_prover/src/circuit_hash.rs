use circuit_verifier::circuit_hash::config_words;
use circuits::blake::HashValue;
use circuits::ivalue::IValue;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasher};

/// Computes the circuit hash. This is the non-circuit version of
/// [`circuit_verifier::circuit_hash::compute_circuit_hash`]. Used by the prover to mix the circuit
/// hash into the channel.
pub fn compute_circuit_hash(
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
    log_blowup_factor: u32,
    preprocessed_root: &HashValue<QM31>,
) -> Blake2sHash {
    let config_words = config_words(log_blowup_factor, component_log_sizes);
    let root_words = preprocessed_root.iter().map(|word| word.get().unpack_u32());

    let mut hasher = Blake2sHasher::new();
    for word in config_words.into_iter().chain(root_words) {
        hasher.update(&word.to_le_bytes());
    }
    hasher.finalize()
}
