use circuit_cairo_verifier::privacy::privacy_cairo_verifier_config;
use circuit_common::preprocessed::PreprocessedCircuit;
use circuits::context::Context;
use circuits::ivalue::NoValue;
use stwo::core::pcs::PcsConfig;

use crate::{
    padding::pad_components_to_target_log_sizes,
    test_utils::{
        ComponentLogSizes, compute_component_sizes, get_pcs_config,
        get_preprocessed_multiverifier_from_circuit,
    },
};
use circuit_cairo_verifier::verify::build_cairo_verifier_circuit;

const PRIVACY_CAIRO_VERIFIER_TRACE_LOG_SIZE: u32 = 21;
const LOG_BLOWUP_FACTOR: u32 = 3;
const PCS_CONFIG: PcsConfig =
    get_pcs_config(PRIVACY_CAIRO_VERIFIER_TRACE_LOG_SIZE, LOG_BLOWUP_FACTOR);
const TARGET_PADDING_LOG_SIZES: ComponentLogSizes =
    ComponentLogSizes { eq: 17, qm31_ops: 21, m31_to_u32: 18, triple_xor: 17, blake_g_gate: 20 };

fn get_preprocessed_cairo_verifier(
    pcs_config: PcsConfig,
    target_padding: Option<ComponentLogSizes>,
) -> (PreprocessedCircuit, Context<NoValue>) {
    let const_config = privacy_cairo_verifier_config(pcs_config.fri_config.log_blowup_factor);
    let mut novalue_context = build_cairo_verifier_circuit(&const_config);
    if let Some(target_padding) = target_padding {
        pad_components_to_target_log_sizes(&mut novalue_context, target_padding);
    }
    let preprocessed_cairo_verifier = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);
    (preprocessed_cairo_verifier, novalue_context)
}

#[test]
#[ignore = "Run after a change to the pcs config to deduce the new required padding."]
fn compare_cairo_and_multiverifier_component_log_sizes() {
    let pcs_config = PCS_CONFIG;
    let (preprocessed_cairo_verifier, novalue_cairo_context) =
        get_preprocessed_cairo_verifier(pcs_config, None);

    let cairo_component_sizes = compute_component_sizes(&novalue_cairo_context);
    println!("{:<20}{}", "cairo:", cairo_component_sizes);

    let (_, novalue_multiverifier_context) =
        get_preprocessed_multiverifier_from_circuit(&preprocessed_cairo_verifier, pcs_config, None);
    let multiverifier_component_sizes = compute_component_sizes(&novalue_multiverifier_context);
    println!("{:<20}{}", "multiverifier:", multiverifier_component_sizes);
}

#[test]
fn test_padding_is_correct() {
    let pcs_config = PCS_CONFIG;
    let target_padding = TARGET_PADDING_LOG_SIZES;
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
    assert_eq!(
        pp_multiverifier_circuit.params.trace_log_size,
        pp_cairo_circuit.params.trace_log_size
    );
}
