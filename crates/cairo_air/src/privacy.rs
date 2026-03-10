use cairo_air::verifier::INTERACTION_POW_BITS;
use circuits::ivalue::NoValue;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::empty_component::EmptyComponent;
use circuits_stark_verifier::proof::ProofConfig;
use itertools::Itertools;
use std::collections::HashSet;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;

use crate::all_components::all_components;
use crate::preprocessed_columns::PREPROCESSED_COLUMNS_ORDER;
use crate::utils::{get_test_data_dir, load_program};
use crate::verify::{CairoVerifierConfig, get_preprocessed_root};

#[cfg(test)]
#[path = "privacy_test.rs"]
pub mod test;

pub const PRIVACY_CAIRO_VERIFIER_CONSTS_HASH: [u32; 8] =
    [1283865202, 2009681603, 1266623912, 534797207, 1489758000, 741307218, 1414461714, 1643656987];

pub const PRIVACY_RECURSION_CIRCUIT_CONSTS_HASH: [u32; 8] =
    [14770276, 1392965202, 52227466, 1742308088, 1732024339, 292075234, 1395568407, 827270248];

pub const PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT: [u32; 8] =
    [263240521, 1367152939, 1051535423, 332158340, 51922189, 1390263238, 1273523373, 1503851977];

/// Returns a fixed [CairoVerifierConfig] for the privacy proof setup.
pub fn privacy_cairo_verifier_config() -> CairoVerifierConfig {
    let privacy_set = privacy_components();
    let components: Vec<Box<dyn CircuitEval<NoValue>>> = all_components::<NoValue>()
        .into_iter()
        .map(|(name, component)| -> Box<dyn CircuitEval<NoValue>> {
            if privacy_set.contains(name) { component } else { Box::new(EmptyComponent {}) }
        })
        .collect_vec();

    let pcs_config = PcsConfig {
        pow_bits: 26,
        fri_config: FriConfig::new(0, 2, 35, 1),
        lifting_log_size: Some(22),
    };

    let proof_config = ProofConfig::from_components(
        &components,
        PREPROCESSED_COLUMNS_ORDER.len(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );

    let program_path = get_test_data_dir().join("privacy/privacy_simple_bootloader_compiled.json");
    let program = load_program(&program_path);

    CairoVerifierConfig {
        preprocessed_root: get_preprocessed_root(22),
        proof_config,
        program,
        n_outputs: 1,
    }
}

// The set of components that are used to verify the privacy transaction.
// The order of the components is determend by the order in all_components()
// TODO(ilya): Fix the privacy bootloader to use the commented out components.
pub fn privacy_components() -> HashSet<&'static str> {
    HashSet::from_iter([
        "add_opcode",
        "add_opcode_small",
        "add_ap_opcode",
        "assert_eq_opcode",
        "assert_eq_opcode_imm",
        "assert_eq_opcode_double_deref",
        "blake_compress_opcode",
        "call_opcode_abs",
        "call_opcode_rel_imm",
        "jnz_opcode_non_taken",
        "jnz_opcode_taken",
        "jump_opcode_abs",
        "jump_opcode_double_deref",
        "jump_opcode_rel",
        "jump_opcode_rel_imm",
        "mul_opcode",
        "mul_opcode_small",
        "ret_opcode",
        "verify_instruction",
        "blake_round",
        "blake_g",
        "blake_round_sigma",
        "triple_xor_32",
        "verify_bitwise_xor_12",
        "bitwise_builtin",
        "pedersen_builtin_narrow_windows",
        "poseidon_builtin",
        "range_check_builtin",
        "pedersen_aggregator_window_bits_9",
        "partial_ec_mul_window_bits_9",
        "pedersen_points_table_window_bits_9",
        "poseidon_aggregator",
        "poseidon_3_partial_rounds_chain",
        "poseidon_full_round_chain",
        "cube_252",
        "poseidon_round_keys",
        "range_check_252_width_27",
        "memory_address_to_id",
        "memory_id_to_big",
        "memory_id_to_small",
        "range_check_6",
        "range_check_8",
        "range_check_11",
        "range_check_12",
        "range_check_18",
        "range_check_20",
        "range_check_4_3",
        "range_check_4_4",
        "range_check_9_9",
        "range_check_7_2_5",
        "range_check_3_6_6_3",
        "range_check_4_4_4_4",
        "range_check_3_3_3_3_3",
        "verify_bitwise_xor_4",
        "verify_bitwise_xor_7",
        "verify_bitwise_xor_8",
        "verify_bitwise_xor_9",
    ])
}
