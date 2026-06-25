# Unbalanced Merkle commitment — fixed-circuit design

## Goal

One **fixed** circuit (fixed AIR / gate structure) that verifies `verify_merkle_commitment`
for *any* binary-tree shape over up to `L` leaves — from a fully balanced tree
(`L = 2^(k-1)` leaves at one depth) to a degenerate caterpillar (one leaf per layer) — at a
cost that depends only on `L`, never on the tree's depth. The same circuit must also handle
`n < L` real leaves.

The current `compute_merkle_root` builds a full balanced topology of `next_pow2(n)` and prunes
padding with the `OptionalNode` promotion selectors. That makes the gate structure depend on the
shape *and* costs `O(2^maxdepth)`: a deep, skinny tree blows up. We replace the layer-by-layer
construction with a **wiring-by-multiset** argument: the shape lives entirely in the witness,
the gate structure is fixed, and cost is `O(L)`.

## Capacity parameters

- `L` — leaf-slot capacity, a power of two (`= 2^(k-1)`).
- `M = L - 1` — internal-node-slot capacity. (Every binary tree with `m` leaves has exactly
  `m - 1` internal nodes, so `M = L - 1` slots suffice for any shape with `≤ L` leaves.)
- `n` — the actual number of real leaves, `1 ≤ n ≤ L`. **Public.**

There are **no enable bits and no multiplicities**: every leaf slot and node slot is always
active. `n < L` is handled purely by padding the leaf list with the all-zero hash.

## Two ideas doing the work

### 1. Shape is wiring, not topology

In the wiring view there are no "layers", only producers and consumers. A node's two child
hashes are witness; its output is derived. The multiset argument ties each produced hash to its
single consumption, *regardless of slot position*. So a balanced tree and a caterpillar are the
**same circuit with different witness values** — the prover just feeds different child hashes
into the same node slots.

A "promoted" *real* leaf (reference convention: an odd node copied up unchanged) is simply a leaf
wired in at a shallower depth; the hash that consumes it is an ordinary two-input `hash_node`.
No special handling needed for real-leaf promotion.

### 2. `n < L` via zero-padding + absorbing nodes

Pad the leaf list to `L` with the constant all-zero hash `Z`. To keep the root equal to the
promotion-convention root of the *real* leaves (`merkle_root_blake2s`), a padding `Z` must vanish
rather than become a real leaf. So every node uses a **data-driven absorbing rule**:

```rust
// Z is the all-zero HashValue constant.
out_i = if is_zero(right_i) { left_i } else { hash_node(left_i, right_i) }
```

- both children non-zero → ordinary hash (the `n = L` case, no zeros present);
- right child is `Z` → copy the left child up (promotion / consume one padding zero);
- both `Z` → `out = left = Z` (a still-empty subtree stays empty).

This is the old `is_some` selector, but **derived from the value**, so there is no separate flag
to store or constrain. (`is_zero` is taken on the *right* child only; the prover canonicalizes
absent siblings to the right. Placing `Z` on the left would just fail to reproduce `root`, so it
is not a soundness concern. A symmetric `is_zero(left) ? right : …` variant is also fine if you
prefer not to rely on canonicalization.)

## Data model

```rust
/// One internal-node slot: the prover supplies both child hashes; `out` is derived.
struct Node {
    left:  HashValue<Var>,   // witness — left child hash (present child under canonicalization)
    right: HashValue<Var>,   // witness — right child hash, or the all-zero hash Z if absent
    out:   HashValue<Var>,   // = is_zero(right) ? left : hash_node(left, right)
}
```

The old `OptionalNode { value, hash, is_some }` is retired: the selector is now the data-driven
`is_zero(right)`.

| Quantity                              | Visibility | Notes                                                   |
|---------------------------------------|------------|---------------------------------------------------------|
| `root` (R)                            | public     | the claimed commitment; bound to the statement          |
| `n`                                   | public     | number of real leaves; the rest are zero-padded         |
| leaf hashes `j < n`                   | witness*   | `= hash_leaf_qm31(value_j)`; *value public or private   |
| leaf slots `j ≥ n`                    | constant   | the all-zero hash `Z`                                    |
| `Node.left`, `Node.right` (all slots) | witness    | the resolved tree, prover-filled                        |

