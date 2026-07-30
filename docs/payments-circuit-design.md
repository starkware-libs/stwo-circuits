# Payments Circuit — Design & Handoff

Status: **steps 1–2 implemented.** Branch `anatg/payments-circuit`. `verify_patricia_skeleton`
(`patricia/skeleton_circuit.rs`) verifies the skeleton in-circuit — units-by-multiset over the
four slot classes, the §4.2 shift and §4.4 truncation/additive-length gadgets
(`patricia/word_gadgets.rs`), and the P4/P5 derived flags — with the full rejection catalogue
running through both the out-of-circuit oracle and the circuit, witness-independence, and a
pinned cross-process fingerprint. Supporting layers: the skeleton extractor + rejection harness
(`patricia/skeleton.rs`), golden vectors pinning the production hash convention
(`patricia/golden_test.rs`), and the topology fingerprint (`fingerprint.rs`). Next: step 2,
`verify_patricia_update` (§5).

Soundness method, the open-gap ledger, and the per-PR obligations: `payments-circuit-soundness.md`.

Audience: whoever picks this task up next. Read §1–§3 for the goal, §4 before writing any
code (it corrects the Patricia design doc), §5 for the delivery plan.

## 1. Goal

An in-circuit proof of a **payment block transition**, native to the stwo-circuits DSL —
replacing the Cairo0 program that does this today.

The contract to reproduce is already specified and implemented once, in Cairo0, at
`payment-threads/crates/payment_thread_prove/` (see its `DESIGN.md`). Two sparse Blake2s
Patricia tries of height 251:

| tree | key | leaf | meaning |
|---|---|---|---|
| balance | `account` (251-bit) | balance | `account = truncate_251(blake2s(pub_key))` |
| nonce | `msg_hash` (251-bit) | `0 → 1` | transfer already executed (replay protection) |

Per transfer: range-check the amount and the expiration window, derive the sender account
from the pubkey, compute the message hash, verify the signature, spend the nonce leaf,
apply `−amount` / `+amount` to the two balance leaves. Then, per tree: one squashed dict
and one Patricia update against that tree's `prev_root → new_root`. All four roots plus the
expiration and transfer count are program outputs.

The Cairo0 version costs **≈3.6k steps/transfer**, dominated by the Patricia update. A
native circuit removes the Cairo VM layer entirely.

Business context (block cadence, artifact format, cost targets) is in
`payment-threads/docs/design-doc.md` §7 and §10. The per-block obligations there — anchor,
authenticate, debits-in-order, credits, conserve, result — are the same statement in
different words.

## 2. Locked decisions

| # | Decision | Rationale |
|---|---|---|
| P1 | **Build in `stwo-circuits`**, branch `ilya/payments-base` off `ilya/circuit-dict` | The dict and the Patricia reference lived on separate branches of separate clones. This branch is their union — see §3. |
| P2 | **Index-keyed dict + binding argument** for the dict↔Patricia bridge | `Dict` is M31 keys *and* values; Patricia keys/values are `Word256`. A 251-bit account and a u128 balance fit neither. See §2.1. |
| P4 | **Absent keys get a leaf slot** — leaf slots are 1:1 with the batch's `K` keys; an absent key carries value `0` | Closes same-kind class migration inside the skeleton instead of deferring it to the step-3 bridge, fixes `n_leaves = K` for any present/absent mix, and makes the insert-dominated case (§4.3) first class. The absent flag is *derived*: a canonical trie has no zero-valued leaves, so `is_absent ⟺ is_zero(value)` — no new free witness. Splits the argument into two multisets: trie-structure (present leaves + binaries + edges + siblings) and batch-binding (all `K` leaf slots). |
| P5 | **Emptiness is derived from the root, not witnessed** | An empty trie has no unit to hold out as the root. `is_empty = is_zero_words(prev_root)` reads a **public input**, so `root == 0` ⟹ all trie slots inert and all keys absent, with nothing for a prover to choose. Rejected: injecting a synthesized empty-root unit (produced by nothing, so it needs the same flag anyway, and collides with `hash = 0` padding) and a free boolean flag (a prover could claim empty for a non-empty trie). Seeding a genesis leaf in production so the case never arises is worth doing as well, but must not be *relied* on: that would make satisfiability a trusted input. |
| P6 | **Structural sharing in the update circuit**: sibling units are guessed once and fed into *both* skeletons' multisets; per-row keys are shared vars; the slot builders are extracted into a `skeleton_core` reused by both `verify_patricia_skeleton` and `verify_patricia_update` | Sharing and cross-instance separation become structural facts (inexpressible to violate) instead of constraints; one copy of the canonicity rules that can never drift. Rejected: eq-bound sibling pairs (duplicated guesses + an alignment seam), one combined side-tagged multiset (turns "a prev unit cannot satisfy the new fold" into a tag argument). |
| P7 | **Siblings carry a dedicated `Opaque` kind tag; an edge bottom may be Opaque only at height 0** — extraction opens one binary level below an edge whose bottom would otherwise be opaque above height 0 | Closes §6 Q3's kind residual: a mis-kinded sibling in the delete-merge shape could otherwise make `new_root` a *non-canonical encoding of the correct map*, breaking the canonical-prev induction across blocks. Cost ≈ one Blake gate per divergence; bonus: a sibling can never cancel against a slot output. |
| P8 | **One shared capacity for both update sides** | `n_binary` is forced common anyway by the shared sibling count; a per-side `n_edge` saves only padded edge slots. |
| P9 | **`(0, 0)` update rows are accepted and prove "unchanged", never absence** | Forbidding them makes honest delete-of-absent and net-zero batches unsatisfiable (an availability trap). The nonce path never produces them — every transfer is a `(0, 1)` insert, the proven-absence case. Recorded in the open-gap ledger. |
| P3 | **Signature verification out of scope for v1** | Nothing in stwo-circuits can verify a signature — no EC arithmetic, no keccak. `grep -ri 'secp256k1\|ecrecover\|keccak' crates/` hits only `cairo_verifier/src/statement.rs`, unrelated. v1 proves the state transition; signatures are an explicit unproven assumption. |

