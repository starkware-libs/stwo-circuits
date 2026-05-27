use circuit_cairo_verifier::privacy::privacy_cairo_verifier_config;
use circuit_common::{finalize::compute_padded_sizes, preprocessed::PreprocessedCircuit};
use circuits::context::Context;
use circuits::ivalue::NoValue;
use stwo::core::pcs::PcsConfig;

use crate::test_utils::{get_pcs_config, get_preprocessed_multiverifier_from_circuit};
use circuit_cairo_verifier::verify::build_cairo_verifier_circuit;

const LOG_BLOWUP_FACTOR: u32 = 3;
const PRIVACY_CAIRO_VERIFIER_TRACE_LOG_SIZE: u32 = 21;
const PCS_CONFIG: PcsConfig =
    get_pcs_config(PRIVACY_CAIRO_VERIFIER_TRACE_LOG_SIZE, LOG_BLOWUP_FACTOR);

fn get_preprocessed_cairo_verifier(
    pcs_config: PcsConfig,
) -> (PreprocessedCircuit, Context<NoValue>) {
    let const_config = privacy_cairo_verifier_config(pcs_config.fri_config.log_blowup_factor);
    let mut novalue_context = build_cairo_verifier_circuit(&const_config);
    let preprocessed_cairo_verifier = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);
    (preprocessed_cairo_verifier, novalue_context)
}

#[test]
#[ignore = "Run after a change to the pcs config to deduce the new required padding."]
fn compare_cairo_and_multiverifier_component_log_sizes() {
    let pcs_config = PCS_CONFIG;
    let (preprocessed_cairo_verifier, novalue_cairo_context) =
        get_preprocessed_cairo_verifier(pcs_config);

    let cairo_component_sizes = compute_padded_sizes(&novalue_cairo_context);
    println!("{:<20}{}", "cairo:", cairo_component_sizes);

    let (_, novalue_multiverifier_context) =
        get_preprocessed_multiverifier_from_circuit(&preprocessed_cairo_verifier, pcs_config);
    let multiverifier_component_sizes = compute_padded_sizes(&novalue_multiverifier_context);
    println!("{:<20}{}", "multiverifier:", multiverifier_component_sizes);
}