## The constraint

Per slot (all `L` leaf slots and all `M` node slots — the gate structure is fixed):

- `leaf_hash_j = hash_leaf_qm31(value_j)` for `j < n`; `leaf_hash_j = Z` for `j ≥ n`.
- `node_i.out = is_zero(node_i.right) ? node_i.left : hash_node(node_i.left, node_i.right)`.

Then one **strict** multiset identity:

```
permute_hash_values(
    // LHS (consumed side): the root, plus every node's two child inputs
    lhs = [ root ]
        ++ [ node_i.left  for i in 0..M ]
        ++ [ node_i.right for i in 0..M ],
    // RHS (produced side): every (real or padding) leaf, plus every node's output
    rhs = [ leaf_hash_j for j in 0..L ]
        ++ [ node_i.out  for i in 0..M ],
)
```

**Length check (fixed, shape- and `n`-independent):**
`|lhs| = 1 + 2M = 2L - 1` and `|rhs| = L + M = 2L - 1`. Always equal — no enable bits required.

No explicit `eq(root, …)` is needed: `root` is the only LHS entry that is not a node input, so
the balance forces exactly one produced value (the actual tree root) to equal it.

## Why it telescopes

Every node's `out` (RHS) is consumed exactly once as a child of its parent (LHS) → cancels. The
root's `out` cancels with the explicit `root`. Each real leaf (RHS) cancels with its consumption
as a child (LHS). Each padding `Z` (RHS) is consumed by an absorbing node (LHS `right = Z`), and
that node's `out = left` keeps the present child flowing. Both sides reduce to ∅ **iff** the
witness describes a valid promotion-convention tree rooted at `root` over exactly the `n` real
leaves.

## Prover witness filling

Given the `n` real leaves:
1. Hash them into leaf slots `0..n`; set slots `n..L` to `Z`.
2. Build the real promotion tree (`n-1` genuine `hash_node` nodes); assign them to node slots in
   any order, `left`/`right` = children's hashes (`hash_node` form).
3. Consume the `L-n` padding zeros with `L-n` absorbing nodes (`right = Z`, `out = left`),
   chaining the partial root upward. `(n-1) + (L-n) = M`, so all node slots are used.
4. Set `root` to the final node's `out`.

## Worked examples (L = 4, so M = 3)

- **Balanced, n=4** `[a,b,c,d]`: `n1=H(Ha,Hb)`, `n2=H(Hc,Hd)`, `root=H(n1,n2)`. No zeros, all
  plain hashes.
- **Caterpillar, n=4** `[a,b,c,d]`: `n1=H(Ha,Hb)`, `n2=H(n1,Hc)`, `root=H(n2,Hd)`. *Same circuit,
  different witness.*
- **n=3** `[a,b,c]`, leaves `[Ha,Hb,Hc,Z]`: `n1=H(Ha,Hb)`, `n2=H(n1,Hc)`, `n3=absorb(n2,Z)=n2`,
  `root=n2`. Matches `merkle_root_blake2s([a,b,c])`.
- **n=1** `[a]`, leaves `[Ha,Z,Z,Z]`: three absorbing nodes chain `Ha` up
  (`n1=n2=n3=Ha`), `root=Ha`. Multiset balances to `{Ha×4, Z×3}` on both sides.

## Soundness

Assumptions: Blake2s is collision- and preimage-resistant; `permute_hash_values` is binding
(below); `root` and `n` are genuine public inputs; **no real leaf hash or internal node hash
equals the all-zero `Z`** (probability `≈ (#nodes)/2^256`).

**Claim.** The constraints hold ⇒ `root` is the reference-convention Merkle root of the `n` real
leaves, except with negligible probability.

1. **Local correctness.** `out_i` and `leaf_hash_j` are enforced, so every produced value is a
   genuine hash (or absorbing copy) of its declared inputs.
2. **Bijection.** The multiset identity ⇒ a producer→consumer bijection between
   `{leaves} ∪ {node outputs}` and `{node inputs} ∪ {root}`: a hash DAG whose only unconsumed
   output is `root` and whose sources are exactly the `n` real leaves plus `L-n` copies of `Z`.
