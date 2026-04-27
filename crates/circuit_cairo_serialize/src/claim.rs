//! Owned mirror structs for Cairo `CircuitClaim` and `CircuitInteractionClaim`.
//!
//! These mirror, field-by-field, the structs in
//! `stwo-cairo/stwo_cairo_verifier/crates/circuit_air/src/claims.cairo`. The Cairo `Serde`
//! derive serializes a struct by emitting each field in declaration order; the field
//! order here MUST match the Cairo side exactly. Components with empty `Claim {}` on the
//! Cairo side (fixed-size LOG_SIZE constants) contribute no fields.
//!
//! Both `CairoSerialize` and `CairoDeserialize` are derived, giving symmetric serde so
//! these types can round-trip in tests.

use circuit_verifier::circuit_claim::{CircuitClaim, CircuitInteractionClaim};
use stwo::core::fields::qm31::QM31;
use stwo_cairo_serialize::{CairoDeserialize, CairoSerialize};

/// Mirror of Cairo `CircuitClaim`.
///
/// Cairo layout:
/// - `public_data: CircuitPublicData { output_values: Array<QM31> }`
/// - one `log_size: u32` per variable-size component, in `ComponentList` order
/// - fixed-size components have empty `Claim {}` and contribute zero felts.
#[derive(Clone, Debug, PartialEq, Eq, CairoSerialize, CairoDeserialize)]
pub struct CairoCircuitClaim {
    pub output_values: Vec<QM31>,
    pub eq_log_size: u32,
    pub qm31_ops_log_size: u32,
    pub blake_gate_log_size: u32,
    pub blake_round_log_size: u32,
    pub blake_g_log_size: u32,
    pub blake_output_log_size: u32,
    pub triple_xor_32_log_size: u32,
    pub m_31_to_u_32_log_size: u32,
}

impl From<&CircuitClaim> for CairoCircuitClaim {
    fn from(c: &CircuitClaim) -> Self {
        // Destructure positionally — order must match `ComponentList` in
        // `circuit_verifier::circuit_components`. Fixed-size components contribute no log_size
        // and are bound to `_`.
        let CircuitClaim { log_sizes, output_values } = c;
        let &[
            eq_log_size,
            qm31_ops_log_size,
            blake_gate_log_size,
            blake_round_log_size,
            _blake_round_sigma_log_size,
            blake_g_log_size,
            blake_output_log_size,
            triple_xor_32_log_size,
            m_31_to_u_32_log_size,
            _verify_bitwise_xor_8_log_size,
            _verify_bitwise_xor_12_log_size,
            _verify_bitwise_xor_4_log_size,
            _verify_bitwise_xor_7_log_size,
            _verify_bitwise_xor_9_log_size,
            _range_check_15_log_size,
            _range_check_16_log_size,
        ] = log_sizes;
        Self {
            output_values: output_values.clone(),
            eq_log_size,
            qm31_ops_log_size,
            blake_gate_log_size,
            blake_round_log_size,
            blake_g_log_size,
            blake_output_log_size,
            triple_xor_32_log_size,
            m_31_to_u_32_log_size,
        }
    }
}

/// Mirror of Cairo `CircuitInteractionClaim`.
///
/// Cairo layout: 16 named QM31 fields in `ComponentList` order. The Rust prover stores
/// the same data as `[QM31; 16]`; this struct just gives each entry a name so the derive
/// macro can produce identical felt output.
#[derive(Clone, Debug, PartialEq, Eq, CairoSerialize, CairoDeserialize)]
pub struct CairoCircuitInteractionClaim {
    pub eq: QM31,
    pub qm31_ops: QM31,
    pub blake_gate: QM31,
    pub blake_round: QM31,
    pub blake_round_sigma: QM31,
    pub blake_g: QM31,
    pub blake_output: QM31,
    pub triple_xor_32: QM31,
    pub m_31_to_u_32: QM31,
    pub verify_bitwise_xor_8: QM31,
    pub verify_bitwise_xor_12: QM31,
    pub verify_bitwise_xor_4: QM31,
    pub verify_bitwise_xor_7: QM31,
    pub verify_bitwise_xor_9: QM31,
    pub range_check_15: QM31,
    pub range_check_16: QM31,
}

impl From<&CircuitInteractionClaim> for CairoCircuitInteractionClaim {
    fn from(c: &CircuitInteractionClaim) -> Self {
        // Destructure positionally — order must match `ComponentList` in
        // `circuit_verifier::circuit_components`.
        let &[
            eq,
            qm31_ops,
            blake_gate,
            blake_round,
            blake_round_sigma,
            blake_g,
            blake_output,
            triple_xor_32,
            m_31_to_u_32,
            verify_bitwise_xor_8,
            verify_bitwise_xor_12,
            verify_bitwise_xor_4,
            verify_bitwise_xor_7,
            verify_bitwise_xor_9,
            range_check_15,
            range_check_16,
        ] = &c.claimed_sums;
        Self {
            eq,
            qm31_ops,
            blake_gate,
            blake_round,
            blake_round_sigma,
            blake_g,
            blake_output,
            triple_xor_32,
            m_31_to_u_32,
            verify_bitwise_xor_8,
            verify_bitwise_xor_12,
            verify_bitwise_xor_4,
            verify_bitwise_xor_7,
            verify_bitwise_xor_9,
            range_check_15,
            range_check_16,
        }
    }
}