### 2.1 The dict↔Patricia bridge (P2)

`Dict` (`crates/circuits/src/dict.rs`) is the in-circuit twin of Cairo's `squash_dict`, and
it is the right tool for collapsing many accesses per account into one `(prev, new)` pair to
feed the Patricia update. But its contract is narrow: keys and values are `M31Wrapper`, and
keys are additionally range-checked below `2^(30 − ceil(log2 n_accesses))` — with 2^14
accesses, keys must be under 2^16.

So neither a 251-bit account id nor a u128 balance can be a dict key or value directly.
The bridge:

- **Key = a batch-local account index**, small enough for the dict's range check. A separate
  perrejection case/multiset argument binds each index to its 251-bit account id exactly once, so
  the index is a faithful stand-in inside the dict and the real key is what reaches the
  Patricia update.
- **Value = balance split across parallel dicts, one per limb**, sharing the same access
  sequence. Each limb's chain condition is independent and the per-key ordering is identical,
  so the limbs stay consistent.

This leaves `dict.rs` — reviewed, soundness-critical — untouched. The alternative
(generalizing the dict to QM31/`HashValue` keys and values) would rework its packing, its
sort-key range check and its sortedness argument, and was rejected for v1.

## 3. What this branch contains

Two cherry-picks onto `ilya/circuit-dict`, both clean (common base `7bbff4c9`):

- `649f4f50` — `feat(unpacker): add Merkle commitment unpacker circuit`
- `f601bb76` — `feat(patricia): add Patricia trie reference implementation and shared gadgets`

plus this document and `crates/unpacker/src/patricia/DESIGN.md` (which was untracked in its
origin clone).

So the branch now has, coexisting for the first time:

- `crates/circuits/src/dict.rs` — the squashed-dict API.
- `crates/unpacker/src/patricia/reference.rs` — the out-of-circuit canonical trie: build,
  hash, `apply_updates`, `is_canonical`. **Witness generator and test twin.**
- `crates/unpacker/src/unpacker.rs` — the wiring-by-multiset architecture the Patricia
  circuits are meant to reuse.
- `crates/unpacker/src/gadgets.rs` — `select_hash`, `is_zero_words`, guessing helpers.

`cargo test -p circuits-unpacker -p circuits --lib` → 56 passed.

**The Patricia circuit itself does not exist.** `crates/unpacker/src/patricia/DESIGN.md`
describes PRs 1–3 and its own status line says PRs 2–3 are upcoming; only the reference
implementation landed. `verify_patricia_update` is a proposed signature, not code. Building
it is the bulk of the remaining work, and §4 is why it is bigger than that doc says.

## 4. Findings that correct `patricia/DESIGN.md`

Read this before trusting the slot counts in that document.

### 4.1 Slot counts are understated by ~12× — DESIGN.md:62

The claim: capacity `K` fixes "`K` leaf slots, `K − 1` binary slots, ~`2K` edge slots —
independent of trie height, batch content, or shape", justified by "a Patricia skeleton over
`K` keys has O(K) nodes".

That counts only the binary nodes where two *batch* keys diverge. It omits the binary nodes
where a touched path meets an **untouched sibling subtree**. In a canonical trie every
binary node has two non-empty children (a lone child folds into an edge), so every such
meeting costs one binary slot *and* one sibling unit.

