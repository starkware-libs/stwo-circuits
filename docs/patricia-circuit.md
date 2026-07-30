# The Patricia Circuit — Statement, Design, Soundness

A self-contained description of the standalone Patricia update circuit
(`crates/unpacker/src/patricia/`): what it proves, how it is built, and why it is sound.
Companion documents: `payments-circuit-design.md` (decisions and delivery plan) and
`payments-circuit-soundness.md` (the method and the open-gap ledger). This one is about the
circuit itself.

## 1. The statement

`verify_patricia_update` proves: **for public `prev_root`, `new_root`, and a batch of `K` rows
`(key, prev_value, new_value)` — given that `prev_root` commits to a canonical trie, and given
that no two live rows share a key — there is a canonical trie `T_new` with
`hash(T_new) = new_root` such that every live row's key maps to `prev_value` in the prev trie
and to `new_value` in `T_new` (value `0` meaning *proven absent*), and every key outside the
batch maps identically in both tries.** Live-key distinctness is a stated precondition, owed by
the batch producer (§6.5) — the circuit does not enforce it, and without it the absence clauses
fail (§6.5, the complementary-pair finding).

The four operations — overwrite `v → v′`, insert `0 → v′`, delete `v → 0`, and the no-op
`(0, 0)` — are just the four value patterns; the circuit has no operation logic. The one
weakening is stated, not hidden: a `(0, 0)` row proves "unchanged", not "absent" (§6.5).

Canonicity of `T_new` is what upgrades "a trie hashing to `new_root`" into "*the* state":
canonical form makes the map → root correspondence injective, so `new_root` is the unique
commitment to `apply(prev_state, rows)`.

## 2. Why Patricia rather than a plain Merkle tree

The keyspace is 2^251; the populated key count `N` is maybe 2^20–2^30. A binary Merkle tree over
that keyspace makes every touched key cost **251 hashes** — the tree height — regardless of `N`;
sparse-Merkle default-node tricks save prover work off-chain but not in-circuit hash gates unless
special-case branches are added. Path compression changes the asymptotics: single-child runs
collapse into **edge nodes** carrying their compressed bits, so a walk crosses only the
~log2(N) *real* branch points. Measured on our tries: at N = 10^6, K = 256, the whole batch
skeleton is ~3.2k binaries + ~0.4k edges — roughly **K·log2(N/K) + 1.6K hashes instead of
K·251**, a ~7–9× cut in Blake gates, and Blake is essentially the entire circuit cost. Batching
compounds it: the skeleton is the *union* of walks, so shared prefixes are paid once.

Patricia also gives absence a shape. In canonical form an absent key's walk *dies at a diverging
edge* — a specific, checkable structure — which is what lets inserts prove non-membership instead
of assuming it. And canonicity gives uniqueness of encoding, which a plain node-by-node Merkle
structure does not: without it, one state could hash to many roots.

The price of compression is the two gadgets Merkle would not need: a variable 256-bit shift for
edge positions and an add-with-carry for the edge hash's additive length. Both cost ~10² cheap
gates per edge slot — noise next to the ~10³-gate Blake each slot already emits.

The node encoding (the production convention, pinned by golden vectors):

```
hash2(x, y) = LE_u32s(blake2s(LE32(x) ‖ LE32(y))) & (2^251 − 1)
binary      = hash2(left, right)
edge        = hash2(bottom, edge_path) + ℓ        (ℓ ADDED, not hashed)
leaf        = value,   empty = 0
```

