//! Friendly names for known circuit constants.
//!
//! The flat `Circuit` only stores constant *values*, not the source-level names
//! they were defined with. This module maps the well-known relation-id values
//! (defined as `pub const`s in `circuit_verifier::relations`) back to their
//! source names, so the viewer can label a constant `GATE_RELATION_ID` instead
//! of `378353459`. Names stay in sync with the code because the values are
//! imported from that crate rather than hard-coded.

use std::collections::HashMap;

use circuit_verifier::relations::{
    GATE_RELATION_ID, RANGE_CHECK_16_RELATION_ID, VERIFY_BITWISE_XOR_4_RELATION_ID,
    VERIFY_BITWISE_XOR_7_RELATION_ID, VERIFY_BITWISE_XOR_8_B_RELATION_ID,
    VERIFY_BITWISE_XOR_8_RELATION_ID, VERIFY_BITWISE_XOR_9_RELATION_ID,
    VERIFY_BITWISE_XOR_12_RELATION_ID,
};
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

/// Returns a map from a constant's QM31 value to its source-level name, for the
/// constants we know names for (the logup relation ids).
pub fn named_constants() -> HashMap<QM31, &'static str> {
    let entries: [(u32, &'static str); 8] = [
        (GATE_RELATION_ID, "GATE_RELATION_ID"),
        (RANGE_CHECK_16_RELATION_ID, "RANGE_CHECK_16_RELATION_ID"),
        (VERIFY_BITWISE_XOR_4_RELATION_ID, "VERIFY_BITWISE_XOR_4_RELATION_ID"),
        (VERIFY_BITWISE_XOR_7_RELATION_ID, "VERIFY_BITWISE_XOR_7_RELATION_ID"),
        (VERIFY_BITWISE_XOR_8_RELATION_ID, "VERIFY_BITWISE_XOR_8_RELATION_ID"),
        (VERIFY_BITWISE_XOR_8_B_RELATION_ID, "VERIFY_BITWISE_XOR_8_B_RELATION_ID"),
        (VERIFY_BITWISE_XOR_9_RELATION_ID, "VERIFY_BITWISE_XOR_9_RELATION_ID"),
        (VERIFY_BITWISE_XOR_12_RELATION_ID, "VERIFY_BITWISE_XOR_12_RELATION_ID"),
    ];
    entries.into_iter().map(|(id, name)| (QM31::from(M31::from(id)), name)).collect()
}
