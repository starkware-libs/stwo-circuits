use circuits::blake::{BLAKE2S_DIGEST_N_WORDS, HashValue, blake2s_u32s};
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits::wrappers::U32Wrapper;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use itertools::{Itertools, chain};
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasher};

use crate::circuit_components::{COMPONENT_NAMES, N_COMPONENTS};

/// Number of bytes in `config_words`: one for `log_blowup_factor` plus one per component log size.
const CONFIG_N_BYTES: usize = 1 + N_COMPONENTS;
// `config_bytes` is packed 4 per u32 with no padding, so its length must be a multiple of 4.
const _: () =
    assert!(CONFIG_N_BYTES.is_multiple_of(4), "CONFIG_N_BYTES must pack into whole u32 words");

/// Packs `log_blowup_factor` (byte 0) then each component's preprocessed log size (in
/// [`COMPONENT_NAMES`] order), one byte each, into little-endian u32 words.
fn config_words(
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
    log_blowup_factor: u32,
) -> [u32; CONFIG_N_BYTES / 4] {
    let log_blowup_factor =
        u8::try_from(log_blowup_factor).expect("log_blowup_factor does not fit in a byte");
    let config_bytes: [u8; CONFIG_N_BYTES] = std::array::from_fn(|i| {
        if i == 0 {
            return log_blowup_factor;
        }
        let size = *component_log_sizes.get(COMPONENT_NAMES[i - 1]).unwrap();
        u8::try_from(size).unwrap_or_else(|_| panic!("log_size {size} does not fit in a byte"))
    });
    std::array::from_fn(|i| u32::from_le_bytes(config_bytes[i * 4..i * 4 + 4].try_into().unwrap()))
}

/// Computes the circuit hash: `blake2s(log_blowup_factor || component_log_sizes ||
/// preprocessed_root)`
pub fn compute_circuit_hash<Value: IValue>(
    context: &mut Context<Value>,
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
    log_blowup_factor: u32,
    preprocessed_root: &HashValue<Var>,
) -> HashValue<Var> {
    // Materialize the packed config words as circuit constants.
    let config_words: Vec<U32Wrapper<Var>> = config_words(component_log_sizes, log_blowup_factor)
        .into_iter()
        .map(|word| U32Wrapper::new_unsafe(context.constant(QM31::pack_u32(word))))
        .collect();
    let config_and_root =
        chain!(config_words.iter().copied(), preprocessed_root.iter().copied()).collect_vec();
    let n_bytes = 4 * (config_words.len() + BLAKE2S_DIGEST_N_WORDS);
    blake2s_u32s(context, config_and_root, n_bytes)
}

/// Host (out-of-circuit) twin of [`compute_circuit_hash`]. Used by the prover to mix the circuit
/// hash into the channel.
pub fn compute_circuit_hash_host(
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
    log_blowup_factor: u32,
    preprocessed_root: &HashValue<QM31>,
) -> Blake2sHash {
    let config_words = config_words(component_log_sizes, log_blowup_factor);
    let root_words = preprocessed_root.iter().map(|word| word.get().unpack_u32());

    let mut hasher = Blake2sHasher::new();
    for word in config_words.into_iter().chain(root_words) {
        hasher.update(&word.to_le_bytes());
    }
    hasher.finalize()
}

#[cfg(test)]
mod tests {
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
        let mut config_bytes = vec![log_blowup_factor as u8];
        config_bytes.extend(
            COMPONENT_NAMES.iter().map(|name| *component_log_sizes.get(*name).unwrap() as u8),
        );
        let config_words =
            config_bytes.chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));

        let mut hasher = Blake2s256::new();
        for word in config_words.chain(preprocessed_root) {
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
        let hash =
            compute_circuit_hash(&mut context, component_log_sizes, log_blowup_factor, &root);
        std::array::from_fn(|i| context.get(*hash[i].get()).unpack_u32())
    }

    #[test]
    fn compute_circuit_hash_matches_expected() {
        let sizes = sample_component_log_sizes();
        let root = std::array::from_fn(|i| i as u32);

        let expected = expected_circuit_hash(&sizes, 3, root);
        let in_circuit = circuit_hash_words(&sizes, 3, root);

        let host_hash = compute_circuit_hash_host(&sizes, 3, &HashValue::<QM31>::from(root));
        let host: [u32; BLAKE2S_DIGEST_N_WORDS] = std::array::from_fn(|i| {
            u32::from_le_bytes(host_hash.0[i * 4..i * 4 + 4].try_into().unwrap())
        });

        // The host twin must match an independent reference implementation, and the in-circuit
        // builder must match the host twin (the invariant the Fiat-Shamir transcript relies on).
        assert_eq!(host, expected, "host twin must match the independent reference");
        assert_eq!(in_circuit, host, "in-circuit builder must match the host twin");
    }
}
