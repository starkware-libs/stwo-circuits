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

use circuit_air::circuit_claim::{CircuitClaim, CircuitInteractionClaim};
use circuit_air::circuit_components::ComponentList;
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
        let CircuitClaim { log_sizes, output_values } = c;
        Self {
            output_values: output_values.clone(),
            eq_log_size: log_sizes[ComponentList::Eq as usize],
            qm31_ops_log_size: log_sizes[ComponentList::Qm31Ops as usize],
            blake_gate_log_size: log_sizes[ComponentList::BlakeGate as usize],
            blake_round_log_size: log_sizes[ComponentList::BlakeRound as usize],
            blake_g_log_size: log_sizes[ComponentList::BlakeG as usize],
            blake_output_log_size: log_sizes[ComponentList::BlakeOutput as usize],
            triple_xor_32_log_size: log_sizes[ComponentList::TripleXor32 as usize],
            m_31_to_u_32_log_size: log_sizes[ComponentList::M31ToU32 as usize],
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
        let s = &c.claimed_sums;
        Self {
            eq: s[ComponentList::Eq as usize],
            qm31_ops: s[ComponentList::Qm31Ops as usize],
            blake_gate: s[ComponentList::BlakeGate as usize],
            blake_round: s[ComponentList::BlakeRound as usize],
            blake_round_sigma: s[ComponentList::BlakeRoundSigma as usize],
            blake_g: s[ComponentList::BlakeG as usize],
            blake_output: s[ComponentList::BlakeOutput as usize],
            triple_xor_32: s[ComponentList::TripleXor32 as usize],
            m_31_to_u_32: s[ComponentList::M31ToU32 as usize],
            verify_bitwise_xor_8: s[ComponentList::VerifyBitwiseXor8 as usize],
            verify_bitwise_xor_12: s[ComponentList::VerifyBitwiseXor12 as usize],
            verify_bitwise_xor_4: s[ComponentList::VerifyBitwiseXor4 as usize],
            verify_bitwise_xor_7: s[ComponentList::VerifyBitwiseXor7 as usize],
            verify_bitwise_xor_9: s[ComponentList::VerifyBitwiseXor9 as usize],
            range_check_15: s[ComponentList::RangeCheck15 as usize],
            range_check_16: s[ComponentList::RangeCheck16 as usize],
        }
    }
}