3. **Tree, not forest/cycle.** A second unconsumed root would break the balance (only `root` is
   unmatched on LHS). A cycle requires `h_i = hash_node(h_{i-1}, …)` to close on itself — a hash
   cycle, infeasible under preimage-resistance. So the DAG is a single tree rooted at `root`.
4. **Padding vanishes.** Under the no-real-hash-equals-`Z` assumption, `is_zero(right)` fires
   exactly on padding edges, so the `Z`s are promoted away and the root depends only on the real
   leaves, reproducing the promotion convention.
5. **Shape/order uniqueness.** `hash_node` is not commutative and each node fixes its own
   `(left,right)`. For a fixed real-leaf set, at most one tree yields a given `root` (a second is
   a Blake2s collision). Hence the only satisfying witness is the intended/canonical one.
6. **Binding compression.** `permute_hash_values` compares 8-word hashes via a collision-
   resistant encoding under a **post-commitment** challenge (the logup challenge `z`). Value-
   multiset equality ⇒ true `HashValue`-multiset equality except w.p. `~ 8 / |challenge space|`.
   A fixed/constant compression coefficient would be unsound.

### What is and isn't enforced — read carefully

- The circuit proves "`root` is the root of **some** promotion tree over the real-leaf
  **multiset** (zero-padded)." Tree **shape** and leaf **order** are pinned only indirectly,
  through `root` + collision-resistance. Correct when `root` is the commitment the verifier
  already trusts; enforcing a *specific* shape independent of `root` would need extra constraints.
- Repeated values (a promoted/absorbed hash appears on both sides more than once) are expected;
  the soundness argument is **multiplicity-aware** and does not rely on all-distinct labels.
- `n` is public, so the real/padding split is fixed by the statement — a prover cannot disable a
  real leaf (every real leaf hash is a fixed RHS entry that must be consumed).

## Assumed utility contract

```rust
/// Enforces that `lhs` and `rhs` are equal as multisets of `HashValue`s.
///
/// Soundness: each HashValue is folded to a single field element with a post-commitment
/// challenge (the logup lookup challenge) before the multiset check, so value-collisions occur
/// with negligible probability. Repeated values (multiplicity > 1) are handled.
fn permute_hash_values<Value: IValue>(
    context: &mut Context<Value>,
    lhs: &[HashValue<Var>],
    rhs: &[HashValue<Var>],
);
```

(See the separate note on realizing this over the QM31Ops logup — either widen the permutation
row to 8 words, or compress in-circuit with the post-commitment challenge.)

## Cost

- `M = L-1` node slots (each an `is_zero` + conditional `hash_node`) + `L` `hash_leaf` gates.
- One `permute_hash_values` over `2·(2L-1)` hash entries → `O(L)` after compression.
- **Total `O(L)`, independent of depth.** A depth-`L` caterpillar costs the same as a depth-
  `log L` balanced tree — the whole point.

## Test plan

- Round-trip success at `n = L`: balanced and caterpillar (same circuit, vs `merkle_root_blake2s`).
- Success across `n ∈ {1,2,3,4,5,7,8,…}` vs `merkle_root_blake2s` (port existing cases).
- Duplicate leaf values; and a real leaf placed adjacent to padding.
- Negative: wrong `root`; tampered leaf; tampered wiring (swap a child); forest attempt (two
  roots); padding `Z` placed where a real leaf belongs; left/right swap on a node.

## Approval checklist (soundness-critical)

1. Governing logic: Blake2s Merkle convention (`stark_verifier::merkle`) + logup permutation
   (QM31Ops component).
2. Invariant: `root` = reference (`merkle_root_blake2s`) root of the `n` real leaves.
3. Preservation: multiset telescoping + CR + no-real-hash-equals-`Z` (§Soundness).
4. Verifying tests: §Test plan.
5. Open: realization of `permute_hash_values` (row-widen vs in-circuit compress) — needs the
   compression-challenge decision and, if row-widen, a stwo-air-infra regen + review.
