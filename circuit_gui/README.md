# circuit-gui

A standalone, **read-only** browser visualizer for `stwo-circuits` circuits.

It renders a circuit as a **gate-centric graph**: nodes are gates, and a wire is
drawn directly from the gate that *yields* a variable to each gate that *uses*
it (so the use/yield lookup terms of a variable become a single edge). High-level
operations (a `blake()` call, authored phases, …) collapse into nested,
expandable group boxes so large circuits stay navigable.

## Quick start

```bash
cd circuit_gui
cargo run                 # builds the menu circuits, writes viewer/data.js
# then open viewer/index.html in a browser (no server needed)
```

`cargo run -- <out_dir>` writes `<out_dir>/data.js` instead of `viewer/data.js`.

## Viewer controls

- **circuit** dropdown — pick which circuit to view.
- **detail level** slider — `overview` (all groups collapsed) → deeper levels →
  `all gates`. Travels top-down through the hierarchy.
- **Collapse all / Expand all / Fit** buttons.
- Click a **group box** (or its ⊕/⊖ cue) to expand/collapse just that group.
- Click any **node or edge** for details in the side panel.
- **Zoom / drag** to travel — the **minimap** (bottom-right) shows your viewport
  within the whole circuit. Zoom never opens groups; expansion is always explicit.

Visual encoding: color per gate kind (see the legend), a gold border marks a
public `Output`, an `Eq` constraint is a dashed `=` edge between the two
producers, and constants (`0`/`1`/`u`/named relation ids) are inlined as `[…]`
badges on the consuming node rather than shared hub nodes.

## How grouping works

Groups come from **scopes** recorded while a circuit is built
(`Context::push_scope` / `pop_scope`). The recording is gated behind the
`circuits` crate's default-off **`gui-scopes`** feature, which this crate
enables. With the feature off (every normal build, test, and CI run) the scope
calls are zero-cost no-ops and nothing is recorded.

The example circuits in [`src/menu.rs`](src/menu.rs) author their own scopes
(e.g. `inputs` / `compute` / `division`, or `message` / `blake` / `outputs`).
Anything that can be reconstructed from the flat circuit is **not** scoped: the
`blake` scope is subdivided into `block` → `round` → G-gate groups
arithmetically by the exporter (Blake2s is 10 rounds × 8 G per block), at no
scope cost.

## Changes to existing crates

This crate is its own cargo workspace (note the empty `[workspace]` table in
`Cargo.toml`), so it is excluded from the parent workspace and adding it changes
no shared manifest. The only edits to existing files are:

- `crates/circuits/Cargo.toml` — adds the default-off `gui-scopes` feature.
- `crates/circuits/src/lib.rs` — one line: `pub mod scopes;`.

All scope machinery lives in the **new** file `crates/circuits/src/scopes.rs`
(thread-local recording + the `push_scope`/`pop_scope` methods). The `Context`
struct, and the builder files (`blake.rs`, `ops.rs`, …), are untouched.

## Status / follow-ups

- The menu currently holds small authored examples. Wiring the **real
  verification circuit** (which needs a proof fixture) and adding the genuinely
  non-derivable **phase scopes** (`fri` / `oods` / `merkle` / … in
  `stark_verifier::verify`) is the next step — those are the only scopes that
  would need to live in existing builder code.
- Grouping of Blake / `div` / `sort` in *loaded* (non-instrumented) circuits
  would use structural detection (Blake gates, the `guess→mul→eq` div pattern,
  the `Permutation` gate); not yet implemented.
