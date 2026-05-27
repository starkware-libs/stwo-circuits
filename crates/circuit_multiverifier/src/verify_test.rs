use circuit_cairo_verifier::privacy::{get_pcs_config, privacy_cairo_verifier_config};
use circuit_common::{
    finalize::{ComponentSizes, compute_padded_sizes, pad_to_targets},
    preprocessed::PreprocessedCircuit,
};
use circuits::context::FinalizedContext;
use circuits::ivalue::NoValue;
use stwo::core::pcs::PcsConfig;

use crate::test_utils::get_preprocessed_multiverifier_from_circuit;
use circuit_cairo_verifier::verify::build_cairo_verifier_circuit;

const PRIVACY_CAIRO_VERIFIER_TRACE_LOG_SIZE: u32 = 21;
const LOG_BLOWUP_FACTOR: u32 = 3;
const PCS_CONFIG: PcsConfig =
    get_pcs_config(PRIVACY_CAIRO_VERIFIER_TRACE_LOG_SIZE, LOG_BLOWUP_FACTOR);
const TARGET_PADDING_SIZES: ComponentSizes = ComponentSizes {
    eq: 1 << 17,
    qm31_ops: 1 << 21,
    m31_to_u32: 1 << 18,
    triple_xor: 1 << 17,
    blake_g_gate: 1 << 20,
};

/// Builds a `NoValue` Cairo verifier circuit (with configs of privacy) and preprocesses it.
///
/// If `target_padding` is `None`, we don't pad the circuits (note that even in this case, the
/// default padding to the next power of 2 happens in [`PreprocessedCircuit::preprocess_circuit`]).
fn get_preprocessed_cairo_verifier(
    pcs_config: PcsConfig,
    target_padding: Option<ComponentSizes>,
) -> (PreprocessedCircuit, FinalizedContext<NoValue>) {
    let const_config = privacy_cairo_verifier_config(pcs_config.fri_config.log_blowup_factor);
    let mut novalue_context = build_cairo_verifier_circuit(&const_config);
    if let Some(target_padding) = target_padding {
        pad_to_targets(&mut novalue_context, target_padding);
    }
    let preprocessed_cairo_verifier = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);
    (preprocessed_cairo_verifier, novalue_context)
}

/// Regression test on the padded sizes of the Cairo verifier and the multiverifier circuits.
///
/// Builds both circuits (unpadded) for `PCS_CONFIG` and prints their `compute_padded_sizes`
/// breakdowns side by side. For the multiverifier to be able to verify executions of the cairo
/// verifier and itself without changing, the sizes of the AIR components for both circuits must be
/// the same. This is achieved by padding each component to the next power of two of the max between
/// the component's size in the two AIRs (note that by doing this we may end up padding both
/// circuits). This test is needed to deduce the target padding whenever the `PCS_CONFIG` changes.
#[test]
fn test_regression_cairo_and_multiverifier_component_log_sizes() {
    let pcs_config = PCS_CONFIG;
    let (preprocessed_cairo_verifier, novalue_cairo_context) =
        get_preprocessed_cairo_verifier(pcs_config, None);

    let cairo_component_sizes = compute_padded_sizes(&novalue_cairo_context);
    let expected_sizes = ComponentSizes {
        eq: 1 << 17,
        qm31_ops: 1 << 21,
        m31_to_u32: 1 << 18,
        triple_xor: 1 << 17,
        blake_g_gate: 1 << 20,
    };
    assert_eq!(cairo_component_sizes, expected_sizes);

    let (_, novalue_multiverifier_context) =
        get_preprocessed_multiverifier_from_circuit(&preprocessed_cairo_verifier, pcs_config, None);
    let multiverifier_component_sizes = compute_padded_sizes(&novalue_multiverifier_context);
    let expected_sizes = ComponentSizes {
        eq: 1 << 15,
        qm31_ops: 1 << 21,
        m31_to_u32: 1 << 18,
        triple_xor: 1 << 17,
        blake_g_gate: 1 << 20,
    };
    assert_eq!(multiverifier_component_sizes, expected_sizes);
}

#[test]
fn test_padding_is_correct() {
    let pcs_config = PCS_CONFIG;
    let target_padding = TARGET_PADDING_SIZES;
    let (pp_cairo_circuit, _) =
        get_preprocessed_cairo_verifier(pcs_config, Some(target_padding.clone()));
    let (pp_multiverifier_circuit, _) = get_preprocessed_multiverifier_from_circuit(
        &pp_cairo_circuit,
        pcs_config,
        Some(target_padding),
    );
    assert_eq!(
        pp_multiverifier_circuit.preprocessed_trace.ids(),
        pp_cairo_circuit.preprocessed_trace.ids()
    );
    assert_eq!(pp_multiverifier_circuit.trace_log_size, pp_cairo_circuit.trace_log_size);
}