Canonical form: edges maximally merged (an edge's bottom is never an edge, ℓ ≥ 1), a binary's
children both non-empty (a lone child folds into an edge), no zero-valued leaf, the empty subtree
always the all-zero hash.

## 3. The tables

Everything is built from one atom, the **unit**: 18 u32 words, `(height, path[8], kind, hash[8])`
— "a node of this kind at this position with this subtree hash." Position is self-describing:
`path = key >> height`, so the root has path 0 and a leaf's path *is* its key. Kind tags:
Padding 0, Leaf 1, Binary 2, Edge 3, Opaque 4 (the sibling tag — P7).

Five tables per fold, sizes fixed by the capacity `(K, n_binary, n_edge)`:

| table | count | fields | produces | consumes |
|---|---|---|---|---|
| leaf slots | K | key (shared), value | `{0, key·p, p, value}` | — |
| binary slots | n_binary | 2 child units, parent path | derived out | both children |
| edge slots | n_edge | bottom unit, parent path, ℓ (8 bits), edge_path | derived out | bottom |
| sibling slots | n_binary − K + 1 | height, path, hash (kind derived) | the unit | — |
| root entry | 1 | root hash, kind | — | `{H·live, 0, kind, root}` |

The update instantiates the middle three **twice** (prev and new) but the shared things **once**:
each row's key vars and each sibling unit are guessed a single time and fed into both folds —
sharing is variable identity, not a constraint (P6).

## 4. The relations

**Local relations** (algebraic, within a slot). Every slot derives its own liveness flag
`is_live = 1 − is_zero(slot's guessed words)`, and the all-zero inert tuple satisfies every
relation, so padding needs no branch:

- *Leaf*: `is_present = 1 − is_zero(value)`; the multiset entry gates key and kind by it. That
  single derivation is all of P4 — an absent slot's entry *is* the inert unit.
- *Binary*: heights equal and `out.height = h + is_live`; positions `left.path = 2·out.path`,
  `right.path = 2·out.path + is_live` (16-limb carry chains); children live and non-empty;
  `out.hash = truncate251(blake2s(left.hash ‖ right.hash)) · is_live`; `out.kind = 2·is_live`.
- *Edge*: `ℓ ≥ 1` when live; `edge_path < 2^ℓ` (alignment); bottom's kind ≠ Edge (maximal
  merging) and Opaque only at height 0 (P7); `out.height = bottom.height + ℓ`;
  `bottom.path = out.path·2^ℓ + edge_path` (the barrel shifter: limb-rotate by ℓ/16, scale by
  2^(ℓ mod 16), carry-free add); `out.hash = truncate251(blake2s(bottom.hash ‖ edge_path)) + ℓ`
  with a limb carry chain; `out.kind = 3·is_live`.
- *Sibling*: hash ≠ 0 when live; `kind = 4·is_live` — derived, so a sibling can claim nothing.
- *Root entry*: `is_empty = is_zero(root)`; kind is 0 iff empty, else in
  {Binary, Edge, Opaque} ({Leaf, Opaque} at height 0). That derivation is all of P5.

Underneath sit the limb gadgets: every 256-bit value is sixteen range-checked 16-bit limbs (field
coordinates cannot cross bit boundaries), carries/borrows/splits are boolean or ranged guesses
uniquely determined by the relation they close, and the 251-bit truncation splits the digest's
top limb as `mid11 + top5·2^11` and drops `top5`.

**Global relations**: one permutation argument per fold — *consumed = produced as multisets of
18-word units*, exact cardinality. The counts balance identically because sibling slots are
derived as `n_binary − K + 1`; even the inert padding flow nets to zero, with each absent row's
inert contribution absorbed by one padded binary slot. This is the wiring: parent and child are
connected only by naming the same unit value, never topologically.

### 4.1 The lookup relations, precisely

Three layers:

**The unit multisets (the design's own).** Two permutation relations per update, one per fold:
`consumed ≡ produced` over 18-word units — `{2B children, E bottoms, root entry}` against
`{K leaf entries, B outs, E outs, S siblings}`, cardinalities equal by the derived
`S = B − K + 1`. Implementation (`permute_units`): words are packed `(low16, high16, 0, 0)` with
the `u` coordinate free; input unit `j`'s words are tagged `+(0,0,j,0)`, each output unit guesses
one base-field source index shared by its 18 words, and **one `Permutation` gate per word-column**
(18 per fold) asserts column-wise multiset equality of tagged values. The shared tag lifts 18
column equalities into unit-level matching. Preconditions, both discharged: clean `u`/`iu`
coordinates on every word (ranged guesses, Blake outputs, coordinate-0 arithmetic only) and a
base-field-constrained source index. At the AIR each `Permutation` gate is a logup fraction-sum.

**The variable-wiring relation (the DSL's).** The whole circuit is one lookup argument over
`(variable, value)` pairs: gates *yield* outputs and *use* inputs, and logup forces every use to
see its yielder's value, under the invariant that each variable is yielded exactly once (guesses
get their yielding gates at finalization). This is what makes shared-by-identity (P6) mean
shared-by-value.

**Table lookups in the primitives.** Every U16 guess is range-floored by the `M31ToU32` relation;
Blake's XORs are checked against fixed bitwise-XOR tables (`VerifyBitwiseXor` 4/7/8/9/12,
`TripleXor32`) with logup multiplicities.

