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

use circuit_verifier::{
    circuit_claim::{CircuitClaim, CircuitInteractionClaim},
    circuit_components::N_COMPONENTS,
};
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
}

impl CairoCircuitClaim {
    pub fn new(claim: &CircuitClaim) -> Self {
        // Destructure positionally — order must match `ComponentList` in
        // `circuit_verifier::circuit_components`. Fixed-size components contribute no log_size
        // and are bound to `_`.
        let CircuitClaim { output_values } = claim;

        Self { output_values: output_values.clone() }
    }
}

/// Mirror of Cairo `CircuitInteractionClaim`.
///
/// Holds the per-component claimed sums in canonical order.
#[derive(Clone, Debug, PartialEq, Eq, CairoSerialize, CairoDeserialize)]
pub struct CairoCircuitInteractionClaim {
    pub claimed_sums: [QM31; N_COMPONENTS],
}

impl From<&CircuitInteractionClaim> for CairoCircuitInteractionClaim {
    fn from(c: &CircuitInteractionClaim) -> Self {
        let CircuitInteractionClaim { claimed_sums } = c;

        Self { claimed_sums: *claimed_sums }
    }
}
