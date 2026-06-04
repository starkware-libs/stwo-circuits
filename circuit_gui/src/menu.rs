//! The menu of circuits, built in-process with `NoValue` (topology only).
//!
//! Includes real verifier circuits built through the project's
//! `build`/`verify` entry points (the Blake hashing and STARK phases inside them
//! are grouped via the recorded scopes), plus small Blake examples.

use circuits::blake::blake;
use circuits::context::{Context, Var};
use circuits::finalize_constants::finalize_constants;
use circuits::ivalue::NoValue;
use circuits::ops::{Guess, guess, output};
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use circuits_stark_verifier::verify::verify;
use circuits_stark_verifier_examples::simple_air::LOG_SIZE_LONG;
use circuits_stark_verifier_examples::simple_statement::{
    COMPONENT_ENABLE_BITS, PREPROCESSED_COLUMN_LOG_SIZES, SimpleStatement,
    simple_statement_components,
};
use stwo::core::pcs::PcsConfig;

type Ctx = Context<NoValue>;

fn input(ctx: &mut Ctx) -> Var {
    guess(ctx, NoValue)
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

/// Returns the menu: `(name, builder)` pairs, in display order. Each builder is
/// invoked by `main` between a `scopes::reset()` and a `scopes::take_spans()`.
pub fn builders() -> Vec<(String, fn() -> Ctx)> {
    // Blake examples first (small, fast default); the fibonacci verifier is real
    // but very large (~44k gates) and slow to render — selectable, not default.
    vec![
        ("blake (1 block)".to_string(), blake_one_block as fn() -> Ctx),
        ("blake (2 blocks)".to_string(), blake_two_blocks as fn() -> Ctx),
        ("fibonacci verifier (large)".to_string(), fibonacci as fn() -> Ctx),
    ]
}