Division of labor: layer three makes individual words honest, layer two makes the dataflow graph
honest, and layer one is where the statement lives — the trie exists only as those two multiset
equalities, which is why §5–§6 reason entirely at that layer.

## 5. How they come together

The derivation argument is a single induction with four load-bearing facts:

1. **Top-down pinning.** The root entry's height, path, and hash are public/constant. Every
   consumed unit must cancel against a produced unit; the producing slot's local relations then
   determine its *children's* positions and heights as exact functions of the parent, and Blake
   binding pins the children's hashes. There is deliberately no per-unit path range check — root
   path 0 plus the doubling/shift relations pin every descendant path exactly, which is stronger.

2. **No cycles, one exit.** Live outputs sit strictly above their inputs (binary +1, edge +ℓ,
   ℓ ≥ 1), so live chains climb monotonically and must terminate at the single root entry. A
   fabricated unit is produced by nobody or consumed by nobody — the multiset fails.

3. **The duplicate-position lemma.** Consumers take children at *sibling* positions (2q vs
   2q + 1), never the same one — so two live units at one position force two parallel parent
   chains that both need the one root entry; one dangles. This is what makes absence *proof*:
   claim a present key absent and insert it, and the new fold must consume both the shared
   sibling hiding the real leaf *and* the inserted leaf's ancestor chain through the same
   positions. Deletes are the mirror: the prev fold's walk to the key excludes any sibling on its
   path, so the new side's divergence is genuine. The lemma also rejects duplicate live keys for
   free.

4. **Shared values, one truth.** Sibling units and row keys are single vars in both folds, so
   "untouched state changed" and "the two sides disagree on a key" are not expressible witnesses
   at all — the multiset never even has to reject them.

Canonicity closes the loop: the edge rules plus non-empty binary children mean any satisfying new
fold describes a *canonical* trie — so `new_root` is the unique encoding of the updated map, and
the "prev is canonical" premise regenerates for the next update.

Running through everything, one discipline: **every case distinction is derived from a bound
value** — presence from the value, emptiness from the root, liveness from the slot's own words.
There is no witnessed flag anywhere, so a prover never gets to choose which statement to prove.

## 6. Soundness

Soundness is the universal claim: *no* assignment satisfying the constraints proves a false
statement. This section walks the claim in full — the assumptions, the argument for each part of
the statement, and the exact boundary of what is not claimed. A Lean 4 formalization of the statement
and this argument lives in `lean/` — same assumption style as the cairo-lang Patricia
verification, **all theorems proven** (no `sorry`s; axiom footprint `propext`,
`Classical.choice`, `Quot.sound`). The formalization is also what found the two statement
corrections recorded in §6.5 and `lean/README.md`; the modeling boundary (Lean model vs Rust
gates) is stated there and is the part only human review covers.

### 6.1 Assumptions

Two, and only two:

**A1 — blake2s collision resistance, including a shifted variant.** Binary hashes need plain
collision resistance of `hash2` (the 251-bit truncation costs 5 bits of output, giving ~125-bit
collision security). Edge hashes need slightly more, because the length is *added*: an edge
stores `e = hash2(bottom, edge_path) + ℓ`, so a prover who could find two openable preimages
whose truncated digests differ by an offset `d ∈ [−251, 251]` could open one stored `e` two ways
— `e = t + ℓ = t′ + ℓ′` with `ℓ − ℓ′ = d`. Call this a *shifted collision*: for each of the
~503 offsets, a collision on `hash2(·) − hash2(·) = d`. This enlarges the target set by a factor
of ≤ 503 ≈ 2^9, i.e. costs at most ~9 bits against a 251-bit digest — negligible, but it is the
honest assumption and it is what the edge-slot argument actually uses. (Both variants are also
what the golden vectors pin: the circuit computes the *production* convention, so a collision
here is a collision against the production trie, not against a circuit-local encoding.)