Measured on tries built by `reference.rs`, height 251, pseudorandom keys, K = 256:

| N accounts | binary | edge | siblings | binary / K |
|---:|---:|---:|---:|---:|
| 1,000 | 657 | 388 | 402 | 2.6 |
| 10,000 | 1,505 | 407 | 1,250 | 5.9 |
| 100,000 | 2,338 | 421 | 2,083 | 9.1 |
| 1,000,000 | 3,173 | 419 | 2,918 | 12.4 |

Binary slots scale as **Θ(K · log₂(N/K))**, not Θ(K). At 1M accounts that is 3,173 slots
where the doc budgets 255. Every slot emits a Blake2s gate, so this is a ~12× circuit-size
miss that *grows with the account set*. The edge count (~1.6K) does match "~2K".

The multiset identity, derived independently and confirmed exactly on every row:

```
siblings = binary_slots − leaf_slots + 1        (3173 − 256 + 1 = 2918 ✓)
```

Consequences:

- Capacity is **two** free parameters (leaves, binary) with edges a third and siblings
  derived — not the single `capacity: usize` of DESIGN.md:92.
- The cost model in any downstream estimate must carry the `log₂(N/K)` factor.

To reproduce, and to keep the identity pinned as the circuits get built:

```bash
cargo test -p circuits-unpacker --lib patricia::slot_count_test -- --ignored --nocapture
```

`crates/unpacker/src/patricia/slot_count_test.rs` holds both the `#[ignore]`d report that
produced the table (~30 s) and `sibling_identity_holds`, which asserts the identity above and
runs in CI.

### 4.2 Edge slots need a 256-bit variable shift — unspecified at DESIGN.md:69

Binary slots check `child.path = 2·parent.path + bit`: multi-word doubling with carries,
cheap (8 guessed words + 8 carry bits + linear constraints).

Edge slots are specified only as "check length bounds and path-bit alignment". The actual
position relation is

```
bottom.path = parent.path · 2^ℓ + edge.path,      ℓ ∈ [1, 251]
```

— a **variable 256-bit shift**. `patricia_with_blake.cairo` gets this for one field multiply
because a 251-bit path is a single felt; the M31/QM31 DSL has no such type, so it needs a
barrel shifter over eight u32 limbs (word-rotate by `ℓ/32` via select stages, then shift by
`ℓ%32` via a verified `2^r` and per-word lo/hi splits).

Costed at ~150–250 gates per edge slot — **cheaper than the Blake2s gate that slot already
emits**, so this is an omission to close, not a blocker. But it is unspecified work that
needs designing and reviewing, and it does not appear in the delivery plan of that document.

### 4.3 The payments use case is insert-dominated — DESIGN.md:98–110

That plan defers absent keys, inserts and deletes to PR 3 (`patricia-update`), treating them
as a refinement over a present-keys-only skeleton.

For payments they are the main case:

- the **nonce tree is 100% inserts** — every leaf goes `0 → 1`, so a present-keys-only
  skeleton cannot express a single nonce spend;
- new accounts insert into the balance tree.

So the insert path belongs in the skeleton circuit's design from the start, not bolted on
afterwards. This is the main open question in §6.

### 4.4 The trie hashes 251-bit values, and the edge length is additive

`reference.rs` originally hashed binary nodes as untruncated `blake2s(l ‖ r)` and edges as
`blake2s(bottom ‖ path ‖ ℓ)`. Production (`payment_thread_patricia::Blake2s251`) and the Cairo0
program both use

```
hash2(x, y) = LE_u32s(blake2s(LE32(x) ‖ LE32(y))) & (2^251 - 1)
binary      = hash2(left, right)
edge        = hash2(bottom, path) + ℓ          // ADDED to the hash, not hashed with it
leaf        = value,   empty = 0
```

so the reference was committing to a different trie than the sequencer. Fixed, and pinned by
`patricia/golden_test.rs` against vectors from a third independent implementation. Two
consequences the slot-count cost model does not carry:

- **Every node hash is a 251-bit value**, not 256. Each `HashValue` over this trie owes a
  top-word range check (27 bits), and the "`hash = 0` means empty" reasoning that inert padding
  leans on lives in the same 251-bit space.
- **The edge hash needs a multi-limb add with carry** — `hash2(...) + ℓ` over eight u32 limbs —
  in addition to the variable 256-bit shift of §4.2. Cheap next to the Blake2s gate the slot
  already emits, but it is a second unspecified gadget in the same slot.

## 5. Delivery plan

A stack of small, independently-reviewable PRs (`gt`-friendly), on top of this branch.

