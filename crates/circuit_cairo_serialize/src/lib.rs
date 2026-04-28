//! Serializes circuit prover output into the format expected by the Cairo verifier
//! (`stwo-cairo/stwo_cairo_verifier/crates/circuit_verifier`).
//!
//! The Cairo verifier's executable signature is:
//!
//! ```cairo
//! fn main(proof: CircuitProof, config: CircuitVerifierConfig) -> VerificationOutput
//! ```
//!
//! and uses the standard `#[derive(Serde)]` to deserialize each argument from the
//! felt252 input stream produced by `scarb execute --arguments-file`. This crate emits
//! the corresponding stream from the Rust prover output.

pub mod claim;
pub mod config;
pub mod proof;

#[cfg(test)]
mod test;

pub use claim::{CairoCircuitClaim, CairoCircuitInteractionClaim};
pub use config::CairoCircuitVerifierConfig;
pub use proof::{CairoCircuitProof, CairoStarkProofForCircuit, prepare_cairo_verifier_input};