**A2 — `prev_root` commits to a canonical trie.** The statement is conditional on the prev trie
being canonical; two things ground the condition. *Genesis*: the chain of updates starts either
from the empty trie (`root = 0`, canonical by definition — the nonce trie's actual genesis) or
from a root computed by the reference builder, which constructs canonical form. *Induction*: the
statement itself proves `T_new` canonical, so the condition regenerates for the next update.
This is why P7 mattered: before the Opaque tag, a mis-kinded sibling in the delete-merge shape
could have made `new_root` a non-canonical encoding of the correct map — a true state, wrongly
encoded — silently breaking the induction for every later block. With P7 the new side's
canonicity is fully in-circuit and the induction is self-sustaining.

Nothing else is assumed. In particular there is no trusted setup, no assumption on the prover,
and no assumption that witnesses come from our extractor — the argument quantifies over *all*
satisfying assignments.

### 6.2 What is proven, claim by claim

**(a) Every opened unit is a real node of the committed trie.** By induction downward from the
root entry (§5.1–5.2): the root entry's hash is the public root; a binary out's Blake preimage
pins `(left.hash, right.hash)` under A1; an edge out pins `(bottom.hash, edge_path, ℓ)` under A1
including the shifted variant — given the pinned `e`, any satisfying `(t′, ℓ′)` split with
`ℓ′ ≠ ℓ` is a shifted collision, so the split is unique and even ℓ, a bare 8-bit witness, is
hash-determined. Positions and heights are exact functions of the parent (doubling and shift
relations over range-checked limbs, with `edge_path < 2^ℓ` making the shift-add injective in the
downward direction). So every live unit reachable from the root entry carries the height, the
position, and the hash of the actual node of the committed trie at that place. Units *not*
reachable from the root cannot exist: they would head a chain with no consumer (§5.2).

**(b) Presence claims are true.** A live leaf entry `{0, key, Leaf, value}` enters the fold and
must be consumed by the rooted structure; by (a) its consumer chain is the real trie, so the trie
holds exactly `value` at `key`. Conversely the value cannot be misreported: the leaf's unit *is*
the row's `value` vars, and the multiset admits no second unit at the same position (§5.3).

**(c) Absence claims on live rows are true — under the live-key-distinctness precondition
(§1).** Distinctness is load-bearing here: it guarantees no *other* live row plants a live leaf
entry at this key's position in either fold, which is what lets the walk-exclusion arguments
below speak for the whole fold. The insert direction: suppose row `(key, 0, v′)`
while the prev trie holds `key ↦ v ≠ 0`. The prev fold does not walk to `key` (the row is absent
on that side), so the real leaf lies inside some sibling `S` at a prefix position of `key` — and
by (a) applied to the prev fold, `S`'s hash is the real subtree there. `S` is shared: the new
fold consumes the very same vars. But the new fold must also consume the live leaf
`{0, key, Leaf, v′}` and fold it to `new_root`, building an ancestor chain through `key`'s
prefix positions — including `S`'s position. Two live units at one position cannot both be
consumed (§5.3), so no satisfying assignment exists. The delete direction is the mirror image:
row `(key, v, 0)` forces the *prev* fold to walk to the leaf, which by the same lemma excludes
any sibling at a prefix of `key`; the new fold's walk toward `key` must then terminate at a
diverging edge, and by (a) that divergence is the real structure of `T_new` — the key is
genuinely absent. A *fake* divergence (an edge whose compressed bits disagree with the trie) is
excluded by (a): the edge's `edge_path` is hash-pinned.

**(d) Untouched state is identical.** Not proven by a constraint — made *inexpressible* (§5.4).
Both folds consume the same sibling vars, so any subtree neither side opens contributes the same
hash to both roots; by canonical uniqueness, the same sub-map. Every position is either on a
walked path (covered by (b)/(c)) or under a shared sibling (covered here) — the frontier is
exhaustive because the multiset admits no third kind of unit.

**(e) `T_new` is canonical, hence `new_root` unique.** Every live slot of the new fold enforces
its canonicity rule locally: ℓ ≥ 1, aligned compressed bits, no edge-over-edge, non-empty binary
children, no zero-valued live leaf. The one rule that touches an unopened subtree — an edge's
bottom is not an edge — is safe by P7: a bottom is either a slot output (kind derived, authentic)
or Opaque at height 0 (a leaf position, where "is it an edge" cannot arise). So the fold's
structure is canonical wherever it is opened, and inherited-canonical (from A2) wherever it is
not. Uniqueness of canonical encoding then gives injectivity: `new_root` determines the map.

**(f) The derived flags are functions, not choices.** `is_zero` is the standard two-constraint
gadget (`x·z = 0`, `x·w + z = 1`), whose output is uniquely determined in a field — so
`is_present`, `is_empty`, and every `is_live` are deterministic functions of already-pinned
values. The prover's only remaining freedoms are immaterial by symmetry: which physical slot
hosts which node (the multiset is order-free), which padded slot absorbs which inert unit (all
inert units are identical), and the values inside fully-inert padding (gated to the fixed tuple).

### 6.3 The completeness direction

Soundness alone would be satisfied by a circuit that rejects everything, so the companion claim
matters: every honest update has a satisfying assignment. The extractor constructs it — walks
both tries over the live keys, opens diverging-edge bottoms one binary level (P7), reconciles the
two sibling frontiers until they coincide (possible exactly because untouched subtrees are
identical in both tries), and pads to capacity. Capacity is the only availability condition:
`n_binary` must cover each side's live binaries plus one spare per absent-on-that-side row (the
padded slot that absorbs the inert contribution), and the derived sibling count then fits
automatically. An undersized capacity makes an honest update unsatisfiable — an availability
failure, never a soundness one.

