//! Rust mirror of the Cairo `CircuitVerifierConfig` struct.
//!
//! Field order MUST match
//! `stwo-cairo/stwo_cairo_verifier/crates/circuit_air/src/lib.cairo::CircuitVerifierConfig`.

use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo_cairo_serialize::{CairoDeserialize, CairoSerialize};

#[derive(Clone, Debug, PartialEq, Eq, CairoSerialize, CairoDeserialize)]
pub struct CairoCircuitVerifierConfig {
    /// Variable indices of the circuit's `Output` gates. One entry per public output value.
    pub output_addresses: Vec<u32>,
    /// Number of Blake gates in the circuit.
    pub n_blake_gates: u32,
    /// Expected preprocessed-trace root.
    pub preprocessed_root: Blake2sHash,
    /// Per-column log sizes in the circuit's preprocessed trace, in canonical column order.
    pub preprocessed_column_log_sizes: Vec<u32>,
    /// `trace_log_size + log_blowup_factor`. The rust circuit prover packs this into the
    /// channel via `PcsConfig::mix_into` (in `stwo::core::pcs`), but cairo's `PcsConfig`
    /// has no such field, so the verifier needs it via out-of-band config — analogous to
    /// the rust in-circuit verifier reading it from `ProofConfig.fri.log_trace_size`.
    pub lifting_log_size: u32,
}
