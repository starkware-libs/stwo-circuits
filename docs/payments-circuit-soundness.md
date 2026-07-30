# Payments Circuit — Soundness Method

How soundness is established for the payments circuits, and what every PR in the stack owes.
Companion to `payments-circuit-design.md` (what is being built) — this is *how we know it is
right*.

## Vocabulary

Fixed terms, so docs, tests and identifiers stay consistent:

| term | meaning |
|---|---|
| **rejection case** | a modified witness that the constraints must reject; the catalogue lives in `patricia/skeleton_rejection.rs` |
| **rejection table** | the per-PR table pairing each rejection case with the check that rejects it |
| **derivation argument** | from any assignment satisfying the constraints, how the claimed object is recovered and why the relation follows |
| **incompletely constrained** | a witness value the constraints do not pin down — the bug class the witness table exists to surface |
| **independent review** | a review pass by someone other than the author, whose deliverable is an assignment satisfying the constraints without the statement |
| **unconstrained prover** | the party choosing witness values freely; what a proof must hold for, not an assumed attacker |

## Why tests are not enough

Soundness is a universal claim: **no** satisfying assignment proves a false transition. Tests
sample the witness space; they cannot cover it. Completeness (valid input → accepted) is what
functional tests check, and a circuit with a constraint silently removed still passes all of
them.

So each circuit ships two artifacts:

1. a **rejection table** — per constraint, a witness that violates it and must be rejected,
   *attributed* to that constraint rather than incidentally rejected by another;
2. a **written derivation argument** — from any satisfying assignment, extract the object
   (a trie, a batch) and show the claimed relation must hold. Gaps here are the real
   findings; missing tests are merely missing tests.

Negative tests are cheap: `Circuit::check(&values)` validates every gate against a concrete
assignment with **no proving** (see `circuits/src/circuit_test.rs`), so a rejection case costs
milliseconds. Reserve `prove` + `verify` for a couple of end-to-end cases and the
finalize/padding path.

## Per-PR template

