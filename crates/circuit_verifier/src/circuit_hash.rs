use circuits::blake::{BLAKE2S_DIGEST_N_WORDS, HashValue, blake2s_u32s};
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits::wrappers::U32Wrapper;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use itertools::{Itertools, chain};
use stwo::core::fields::qm31::QM31;

use crate::circuit_components::{COMPONENT_NAMES, N_COMPONENTS};

/// Number of bytes in `config_words`: one for `log_blowup_factor` plus one per component log size.
const CONFIG_N_BYTES: usize = 1 + N_COMPONENTS;
// `config_bytes` is packed 4 per u32 with no padding, so its length must be a multiple of 4.
const _: () =
    assert!(CONFIG_N_BYTES.is_multiple_of(4), "CONFIG_N_BYTES must pack into whole u32 words");

/// Builds, in-circuit, the circuit hash `blake2s(config_words || preprocessed_root)` — the value
/// that uniquely identifies a circuit.
///
/// `config_words` packs the FRI `log_blowup_factor` followed by each component's preprocessed log
/// size (in [`COMPONENT_NAMES`] order), one byte each, packed 4 per little-endian u32, and is
/// materialized as circuit constants. These words are concatenated with the eight words of
/// `preprocessed_root` (the circuit's guessed preprocessed trace root) and hashed via
/// [`blake2s_u32s`]. `component_log_sizes` is the map returned by
/// [`crate::statement::circuit_component_log_sizes`].
pub fn compute_circuit_hash<Value: IValue>(
    context: &mut Context<Value>,
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
    log_blowup_factor: u32,
    preprocessed_root: &HashValue<Var>,
) -> HashValue<Var> {
    let log_blowup_factor =
        u8::try_from(log_blowup_factor).expect("log_blowup_factor does not fit in a byte");
    // Byte 0 is `log_blowup_factor`; byte `i + 1` is the log size of `COMPONENT_NAMES[i]`.
    let config_bytes: [u8; CONFIG_N_BYTES] = std::array::from_fn(|i| {
        if i == 0 {
            return log_blowup_factor;
        }
        let size = *component_log_sizes.get(COMPONENT_NAMES[i - 1]).unwrap();
        u8::try_from(size).unwrap_or_else(|_| panic!("log_size {size} does not fit in a byte"))
    });
    // Materialize the packed config words as circuit constants.
    let config_words: Vec<U32Wrapper<Var>> = config_bytes
        .chunks_exact(4)
        .map(|bytes| {
            let packed = u32::from_le_bytes(bytes.try_into().unwrap());
            U32Wrapper::new_unsafe(context.constant(QM31::pack_u32(packed)))
        })
        .collect();
    let config_and_root =
        chain!(config_words.iter().copied(), preprocessed_root.iter().copied()).collect_vec();
    let n_bytes = 4 * (config_words.len() + BLAKE2S_DIGEST_N_WORDS);
    blake2s_u32s(context, config_and_root, n_bytes)
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

    /// Independent, out-of-circuit reference for [`compute_circuit_hash`]: a plain `blake2s` over
    /// `config_words || preprocessed_root`, using the standard `blake2` crate.
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
        assert_eq!(circuit_hash_words(&sizes, 3, root), expected_circuit_hash(&sizes, 3, root));
    }
}
