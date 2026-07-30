# Lean formalization of the Patricia circuit

A Lean 4 development (core Lean only, no mathlib; toolchain pinned in `lean-toolchain`,
`lake build` to check) formalizing the statement and soundness argument of
`docs/patricia-circuit.md`, in the style of the cairo-lang Patricia verification
(`starkware-libs/cairo-lang`, `src/starkware/cairo/common/lean/patricia/`): the hash is a
parameter and collision resistance is an injectivity assumption — including under bounded
additive offsets, which is exactly the shifted-collision assumption (A1) the additive edge
length needs. Same statement shape, same assumptions.

## What is at stake, and the honest status

The development has two layers with different epistemic weight:

1. **Fully machine-checked theorems** — every theorem in the development is proven; there
   are no `sorry`s. Axiom footprint is standard (`propext`, `Classical.choice`,
   `Quot.sound`; verified with `#print axioms`).
2. **The modeling boundary** — what connects the Lean model to the Rust circuit, stated in
   `Model.lean`: the model is the circuit's *layer 1* (units, slot relations as integer
   semantics, `permute_units` as `List.Perm`); the limb-gadget layer (that gates imply those
   integer relations) and the DSL's variable-wiring lookup are assumed, justified by
   `word_gadgets_test.rs` and the DSL's single-yield invariant respectively. No Lean proof
   can substitute for that boundary being right — reviewing it means comparing `Model.lean`
   against `skeleton_circuit.rs` slot by slot.

Two amendments were made against the original transcription, both reported and approved:

* **`at'` denotes the trie-as-map** (`Spec.lean`): keys outside the keyspace `[0, 2^h)`
  read `0`. The doc fixes the keyspace (§2); the original `at'` returned garbage on
  out-of-range keys, falsifying `update_sound`'s `∀ k` agreement clause even for honest
  updates.
* **`update_sound` carries an interface precondition**: live rows have pairwise-distinct
  keys. The circuit does not enforce it and the statement is false without it — a
  delete-shaped plus an insert-shaped row on one key act jointly as an overwrite while each
  row's absence claim is wrong, and the duplicate-position rejection never fires (each fold
  sees only one live entry at the position). The batch producers guarantee the
  precondition; see the theorem's docstring.

## Files

| file | contents |
|---|---|
| `Crypto.lean` | A1 as the cairo-lang-style `Crypto` structure: `hash_shift_inj` (shifted collision resistance as injectivity), `hash_ne_small`, output bound |
| `Spec.lean` | canonical tries, node hashing, key semantics (`at'`, keyspace-ranged) |
| `Model.lean` | the circuit model: `SUnit`, the five slot tables with their local `ok` relations, `Fold.sat`, `UpdateAssign.sat` (P6 sharing by construction) |
| `Node.lean` | positions in a trie: `nodeAtDepth`, the two-block path arithmetic, composition/validity/bounds, and how `at'` factors through nodes |
| `Pinning.lean` | §5.1–5.2: `Pinned`, per-slot hash/position decomposition, `pinned_of_consumed` (descending induction from the root entry), `present_reads` |
| `Counting.lean` | §5.3–5.4 as one `countP` identity per key (`master`), its per-slot bridges, and the decoders (`diverges_absent`, `sibling_covers`) |
| `Reconstruct.lean` | §6.2 e: decoding a satisfying fold into a canonical tree (`root_tree_exists`) |
| `Soundness.lean` | the main theorems, assembled |

## Theorem ledger

**All proven** (axioms: `propext`, `Classical.choice`, `Quot.sound`):

| theorem | meaning (doc reference) |
|---|---|
| `Crypto.hash_inj`, `hash_ne_zero`, `hash_add_lt_wordModulus` | derived hash facts |
| `Tree.valid_hashOf_ne_zero` | a canonical tree never hashes to the empty encoding (P5, spec side) |
| `Tree.hashOf_inj` | canonicity makes hashing injective — cairo-lang's `hash_inj` for our node encoding (§6.2 e) |
| `newRoot_commits` | the roots are commitments: canonical tries with equal root hashes are equal, empty case included |
| `climb` | the core of "no cycles, one exit" (§5.2): over the empty root, a live produced unit forces a live produced unit strictly higher |
| `empty_root_all_inert`, `empty_root_all_absent` | **P5, end to end**: against the zero root every slot is inert and every leaf claim absent (§6.2, emptiness) |
| `pinned_of_consumed` | **top-down pinning** (§5.1, §6.2 a): every consumed unit with non-zero hash carries the height, position, and hash of a real node of the committed trie |
| `master` | **the counting identity** (§5.3–5.4): per keyspace key, `#present-leaves + #diverging-edges + #covering-siblings = 1` — duplicate-position, absence, and frontier exhaustiveness in one `List.Perm.countP` computation |
| `reconstruct_consumed`, `root_tree_exists` | **decoding** (§6.2 e): a satisfying fold's units open to canonical trees; applied to the root entry this constructs `T_new` |
| `fold_reads_the_tree` | **skeleton soundness** (§6.2 a–b): every present leaf slot reads the committed trie truthfully |
| `update_sound` | **update soundness** (§1, §6.2 c–e): `T_new` exists, is canonical, live rows read both tries (absence proven), off-row keys agree — under the documented live-key-distinctness precondition |

## Relation to the rejection-case harness

The two verification artifacts are complementary, not redundant: the Lean theorems
quantify over *all* satisfying assignments of the model but stop at the modeling boundary;
the Rust rejection catalogue exercises the *actual* circuit end to end (gates, gadgets,
permutation, finalization) but only at sampled witnesses. A defect below the modeling
boundary (a gadget emitting the wrong relation) is the harness's to catch; a defect in the
argument itself (an incompletely constrained witness the catalogue never thought to try) is
the formalization's — and one such defect was found this way (the same-key
delete-plus-insert pair, now a documented interface precondition).