### 6.4 What the prover controls, and why it does not matter

It is worth stating the attack surface positively. An unconstrained prover chooses: every unit
field of every slot (pinned by cancellation, §6.2a), every carry/borrow/split (uniquely
determined by ranged relations), ℓ and `edge_path` (hash-pinned), the root kinds (pinned by
cancellation and the tag rules), sibling contents (pinned by hash against `prev_root`; interiors
never claimed), slot assignment and padding placement (symmetric), and the permutation's source
indices (checked by the permutation itself). Every degree of freedom lands in one of three
buckets: *pinned*, *derived*, or *symmetric*. The witness tables in the module docs are the
per-guess ledger of which bucket each one is in; an entry in none of them would be an
incompletely constrained bug, which is exactly what the tables exist to surface.

### 6.5 Deliberately unproven

Stated so nothing downstream over-claims:

- **`(0, 0)` rows prove "unchanged", never absence** (P9). A row that writes nothing forces no
  walk, so a present-and-unchanged key can hide behind a `(0, 0)` claim. What survives: the key
  maps identically in both tries (it is either behind one shared sibling on both sides or
  genuinely diverged on both — one-side-hidden is excluded by the duplicate-position lemma). No
  value can be created or moved through it; the only rule it imposes on callers is *never treat
  a `(0, 0)` row as non-membership*. An insert-flavored consumer (the nonce trie: every write is
  `0 → 1`) never produces one, so its replay protection rests entirely on the proven case.
- **A both-sides-absent row's key is unbound.** Its key words are range-checked u32s and nothing
  more; the caller owes the binding of every key (in the payments stack, the batch-binding
  layer). A live row's key, by contrast, is pinned by the walked side's leaf cancellation.
- **Live-key distinctness is a precondition, not a circuit guarantee.** Two *same-side*
  duplicate live rows on one key are rejected (two live units at one position in one fold), but
  the **complementary pair** — a delete-shaped row `(k, v, 0)` plus an insert-shaped row
  `(k, 0, w)` — is *accepted*: each fold sees exactly one live entry at the position, and the
  pair jointly behaves as the overwrite `v → w` while each row's absence claim is false. Found
  by the Lean formalization (the statement was unprovable as previously written); pinned by
  `complementary_rows_on_one_key_act_as_an_overwrite`. Every consumer of row outputs must
  guarantee live-key distinctness — the dict squashing emits one row per key by construction,
  and nonce batches are insert-only (same-side, genuinely rejected). A live row plus a `(0, 0)`
  row on one key remain merely independent.
- **Sibling interiors.** A sibling's hash is pinned to the real prev subtree, but the circuit
  never looks inside it. Everything the statement claims is invariant to interiors; anything a
  caller wants to know about an untouched subtree must come from opening it in some update's
  batch.