1. **`patricia-skeleton`** — **done** (`patricia/skeleton_circuit.rs`): units-by-multiset over
   leaf / binary / edge / sibling slots plus a root entry on the consumed side, per-slot path
   and height constraints, the §4.2 shift gadget, the §4.4 top-word truncation and
   additive-length gadgets, and the P4/P5 `is_zero`-derived presence/emptiness flags.
   Statement, witness table and the deliberately-unproven list live in the module docs;
   acceptance and rejection coverage in `skeleton_circuit_test.rs`. One capacity consequence
   found during implementation: each absent key needs one spare binary slot
   (`SkeletonCapacity::covering`).
2. **`patricia-update`** — **done** (`patricia/update_circuit.rs`): two `skeleton_flow`
   instances over shared sibling units and shared row keys (P6), the `Opaque` sibling tag with
   the height-0 edge-bottom rule (P7), one shared capacity (P8), and `(0, 0)` rows accepted with
   "unchanged" semantics (P9). Witness extraction with frontier reconciliation and the
   update-level oracle live in `patricia/update.rs`; the rejection catalogue
   (`update_rejection.rs`) covers modified siblings (shared and one-sided), unbacked and dropped
   writes, false absence, fake divergence, rebound keys and both root bindings, and runs through
   both seams. Statement, closing argument and witness table: `update_circuit.rs` module docs.
3. **`payments-state`** — the §2.1 bridge: index-keyed dicts → Patricia batch updates,
   including the index↔account binding argument and the balance-limb dicts.
4. **`payments-transfer`** — per-transfer execution: Blake2s message hash, window checks,
   nonce spend, debit/credit, conservation (Σ debits = Σ credits).
5. **`payments-top`** — the block circuit: four roots in and out, wired to the
   payment-threads artifact encoding, cross-checked against
   `payment_thread_prove/tools/gen_input.py` golden vectors.

Steps 1–2 are unavoidable prerequisites and are most of the effort.

## 6. Open questions for the assignee

1. ~~**Does the skeleton circuit handle absent keys (value 0) from the start?**~~ **Decided:
   yes** — inserts and absent keys are in step 1 (P4), and the empty trie is handled by a
   root-derived flag (P5). The nonce trie needs both: it is 100% inserts and starts empty.
2. **How is capacity sized in production?** `n_leaves = K` is now fixed by P4, and siblings
   are derived, so the open parameter is `n_binary` (plus `n_edge`). It scales as
   `K·log₂(N/K)` — measured `binary/K ≈ log₂(N/K) + 0.5`, with `edges ≈ 1.6K` independent of
   `N` — so a fixed generous budget wastes gates on small blocks and *breaks* on large account
   sets. An undersized capacity is unsatisfiable (availability, not soundness), an oversized
   one grows the padding surface that must be proven inert. Options: per-block capacity
   classes, or a capacity derived from a *committed* account count — assuming it makes the
   choice a trusted input.
3. ~~**Sibling canonicity.**~~ **Decided (P7)**: siblings carry a dedicated `Opaque` tag and
   an edge bottom may be Opaque only at height 0, so no structural claim about a sibling's
   kind survives to be wrong. The residual assumptions of the injectivity argument reduce to
   blake collision resistance — including its *shifted* variant, since the edge hash is
   `hash2 + ℓ` with the length added (an offset-collision within `[−251, 251]` would open one
   edge hash two ways; ≈9 bits against a 251-bit hash, stated rather than assumed) — plus a
   canonical `prev_root` (genesis + induction, which full new-side canonicity supports).
4. **Signature verification** (P3) is deferred, so v1 proves bookkeeping over
   *unauthenticated* transfers. `payment-threads` D9 is explicit that this attests
   bookkeeping, not validity. Confirm that is acceptable for the intended use, and track
   what it would take to close: STARK-curve ECDSA needs 252-bit field EC arithmetic that
   this DSL does not have.

## 7. Interfaces (as landed)

```rust
/// Slot budget. `n_binary` must cover sibling-induced branch points (K·log2(N/K), §4.1) plus
/// one slot per absent key; siblings are derived: n_binary - n_leaves + 1. Leaf slots are the
/// batch — n_leaves = K exactly, never padded.
pub struct SkeletonCapacity { pub n_leaves: usize, pub n_binary: usize, pub n_edge: usize }

pub fn verify_patricia_skeleton<Value: IValue>(
    context: &mut Context<Value>,
    witness: &SkeletonWitness,   // carries the trie height
    capacity: &SkeletonCapacity,
) -> VerifiedSkeleton;
// { root: HashValue<Var>,                      — caller binds to the public input
//   leaves: Vec<VerifiedLeaf>,                 — { key, value, is_present } per batch key
//   siblings: Vec<SkeletonUnitVars> }          — for step 2's shared-sibling argument
```

Note this replaces the `capacity: usize` of DESIGN.md:92, which propagates to
`verify_patricia_update` and every caller.