Copy this into the PR description (and, condensed, into the circuit's module docs).

### Statement

What a satisfying assignment proves, stated so it can be falsified. Name the public
inputs/outputs and what binds them.

### Witness table

Every `guess()` site — every value the prover chooses freely — and what forces it. An entry
whose "determined by" column reads *nothing* is an incompletely constrained bug.

| witness value | width / range | determined by |
|---|---|---|
| … | … | … |

### Derivation argument

Given any assignment satisfying the constraints, how the claimed object is recovered and why
the relation follows. For tries this is injectivity: root equality forces leaf-set equality,
*given canonicity*. State where canonicity is enforced.

### Rejection table

| rejection case | must be rejected by |
|---|---|
| … | … |

### RejectionCases required by P4 / P5 — **landed**

The three rows now live in the catalogue (`Family::PresenceBinding`), rejected by both seams
(oracle and circuit). Out-of-circuit the two slot-local presence flips are attributed to the
live-count identity (`Check::SiblingCount`), a corollary of the multiset balance that the
oracle checks first:

| rejection case | oracle attribution |
|---|---|
| `present leaf value zeroed in its slot only (claimed absent)` | `SiblingCount` (the consumed copy also dangles) |
| `absent leaf slot given a live value (claimed present)` | `SiblingCount` (the produced unit also dangles) |
| `live skeleton claimed against the empty root` | `Root` (P5: root 0 forces everything inert) |

### Deliberately unproven

What the circuit does **not** attest, so nothing downstream over-claims. For v1 this includes
signature verification (design doc P3): the circuit proves bookkeeping over *unauthenticated*
transfers. The update circuit adds: `(0, 0)` rows prove "unchanged" only (P9), both-sides-absent
keys are unbound (the caller binds every key), and row distinctness is the batch producer's
obligation (two *live* rows on one key are rejected as a side effect; a live row plus a `(0, 0)`
row are independent).

## Standing checklist

Applied per PR; each line is a bug class seen in circuits of this kind.

- [ ] **Unconstrained witness** — every `guess()` appears in the witness table with a real
      determiner. `check_vars_used` proves a variable is *used*, not that it is *determined*.
- [ ] **Missing range check** — every field with a width (`height ≤ 251`, `ℓ ∈ [1, 251]`,
      key `< 2^height`, balance limbs) is bounded, and tested at ±1 of each bound.
- [ ] **Multiset without domain separation** — `kind` inside the hashed tuple; distinct tags
      for distinct instances, so a unit cannot migrate between them.
- [ ] **Cross-instance reuse** — a unit of the prev skeleton cannot satisfy the new skeleton
      (and vice versa); the balance and nonce tries cannot exchange units.
- [ ] **Kind confusion** — a leaf accepted where a binary child belongs, or an edge where a
      leaf belongs.
- [ ] **Live padding** — padded slots are forced to a fixed inert tuple; populating live-looking
      data into padding is rejected.
- [ ] **Position binding** — leaf path at height 0 *is* the key; `child.path = 2·parent.path +
      bit`; `bottom.path = parent.path·2^ℓ + edge.path` (variable 256-bit shift, design doc
      §4.2), including limb-boundary cases.
- [ ] **Canonicity** — lone child folded into an edge, no zero-value leaf, no `ℓ = 0`, no
      edge→edge chain. Each rule points at the constraint enforcing it.
- [ ] **Sibling opacity** — a sibling's claimed kind is unverifiable in-circuit; the argument
      for why that is safe is *written*, not assumed (design doc §6 Q3).
- [ ] **Topology determinism** — the emitted circuit depends only on shape parameters:
      witness-independent (existing pattern: `structure_is_witness_independent`,
      `circuit_is_fixed_across_shape`) **and** stable across processes
      (`fingerprint_is_pinned`, see below). A witness-dependent circuit means the verifier is
      not checking the statement it believes.
- [ ] **Case distinctions are derived, never witnessed** — every branch (empty vs non-empty,
      absent vs present) is a function of a public input or an already-constrained value
      (`is_zero_words(prev_root)`, `is_zero(value)`), never a fresh `guess()`. A free flag
      selecting between behaviours is a prover's choice of which statement to prove.
- [ ] **Bridge integrity** (payments-state) — the index↔account binding is injective *and*
      total; the per-limb balance dicts share one access sequence.

## Determinism of the generated circuit

Two properties, needing two different checks:

* **Witness independence** — build with `QM31` and with `NoValue`, compare the whole
  `Circuit`. Already the repo's pattern.
* **Cross-process stability** — a same-process comparison cannot see nondeterminism that
  varies *per process*. Iterating a `HashMap`/`HashSet` while emitting gates picks up std's
  per-process random seed, so gate order would differ between `cargo test` runs while every
  same-process assertion still passed. `fingerprint::circuit_fingerprint` hashes topology
  (gate order plus each gate's `uses`/`yields` wiring, never values); a **pinned** constant
  fails on nondeterminism and makes intentional topology changes explicit in the commit that
  causes them.

The existing code is already careful — `reference.rs` uses `BTreeMap`,
`finalize_constants.rs` uses `IndexMap` "so that iteration order is deterministic", and its
`HashMap`s are lookup caches that are never iterated. New circuit-building code keeps that
rule: **never iterate an unordered collection while emitting gates.**

## Open-gap ledger

What is knowingly **not** covered yet, and by what it must be closed. A gap recorded here with
a test that pins its existence is a tracked obligation; the same gap unwritten is a soundness
hole waiting to be assumed away. Every row must name the layer that closes it.

| gap | pinned by | closed by |
|---|---|---|
| **Same-kind class migration.** Moving a `Leaf`-tagged unit from the leaf class into the sibling class leaves every tag, cardinality and the multiset balance valid, so the *extractor's* invariants alone cannot detect it. | `skeleton_test::same_kind_class_migration_is_not_detected_here` (asserts the out-of-circuit gap, which remains) | **closed in step 1** (P4): in `verify_patricia_skeleton` the leaf slots *are* the batch — a fixed slot list — so there is no class to migrate out of. |
| **The empty trie has no skeleton.** `build_trie` returns `None` for an empty entry set, so there is no root unit for the multiset to hold out. **The nonce trie starts empty**, so this is on the payments critical path, not a corner case. | `skeleton_circuit_test::{empty_trie_is_accepted, nonzero_root_over_an_empty_skeleton_is_rejected}` and the `claim_empty_root` rejection case | **closed in step 1** (P5): the root entry is `is_zero_words(root)`-derived — `root == 0` consumes the inert unit, forcing every slot inert and every key absent; a non-zero root over an empty skeleton dangles. |
| **Sibling subtree authenticity** — *closed in step 2 (P7)*. Siblings carry the dedicated `Opaque` tag and may sit under an edge only at height 0, so no structural claim about an untouched subtree survives to be wrong; a sibling's hash is pinned through the prev fold to the public `prev_root`. Residual assumptions, written out in `update_circuit`'s module docs: blake collision resistance **including the shifted variant** (the edge hash adds `ℓ`, so an offset-collision within `[−251, 251]` — ≈9 bits against 251 — would open one edge hash two ways), plus a canonical `prev_root` (genesis + the induction the statement itself supports). | `edge bottom claims opaque above height 0` rejection case; `migrate_leaf_into_siblings` | — |
| **Absent keys emit no leaf unit** *(historical)*. | `skeleton_test` absent-key cases | **closed in step 1** (P4): the extractor emits a zero-valued leaf slot per absent key, 1:1 with the batch in key order, and the circuit derives `is_present ⟺ value ≠ 0`. Note the capacity consequence: each absent key needs one spare binary slot (`SkeletonCapacity::covering`). |
| **`(0, 0)` update rows prove "unchanged", never absence** (P9). For *live* rows absence is now proven — a false absence plus an insert leaves two live units at one position with a single root entry to consume them, so one chain dangles (`update_circuit` module docs; `false_absence_with_an_insert_cannot_be_hosted`). The residue: a row that writes nothing forces no walk, so it attests only `T_prev(k) = T_new(k)`. | `a_no_op_row_can_hide_a_present_unchanged_key` / `a_no_op_row_hiding_a_present_key_is_accepted` (pin the accepted semantics) | the caller: no consumer of the update may treat a `(0, 0)` row as non-membership. The nonce path never produces one — every transfer is a `(0, 1)` insert, the proven case. Step 3 inherits this as an interface rule. |
| **Production canonical *construction* is not yet pinned.** The golden vectors pin the hash convention (`hash2` truncated to 251 bits, edge = `hash2(bottom, path) + ℓ`), but not that production builds the same canonical shape — specifically that `Felt::from(EdgePath)` uses the same LSB-aligned `path` layout as `reference.rs`. | `patricia::reference::golden_test` (pins the spec, not the agreement) | **deliberately deferred** (decision, 2026-07-29): the closing test lives in `payment_thread_patricia` (a separate repo) and is not being added for now; until then "reference == production" rests on reading both. |
| **Signature verification (design doc P3).** v1 proves bookkeeping over *unauthenticated* transfers. | — | out of scope for v1; needs 252-bit field EC arithmetic the DSL lacks. Must be stated in the top-level circuit's own docs so nothing downstream over-claims. |

Note on the §4.1 identity: it is `siblings = binary − leaf_units + 1`, **not** `− K`. With
absent keys `leaf_units < K`, so a capacity sized from `K` stays correct while the
witness-side check must count leaf *units*.

### Why this ledger exists — a worked example

The reference trie originally hashed binary nodes as untruncated `blake2s(l ‖ r)` and edges as
`blake2s(bottom ‖ path ‖ ℓ)`, while production and the Cairo0 program use `hash2` truncated to
251 bits with `+ ℓ` added to the edge hash. **All 107 tests passed** before and after the
correction, because every one of them compared the reference against itself. A circuit built on
it would have proven roots the sequencer never computes — an airtight proof of the wrong
statement. `golden_test` now pins the convention against an independently computed vector, and
deliberately breaking the truncation fails 8 tests.

## Oracles worth relying on

* **`patricia::reference`** as a property-based twin: random `(N, K)` with the reference
  deciding validity, plus the corner shapes — empty trie, single leaf (full-height edge), keys
  differing only in the last bit, adjacent keys, `K = 1`, `K = capacity`, all-inserts (the
  nonce trie), all-deletes, mixed, duplicate keys in a batch, zero writes.
* **The Cairo0 implementation** in `payment-threads/crates/payment_thread_prove`: it proves
  the same statement and ships golden vectors plus a rejection-case suite. Every
  `(prev_root, batch, new_root)` accepted by one must be accepted by the other, and every
  rejection case it rejects must be rejected here. Two independent implementations disagreeing is
  the cheapest spec-drift detector available — note its contract is the EVM variant
  (`main_up_evm`: 160-bit address keys, EIP-712 keccak digest as the nonce key, block
  timestamp and domain-separator words in the output).

## Independent review

Every soundness-critical PR gets a review pass whose *only* goal is to construct an assignment
that satisfies the constraints without the statement, given the witness API and the
rejection-case harness — performed by someone other than the author. Findings land as new
rejection-table rows.
