# Patricia Trie Update Circuit — Design

Status: PR 1 (reference implementation + shared gadgets) implemented; PRs 2–3 (circuits) upcoming.

> **Partly superseded.** The slot counts in "Fixed topology" below understate the binary-slot and
> sibling counts by a factor of ~`log2(N/K)` (~12× at 1M accounts), the edge-slot position check
> hides a 256-bit variable shift, and the PR-2/PR-3 split does not fit an insert-dominated
> workload. See [`docs/payments-circuit-design.md`](../../../../docs/payments-circuit-design.md)
> §4 for measurements and corrected interfaces before implementing PRs 2–3.

## Goal

An in-circuit *update proof* for a Starknet-style Patricia trie over blake2s: prove that applying
a batch of writes `{key → new_value}` to the key-value map committed by `old_root` yields exactly
`new_root`, touching nothing else.

- **Reads** come free: an entry with `new = old` proves membership without changing anything.
- **Inserts** are writes to a previously-absent key.
- **Deletes** are writes of the value `0`.

## Trie specification (implemented in [`reference.rs`](reference.rs))

- **Key space**: `height`-bit keys (`height ≤ 256`). Keys, values, and hashes are all `Word256` =
  eight little-endian `u32` words (word 0 least significant). Key bit `h − 1` selects the child
  (0 = left, 1 = right) at a node of height `h`; leaves sit at height 0.
- **Hash rules**:

  | Node | Hash |
  |------|------|
  | Empty subtree | `0` (all-zero words) |
  | Leaf (height 0) | its value (non-zero for present keys) |
  | Binary (height `h ≥ 1`) | `blake2s(left ‖ right)` — 64 bytes |
  | Edge (height `h`, length `1 ≤ ℓ ≤ h`) | `blake2s(bottom ‖ path ‖ ℓ)` — 68 bytes |

  An edge compresses `ℓ` single-child levels down to a non-empty `bottom` at height `h − ℓ`;
  `path` holds the `ℓ` compressed key bits LSB-aligned (path bit `j` is key bit `h − ℓ + j`,
  bits `≥ ℓ` are zero). A leaf value of `0` means the key is absent.

- **Canonical form** — the map → root mapping must be *injective*:
  - edges are maximally merged: an edge's `bottom` is never itself an edge, every edge has
    length `≥ 1`;
  - a binary node's children are both non-empty (a lone child folds into an edge);
  - the empty subtree is always the all-zero hash, never a hashed encoding of "empty".

  This uniqueness is what update soundness leans on: without it the same state could hash to two
  different roots via shape-only rewrites, and "only these keys changed" would be ill-defined.

## Circuit design (PRs 2–3)

The statement is existential:

> There exists a set of untouched sibling subtrees such that the trie assembled from
> (batch keys → old values) + siblings hashes to `old_root`, **and** the trie assembled from
> (batch keys → new values) + the **same** siblings hashes to `new_root`.

It reuses the unpacker's **wiring-by-multiset** architecture (see
[`unpacker.rs`](../unpacker.rs)) rather than per-key authentication paths:

- **Units, not topology.** Every produced node is a unit `(height, path, kind, hash)` flowing
  through one `permute_units` multiset identity that ties each production to exactly one
  consumption. Carrying `path` in the unit binds a leaf's *position* to its key (the unpacker
  had no keys, so it didn't need this); `kind` lets an edge slot enforce "my bottom is not an
  edge" (canonicity).
- **Fixed topology.** A batch capacity `K` (power of two) fixes the slot counts — `K` leaf
  slots, `K − 1` binary slots, ~`2K` edge slots per side — independent of trie height, batch
  content, or shape. Padding slots use guessed zeros and copy-up/no-op rules, exactly like the
  unpacker, so the gate structure depends only on `(K, height)`. Depth-independence holds
  because a Patricia skeleton over `K` keys has O(K) nodes — edges compress the long runs.
- **Two skeletons, one sibling set.** The old and new skeletons are built independently (their
  shapes can differ — inserts and deletes split and merge edges), but the guessed sibling units
  are shared between them through the multiset. A sibling is an untouched subtree hanging off
  the touched paths; sharing it forces "everything off the batch keys is byte-identical in both
  tries".
- **Per-slot constraints.** Binary slots check equal child heights and
  `child.path = 2·parent.path + bit` (word-wise arithmetic with `extract_bits` carries); edge
  slots check length bounds and path-bit alignment; both emit the corresponding blake gate.

The top-level interface (PR 3):

```rust
pub struct PatriciaLeafUpdate<T> {
    pub key: HashValue<T>,
    pub old_value: HashValue<T>, // 0 = was absent
    pub new_value: HashValue<T>, // 0 = delete
}

pub struct VerifiedPatriciaUpdate {
    pub old_root: HashValue<Var>,
    pub new_root: HashValue<Var>,
    pub updates: Vec<PatriciaLeafUpdate<Var>>, // padded to capacity
}

pub fn verify_patricia_update<Value: IValue>(
    context: &mut Context<Value>,
    witness: &PatriciaUpdateWitness, // concrete skeletons + siblings (QM31)
    height: u32,
    capacity: usize,
) -> VerifiedPatriciaUpdate;
```

As with the unpacker, the circuit *returns* the resolved roots and updates; the caller is
responsible for binding them to its public inputs.

## Delivery plan

1. **`ilya/patricia-reference`** (done) — out-of-circuit reference implementation: canonical
   trie build, hashing, `apply_updates`, `is_canonical`. Serves as witness generator and test
   twin. Also extracts the unpacker's reusable gadgets (`select_hash`, `is_zero_words`,
   guessing helpers) into a shared [`gadgets`](../gadgets.rs) module.
2. **`patricia-skeleton`** — single-root circuit `verify_patricia_skeleton`: proves one root
   commits to the batch's (key → value) entries, exposing the sibling units. Witness extraction
   (touched skeleton + siblings from a reference trie) lands here, next to its consumer.
3. **`patricia-update`** — `verify_patricia_update`: two skeletons + shared siblings +
   in-circuit canonicity constraints. Negative tests: tampered sibling, non-canonical encoding,
   insert/delete edge splits, phantom write.
