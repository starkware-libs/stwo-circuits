//! The menu of circuits, built in-process with `NoValue` (topology only).
//!
//! Includes real verifier circuits built through the project's
//! `build`/`verify` entry points (the Blake hashing and STARK phases inside them
//! are grouped via the recorded scopes), plus small Blake examples.

use circuits::blake::blake;
use circuits::context::{Context, Var};
use circuits::finalize_constants::finalize_constants;
use circuits::ivalue::NoValue;
use circuits::extract_bits::extract_bits;
use circuits::ops::{Guess, guess, output};
use circuits::simd::Simd;
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use circuits_stark_verifier::verify::verify;
use circuits_stark_verifier_examples::simple_air::LOG_SIZE_LONG;
use circuits_stark_verifier_examples::simple_statement::{
    COMPONENT_ENABLE_BITS, PREPROCESSED_COLUMN_LOG_SIZES, SimpleStatement,
    simple_statement_components,
};
use stwo::core::pcs::PcsConfig;

type Ctx = Context<NoValue>;

// Genuine boundary inputs are created with `input()` (a `guess()` the builder
// DECLARES as an input). At the circuit level a boundary input and a prover
// witness are both guessed vars — indistinguishable — so we record which guesses
// are inputs here (thread-local, mirroring `circuits::scopes`) and hand the set
// to the exporter, which marks them `kind:"input"` (top row). This is how blake's
// message words (and any builder's inputs) get marked as inputs, not witnesses.
thread_local! {
    static INPUT_VARS: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
}
/// Clear the declared-input set (call before building a circuit).
pub fn reset_inputs() {
    INPUT_VARS.with(|v| v.borrow_mut().clear());
}
/// Drain the declared-input var indices recorded during the last build.
pub fn take_inputs() -> Vec<usize> {
    INPUT_VARS.with(|v| std::mem::take(&mut *v.borrow_mut()))
}

fn input(ctx: &mut Ctx) -> Var {
    let g = guess(ctx, NoValue);
    INPUT_VARS.with(|v| v.borrow_mut().push(g.idx));
    g
}

/// The STARK verifier circuit for the example Fibonacci AIR, built with
/// `NoValue` from an empty proof of the right shape (same path the tests use).
fn fibonacci() -> Ctx {
    let mut pcs = PcsConfig::default();
    pcs.lifting_log_size = Some(LOG_SIZE_LONG + pcs.fri_config.log_blowup_factor);
    let components = simple_statement_components::<NoValue>();
    let config = ProofConfig::new(
        &components,
        COMPONENT_ENABLE_BITS.to_vec(),
        PREPROCESSED_COLUMN_LOG_SIZES.len(),
        &pcs,
        8,
    );

    let mut ctx = Context::<NoValue>::default();
    // Scope the stages that produce gates (from here, not the library) so they
    // land under a top-level group. `proof`/`statement` only `guess()` — no gates
    // of their own (those become x+0=x gates under `finalize_guessed_vars`) — so
    // they are left unscoped.
    let proof_vars = empty_proof(&config).guess(&mut ctx);
    let statement = SimpleStatement::new(&mut ctx);
    ctx.push_scope("verify");
    verify(&mut ctx, &proof_vars, &config, &statement);
    ctx.pop_scope();
    ctx.push_scope("finalize_constants");
    finalize_constants(&mut ctx);
    ctx.pop_scope();
    ctx.push_scope("finalize_guessed_vars");
    ctx.finalize_guessed_vars();
    ctx.pop_scope();
    ctx
}

/// The real Cairo verifier circuit (privacy bootloader config), NoValue topology.
/// Excluded from the menu for now: it takes >150s to build and is far too large
/// (100k+ gates) to render in the browser. Kept here for when we add a
/// group-only / lazy export path for very large circuits.
#[allow(dead_code)]
fn cairo_verifier() -> Ctx {
    use circuit_cairo_verifier::privacy::privacy_cairo_verifier_config;
    use circuit_cairo_verifier::verify::build_cairo_verifier_circuit;
    build_cairo_verifier_circuit(&privacy_cairo_verifier_config(1))
}

/// A single-block Blake2s hash (small example). The `blake` scope is subdivided
/// into 10 rounds of 8 G-gates by the exporter.
fn blake_one_block() -> Ctx {
    blake_hash(1)
}

/// A two-block Blake2s hash, to show the `blake block` level of the hierarchy.
fn blake_two_blocks() -> Ctx {
    blake_hash(2)
}

fn blake_hash(n_blocks: usize) -> Ctx {
    let mut ctx = Context::new(0);
    let n_words = n_blocks * 4;
    let n_bytes = n_blocks * 64;

    let message: Vec<_> = (0..n_words).map(|_| input(&mut ctx)).collect();

    // `blake()` pushes its own "blake" scope; don't double-wrap it here.
    let h = blake(&mut ctx, &message, n_bytes);

    output(&mut ctx, h.0);
    output(&mut ctx, h.1);
    ctx
}

/// A standalone `extract_bits` demo (16 M31 lanes = 4 packed vars, 8 bits each).
/// Deliberately UNSCOPED: the `extract_bits` / `simd::*` grouping you see is
/// produced entirely by the exporter's motif catalog, not a hand-placed scope.
fn extract_bits_demo() -> Ctx {
    let mut ctx = Context::new(0);
    let data: Vec<Var> = (0..4).map(|_| input(&mut ctx)).collect();
    let value = Simd::from_packed(data, 16);
    let bits = extract_bits(&mut ctx, &value, 8);
    for bit in &bits {
        for &v in bit.get_packed() {
            output(&mut ctx, v);
        }
    }
    ctx
}

/// Returns the menu: `(name, builder)` pairs, in display order. Each builder is
/// invoked by `main` between a `scopes::reset()` and a `scopes::take_spans()`.
pub fn builders() -> Vec<(String, fn() -> Ctx)> {
    // Blake examples first (small, fast default); the fibonacci verifier is real
    // but very large (~44k gates) and slow to render — selectable, not default.
    vec![
        ("blake (1 block)".to_string(), blake_one_block as fn() -> Ctx),
        ("blake (2 blocks)".to_string(), blake_two_blocks as fn() -> Ctx),
        ("fibonacci verifier (large)".to_string(), fibonacci as fn() -> Ctx),
        ("extract_bits (demo)".to_string(), extract_bits_demo as fn() -> Ctx),
    ]
}
