//! Run-once motif catalog.
//!
//! The principle: a library function's *role assignment* (which of its variables
//! are genuine inputs vs. prover witnesses) is part of its signature. Rather than
//! guessing that from graph heuristics, we learn it **definitionally** by running
//! the function ONCE in a fresh [`Context`] with a known input and observing what
//! it creates.
//!
//! Currently catalogs one motif: [`extract_bits`].
//!
//! By building `extract_bits(value, n_bits)` over a known input `Simd` and
//! snapshotting `ctx.guessed_vars` before/after the call, we learn:
//! * its **input port** = the value vector fed into the reduction chain's first
//!   `sub` — created *before* the call (so promoted to `kind:"input"`);
//! * its **witnesses** = the per-bit lsb guesses created *during* the call
//!   (`guessed_vars[before..after]`, left as `kind:"witness"`).
//!
//! From the same single build we also capture the chain's structural shape so the
//! exporter can recognize other `extract_bits` instances in real circuits.

use circuits::context::Context;
use circuits::extract_bits::extract_bits;
use circuits::ivalue::NoValue;
use circuits::ops::guess;
use circuits::simd::Simd;

/// The structural signature of one `extract_bits` invocation, learned by running
/// it once. All counts are expressed *per packed var* (per lane-vector slot), so
/// they transfer to instances over inputs of any width.
#[derive(Debug, Clone)]
pub struct ExtractBitsSig {
    /// `n_bits` the catalog instance was built with.
    pub n_bits: usize,
    /// Number of `sub` gates per packed var (one per non-MSB bit) — i.e.
    /// `n_bits - 1`. The chain entry feeds the FIRST of these.
    pub subs_per_packed: usize,
    /// Number of `pmul` gates per packed var across the whole call:
    /// per bit: 1 (`lsb²` in assert_bits) + 1 (`·inv_two` reduction); plus the
    /// final MSB assert_bits 1. = `2*(n_bits-1) + 1`.
    pub pmuls_per_packed: usize,
    /// Number of `eq` gates per packed var: one per `assert_bits` (every bit incl.
    /// MSB) = `n_bits`.
    pub eqs_per_packed: usize,
    /// Number of lsb-guess witnesses created per packed var = `n_bits - 1`
    /// (the MSB is not guessed; it is the reduced `value`).
    pub guesses_per_packed: usize,
    /// Logical Simd length of the input `value` vector (M31 lanes). Learned from
    /// the catalog instance (`Simd::from_packed(data, LANES)`); the viewer uses
    /// it as the exact length of the recognized input port's merged SIMD node.
    pub input_len: usize,
    /// Source-level name of the per-bit guesses, transcribed from the function
    /// body (`let lsb = value.guess_lsb(context)`). Because each motif is a fixed,
    /// known function, we take the variable name from the code in advance rather
    /// than deriving a positional label — every bit guess is the loop's `lsb`.
    pub guess_name: &'static str,
}

/// The full catalog of learned motifs.
#[derive(Debug, Clone)]
pub struct Catalog {
    pub extract_bits: ExtractBitsSig,
}

/// Builds the catalog by running each motif once in a fresh context.
pub fn build() -> Catalog {
    Catalog { extract_bits: learn_extract_bits() }
}

/// Runs `extract_bits` ONCE over a known input (mirroring the demo:
/// `Simd::from_packed(4 guessed vars, 16)`, `n_bits = 8`) and reads its
/// signature off the resulting circuit and the guessed-vars delta.
fn learn_extract_bits() -> ExtractBitsSig {
    const N_BITS: usize = 8;
    const LANES: usize = 16;
    const PACKED: usize = 4; // 16 lanes / 4 lanes-per-QM31

    let mut ctx = Context::<NoValue>::new(0);

    // The value vector — created BEFORE the call. Definitionally the input port.
    let data: Vec<_> = (0..PACKED).map(|_| guess(&mut ctx, NoValue)).collect();
    let value = Simd::from_packed(data, LANES);
    let input_len = value.len();

    // Snapshot the gate counts and guess count just before the call.
    let subs_before = ctx.circuit.sub.len();
    let pmuls_before = ctx.circuit.pointwise_mul.len();
    let eqs_before = ctx.circuit.eq.len();
    let guesses_before = ctx.guessed_vars.as_ref().map(|g| g.len()).unwrap_or(0);

    let _bits = extract_bits(&mut ctx, &value, N_BITS as u32);

    let subs = ctx.circuit.sub.len() - subs_before;
    let pmuls = ctx.circuit.pointwise_mul.len() - pmuls_before;
    let eqs = ctx.circuit.eq.len() - eqs_before;
    let guesses = ctx.guessed_vars.as_ref().map(|g| g.len()).unwrap_or(0) - guesses_before;

    let learned = ExtractBitsSig {
        n_bits: N_BITS,
        subs_per_packed: subs / PACKED,
        pmuls_per_packed: pmuls / PACKED,
        eqs_per_packed: eqs / PACKED,
        guesses_per_packed: guesses / PACKED,
        input_len,
        guess_name: "lsb",
    };

    // The learned counts must match the function's closed form, confirming the
    // single build captured the motif's true shape (n_bits-1 reduction subs,
    // 2*(n_bits-1)+1 pmuls, n_bits eqs, n_bits-1 lsb guesses).
    debug_assert_eq!(learned.subs_per_packed, N_BITS - 1);
    debug_assert_eq!(learned.pmuls_per_packed, 2 * (N_BITS - 1) + 1);
    debug_assert_eq!(learned.eqs_per_packed, N_BITS);
    debug_assert_eq!(learned.guesses_per_packed, N_BITS - 1);

    learned
}
