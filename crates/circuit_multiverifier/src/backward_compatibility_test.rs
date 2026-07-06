use circuit_common::N_RESERVED;
use circuit_common::finalize::pad_to_targets;
use circuit_serialize::deserialize::deserialize_proof_with_config;
use circuit_verifier::statement::{INTERACTION_POW_BITS, all_circuit_components};
use circuits::blake::HashValue;
use circuits::ivalue::IValue;
use circuits_stark_verifier::proof::ProofConfig;
use itertools::chain;
use stwo::core::fields::qm31::QM31;

use crate::test_utils::{
    CIRCUIT_N_PREPROCESSED_COLUMNS, MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH,
    MULTIVERIFIER_PREPROCESSED_ROOT, PCS_CONFIG, PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES,
    PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT, TARGET_PADDING_SIZES, blake2s_u32s_host,
    multiverifier_preprocessed_column_log_sizes,
};
use crate::verify::{MultiverifierInput, SharedConfig, build_multiverifier_circuit};

/// The preprocessed root of the Cairo verifier circuit as it was at commit 9ab9447.
const BACKWARD_COMPATABILIT_CAIRO_VERIFIER_PREPROCESSED_ROOT: [u32; 8] =
    [4124328080, 2746910001, 1609873110, 504894878, 729742955, 1821617411, 3413982230, 3387905857];
/// An older Cairo verifier proof, produced at commit 9ab9447 (that version's `proof_cairo.bin`).
const BACKWARD_COMPATABILIT_CAIRO_PROOF_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test_data/circuit_multiverifier/backward_compatibility_cairo_proof.bin"
);

/// Backward-compatibility regression test.
///
/// Checks that the *current* multiverifier can verify, in the same circuit, both a Cairo verifier
/// proof produced by an older version of the prover/circuit (commit 9ab9447)
/// in slot 0, and a multiverifier proof produced by the current version in slot 1.
///
/// `build_multiverifier_circuit::<QM31>` executes the in-circuit STARK verifier with concrete
/// values and `validate_circuit` then checks every gate constraint holds, so if either proof
/// failed to verify (e.g. a Merkle root mismatch) this test would fail.
#[test]
fn test_backward_compatibility() {
    let shared_config = SharedConfig {
        pcs_config: PCS_CONFIG,
        proof_config: ProofConfig::new(
            &all_circuit_components::<QM31>(),
            CIRCUIT_N_PREPROCESSED_COLUMNS,
            &PCS_CONFIG,
            INTERACTION_POW_BITS,
        ),
        preprocessed_column_log_sizes: multiverifier_preprocessed_column_log_sizes(),
    };

    // Slot 0: the older Cairo verifier proof (backward-compatibility fixture), deserialized with
    // the current proof config.
    let old_bytes = std::fs::read(BACKWARD_COMPATABILIT_CAIRO_PROOF_PATH).unwrap();
    let old_cairo_proof =
        deserialize_proof_with_config(&mut old_bytes.as_slice(), &shared_config.proof_config)
            .unwrap();
    let old_input = MultiverifierInput {
        proof: old_cairo_proof,
        output_values: PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES.map(QM31::pack_u32),
        preprocessed_root: BACKWARD_COMPATABILIT_CAIRO_VERIFIER_PREPROCESSED_ROOT.into(),
    };

    // Slot 1: the current multiverifier proof.
    let cairo_input_preimage_words: Vec<u32> = PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT
        .into_iter()
        .chain(
            PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES.iter().flat_map(|&w| [w & 0xFFFF, w >> 16, 0, 0]),
        )
        .collect();
    let payload_words: Vec<u32> =
        chain!(&cairo_input_preimage_words, &cairo_input_preimage_words).copied().collect();
    let current_output_values: [QM31; N_RESERVED] =
        HashValue::<QM31>::from(blake2s_u32s_host(&payload_words)).0.map(|w| *w.get());

    let current_bytes = std::fs::read(MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH).unwrap();
    let current_proof =
        deserialize_proof_with_config(&mut current_bytes.as_slice(), &shared_config.proof_config)
            .unwrap();
    let current_input = MultiverifierInput {
        proof: current_proof,
        output_values: current_output_values,
        preprocessed_root: MULTIVERIFIER_PREPROCESSED_ROOT.into(),
    };

    // Verifying both the old Cairo verifier proof and the current multiverifier proof inside a
    // current multiverifier must succeed.
    let mut context = build_multiverifier_circuit::<QM31>(old_input, current_input, &shared_config);
    pad_to_targets(&mut context, TARGET_PADDING_SIZES);
    context.validate_circuit();
}
