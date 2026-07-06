use circuit_common::N_RESERVED;
use circuit_common::finalize::pad_to_targets;
use circuit_serialize::deserialize::deserialize_proof_with_config;
use circuit_verifier::statement::{INTERACTION_POW_BITS, all_circuit_components};
use circuits::blake::HashValue;
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

/// Constants pinning a multiverifier proof produced by and older version of the prover/circuit.
/// The root below is that version's multiverifier
/// preprocessed root (i.e. the value `MULTIVERIFIER_PREPROCESSED_ROOT` held at HEAD~1).
const BACKWARD_COMPATABILIT_PREPROCESSED_ROOT: [u32; 8] =
    [4268871180, 1648605015, 1518856044, 936813334, 8391980, 3571729286, 3315525509, 1034558230];
/// The preprocessed root of the old Cairo verifier.
const BACKWARD_COMPATABILIT_CAIRO_VERIFIER_PREPROCESSED_ROOT: [u32; 8] =
    [4124328080, 2746910001, 1609873110, 504894878, 729742955, 1821617411, 3413982230, 3387905857];
const BACKWARD_COMPATABILIT_PROOF_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test_data/circuit_multiverifier/backward_compatibility_proof.bin"
);

/// Backward-compatibility regression test.
///
/// Checks that the *current* multiverifier can verify, in the same circuit, both a multiverifier
/// proof produced by an older version of the prover/circuit (slot 0) and a multiverifier proof
/// produced by the current version (slot 1).
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

    // Both the old and the current multiverifier proofs verified two identical Cairo verifier
    // inputs. Each such input is the eight-word Cairo verifier preprocessed root followed by the
    // (unchanged) Cairo verifier output values unpacked into u32 words. Reconstruct the unreduced
    // Blake2s digest each multiverifier wrote into its output (mirroring
    // `build_multiverifier_circuit`). The two proofs differ only in the Cairo verifier root they
    // hashed, so parameterize the reconstruction by that root.
    let mv_output_values = |cairo_root: [u32; 8]| -> [QM31; N_RESERVED] {
        let cairo_input_preimage_words: Vec<u32> = cairo_root
            .into_iter()
            .chain(
                PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES
                    .iter()
                    .flat_map(|&w| [w & 0xFFFF, w >> 16, 0, 0]),
            )
            .collect();
        let payload_words: Vec<u32> =
            chain!(&cairo_input_preimage_words, &cairo_input_preimage_words).copied().collect();
        HashValue::<QM31>::from(blake2s_u32s_host(&payload_words)).0.map(|w| *w.get())
    };

    // Slot 0: the old multiverifier proof (backward-compatibility fixture), deserialized with the
    // current proof config and presented with its original preprocessed root.
    let old_bytes = std::fs::read(BACKWARD_COMPATABILIT_PROOF_PATH).unwrap();
    let old_proof =
        deserialize_proof_with_config(&mut old_bytes.as_slice(), &shared_config.proof_config)
            .unwrap();
    let old_input = MultiverifierInput {
        proof: old_proof,
        output_values: mv_output_values(BACKWARD_COMPATABILIT_CAIRO_VERIFIER_PREPROCESSED_ROOT),
        preprocessed_root: BACKWARD_COMPATABILIT_PREPROCESSED_ROOT.into(),
    };

    // Slot 1: the current multiverifier proof, with the current multiverifier preprocessed root.
    let current_bytes = std::fs::read(MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH).unwrap();
    let current_proof =
        deserialize_proof_with_config(&mut current_bytes.as_slice(), &shared_config.proof_config)
            .unwrap();
    let current_input = MultiverifierInput {
        proof: current_proof,
        output_values: mv_output_values(PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT),
        preprocessed_root: MULTIVERIFIER_PREPROCESSED_ROOT.into(),
    };

    // Verifying both the old and the current proof inside a current multiverifier must succeed.
    let mut context = build_multiverifier_circuit::<QM31>(old_input, current_input, &shared_config);
    pad_to_targets(&mut context, TARGET_PADDING_SIZES);
    context.validate_circuit();
}
