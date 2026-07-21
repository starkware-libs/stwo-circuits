use blake2::{Blake2s256, Digest};
use circuits::blake::HashValue;
use circuits::context::TraceContext;
use circuits::ivalue::IValue;
use circuits::ops::Guess;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use stwo::core::fields::qm31::QM31;

use super::*;

fn sample_component_log_sizes() -> OrderedHashMap<&'static str, u32> {
    COMPONENT_NAMES.iter().enumerate().map(|(i, name)| (*name, (i % 20) as u32)).collect()
}

/// Independent reference for [`compute_circuit_hash`] using the standard `blake2` crate.
fn expected_circuit_hash(
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
    log_blowup_factor: u32,
    preprocessed_root: [u32; BLAKE2S_DIGEST_N_WORDS],
) -> [u32; BLAKE2S_DIGEST_N_WORDS] {
    let mut hasher = Blake2s256::new();
    for word in
        config_words(log_blowup_factor, component_log_sizes).into_iter().chain(preprocessed_root)
    {
        hasher.update(word.to_le_bytes());
    }
    let digest: [u8; 4 * BLAKE2S_DIGEST_N_WORDS] = hasher.finalize().into();
    std::array::from_fn(|i| u32::from_le_bytes(digest[i * 4..i * 4 + 4].try_into().unwrap()))
}

/// Computes the circuit hash in a fresh context and reads back the eight concrete output words.
fn circuit_hash_words(
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
    log_blowup_factor: u32,
    preprocessed_root: [u32; BLAKE2S_DIGEST_N_WORDS],
) -> [u32; BLAKE2S_DIGEST_N_WORDS] {
    let mut context = TraceContext::default();
    let root = HashValue::<QM31>::from(preprocessed_root).guess(&mut context);
    let hash = compute_circuit_hash(&mut context, component_log_sizes, log_blowup_factor, &root);
    std::array::from_fn(|i| context.get(*hash[i].get()).unpack_u32())
}

#[test]
fn compute_circuit_hash_matches_expected() {
    let sizes = sample_component_log_sizes();
    let root = std::array::from_fn(|i| i as u32);

    let expected = expected_circuit_hash(&sizes, 3, root);
    let in_circuit = circuit_hash_words(&sizes, 3, root);

    // The in-circuit builder must match the independent reference (the invariant the Fiat-Shamir
    // transcript relies on).
    assert_eq!(in_circuit, expected, "in-circuit builder must match the independent reference");
}
