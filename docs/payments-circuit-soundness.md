# Payments Circuit — Soundness Method

How soundness is established for the payments circuits, and what every PR in the stack owes.
Companion to `payments-circuit-design.md` (what is being built) — this is *how we know it is
right*.

## Why tests are not enough

Soundness is a universal claim: **no** satisfying assignment proves a false transition. Tests
sample the witness space; they cannot cover it. Completeness (valid input → accepted) is what
functional tests check, and a circuit with a constraint silently removed still passes all of
them.

So each circuit ships two artifacts:

1. a **mutation matrix** — per constraint, a witness that violates it and must be rejected,
   *attributed* to that constraint rather than incidentally rejected by another;
2. a **written extraction argument** — from any satisfying assignment, extract the object
   (a trie, a batch) and show the claimed relation must hold. Gaps here are the real
   findings; missing tests are merely missing tests.

Negative tests are cheap: `Circuit::check(&values)` validates every gate against a concrete
assignment with **no proving** (see `circuits/src/circuit_test.rs`), so a mutation costs
milliseconds. Reserve `prove` + `verify` for a couple of end-to-end cases and the
finalize/padding path.

## Per-PR template

Copy this into the PR description (and, condensed, into the circuit's module docs).

### Statement

What a satisfying assignment proves, stated so it can be falsified. Name the public
inputs/outputs and what binds them.

### Witness table

Every `guess()` site — every value the prover chooses freely — and what forces it. An entry
whose "determined by" column reads *nothing* is an under-constraining bug.

| witness value | width / range | determined by |
|---|---|---|
| … | … | … |

### Extraction argument

Given any assignment satisfying the constraints, how the claimed object is recovered and why
the relation follows. For tries this is injectivity: root equality forces leaf-set equality,
*given canonicity*. State where canonicity is enforced.

### Mutation matrix

| mutation | must be rejected by |
|---|---|
| … | … |

### Deliberately unproven

What the circuit does **not** attest, so nothing downstream over-claims. For v1 this includes
signature verification (design doc P3): the circuit proves bookkeeping over *unauthenticated*
transfers.

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
- [ ] **Live padding** — padded slots are forced to a fixed inert tuple; stuffing live-looking
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

## Oracles worth exploiting

* **`patricia::reference`** as a property-based twin: random `(N, K)` with the reference
  deciding validity, plus the corner shapes — empty trie, single leaf (full-height edge), keys
  differing only in the last bit, adjacent keys, `K = 1`, `K = capacity`, all-inserts (the
  nonce trie), all-deletes, mixed, duplicate keys in a batch, zero writes.
* **The Cairo0 implementation** in `payment-threads/crates/payment_thread_prove`: it proves
  the same statement and ships golden vectors plus a tamper matrix. Every
  `(prev_root, batch, new_root)` accepted by one must be accepted by the other, and every
  tamper case it rejects must be rejected here. Two independent implementations disagreeing is
  the cheapest spec-drift detector available — note its contract is the EVM variant
  (`main_up_evm`: 160-bit address keys, EIP-712 keccak digest as the nonce key, block
  timestamp and domain-separator words in the output).

## Adversarial review

Every soundness-critical PR gets a review pass whose *only* goal is to construct a
false-but-accepted witness, given the witness API and the mutation harness — performed by
someone other than the author. Findings land as new matrix rows.
