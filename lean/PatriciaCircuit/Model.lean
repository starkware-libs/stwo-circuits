import PatriciaCircuit.Crypto

/-!
# The circuit model (`docs/patricia-circuit.md` §3–§4)

A faithful integer model of the circuit's *layer 1*: units, the five slot tables with their
local relations, and the fold — produced ≡ consumed as multisets (`List.Perm`).

## Modeling boundary

This is the statement level at which the soundness argument reasons (doc §4.1): we model each
slot's constraints as their integer semantics — heights and kinds as `Nat`s, paths and hashes
as `Nat`s below `2^256` with the circuit's mod-`2^256` arithmetic written explicitly. What is
*assumed* of the layers below, justified there and not re-proven here:

* every guessed word satisfies its range constraint (layer 3: `M31ToU32`, boolean gates), so
  unit fields denote integers within their widths;
* the limb gadgets implement the stated integer relations (doubling, shift-add, alignment,
  truncation, add-with-carry — unit-tested in `word_gadgets_test.rs`);
* the DSL's variable-wiring lookup gives each variable one value (layer 2), so sharing by
  variable identity is sharing by value — the model expresses P6 by *construction* (one
  sibling list, one key list, used by both folds);
* `permute_units` enforces exact multiset equality of 18-word tuples — modeled as
  `List.Perm` over `SUnit`.
-/


namespace PatriciaCircuit

/-- Kind tags, as the integers the unit tuples carry. -/
def tagPadding : Nat := 0
def tagLeaf : Nat := 1
def tagBinary : Nat := 2
def tagEdge : Nat := 3
def tagOpaque : Nat := 4

/-- One multiset atom: the circuit's 18-word unit `(height, path, kind, hash)`. -/
structure SUnit where
  height : Nat
  path : Nat
  kind : Nat
  hash : Nat
deriving DecidableEq

/-- The unit filling padded slots and gated-out contributions: all words zero. -/
def SUnit.inert : SUnit := ⟨0, 0, 0, 0⟩

/-- A leaf slot (P4): the shared key vars and this side's value. -/
structure LeafSlot where
  key : Nat
  value : Nat
deriving DecidableEq

namespace LeafSlot

/-- Presence is derived: `value ≠ 0`. -/
def isPresent (s : LeafSlot) : Prop := s.value ≠ 0

/-- The multiset contribution `{0, key·p, p, value}` — exactly the inert unit when absent. -/
def entry (s : LeafSlot) : SUnit :=
  if s.value = 0 then .inert else ⟨0, s.key, tagLeaf, s.value⟩

end LeafSlot

/-- A binary slot: two guessed child units and the guessed parent path. -/
structure BinarySlot where
  left : SUnit
  right : SUnit
  outPath : Nat
deriving DecidableEq

namespace BinarySlot

/-- Liveness is derived from the children's words: live iff not both inert. -/
def isLive (s : BinarySlot) : Prop := ¬(s.left = .inert ∧ s.right = .inert)

instance : DecidablePred isLive := fun _ => by unfold isLive; infer_instance

def liveBit (s : BinarySlot) : Nat := if s.isLive then 1 else 0

/-- The local relations of `binary_slot` (`skeleton_circuit.rs`), as integer semantics. The
position relations are the circuit's mod-`2^256` carry chains. -/
def ok (s : BinarySlot) : Prop :=
  s.left.height = s.right.height ∧
  s.left.path = (2 * s.outPath) % wordModulus ∧
  s.right.path = (2 * s.outPath + s.liveBit) % wordModulus ∧
  (s.isLive → s.left ≠ .inert ∧ s.right ≠ .inert ∧ s.left.hash ≠ 0 ∧ s.right.hash ≠ 0)

/-- The derived output unit: height and path ungated (the inert case zeroes them through the
liveness of the whole tuple), kind and hash `is_live`-gated. -/
def out (c : Crypto) (s : BinarySlot) : SUnit :=
  { height := s.left.height + s.liveBit
    path := s.outPath
    kind := tagBinary * s.liveBit
    hash := if s.isLive then c.hash s.left.hash s.right.hash else 0 }

end BinarySlot

/-- An edge slot: the guessed bottom unit, parent path, length bits, and compressed bits. -/
structure EdgeSlot where
  bottom : SUnit
  outPath : Nat
  length : Nat
  edgePath : Nat
deriving DecidableEq

namespace EdgeSlot

/-- Liveness covers the whole slot-local tuple. -/
def isLive (s : EdgeSlot) : Prop :=
  ¬(s.bottom = .inert ∧ s.length = 0 ∧ s.edgePath = 0)

instance : DecidablePred isLive := fun _ => by unfold isLive; infer_instance

def liveBit (s : EdgeSlot) : Nat := if s.isLive then 1 else 0

/-- The local relations of `edge_slot`: ℓ is eight bits; canonicity (ℓ ≥ 1 live, aligned
compressed bits, no edge-over-edge, Opaque bottoms only at height 0, non-empty bottom); the
§4.2 shift; and the §4.4 hash with the additive length. -/
def ok (s : EdgeSlot) : Prop :=
  s.length < 256 ∧
  (s.isLive → 1 ≤ s.length) ∧
  s.edgePath < 2 ^ s.length ∧
  (s.isLive → s.bottom.kind ≠ tagEdge) ∧
  (s.bottom.kind = tagOpaque → s.bottom.height = 0) ∧
  (s.isLive → s.bottom.hash ≠ 0) ∧
  s.bottom.path = (s.outPath * 2 ^ s.length + s.edgePath) % wordModulus

/-- The derived output unit; the hash is the truncated digest plus the length, gated. -/
def out (c : Crypto) (s : EdgeSlot) : SUnit :=
  { height := s.bottom.height + s.length
    path := s.outPath
    kind := tagEdge * s.liveBit
    hash := if s.isLive then c.hash s.bottom.hash s.edgePath + s.length else 0 }

end EdgeSlot

/-- The local relations of `sibling_slot` (P7): a sibling is fully inert, or carries the
derived Opaque tag and a non-empty hash — it claims nothing else. -/
def siblingOk (s : SUnit) : Prop :=
  s = .inert ∨ (s.kind = tagOpaque ∧ s.hash ≠ 0)

/-- One skeleton fold: the slot tables (`docs/patricia-circuit.md` §3), the claimed root, and
the guessed root kind. Leaf keys and siblings are supplied by the caller so the update circuit
can share them across its two folds (P6). -/
structure Fold where
  height : Nat
  leaves : List LeafSlot
  binaries : List BinarySlot
  edges : List EdgeSlot
  siblings : List SUnit
  root : Nat
  rootKind : Nat

namespace Fold

/-- The root entry (P5): `{H·live, 0, kind, root}`, emptiness derived from the root. -/
def rootEntry (f : Fold) : SUnit :=
  ⟨if f.root = 0 then 0 else f.height, 0, f.rootKind, f.root⟩

/-- The root entry's tag rules: the padding tag iff empty; a live root at height > 0 is a
binary, an edge, or a wholly untouched trie; at height 0 the single leaf or untouched. -/
def rootOk (f : Fold) : Prop :=
  (f.root = 0 → f.rootKind = tagPadding) ∧
  (f.root ≠ 0 →
    if f.height = 0 then f.rootKind = tagLeaf ∨ f.rootKind = tagOpaque
    else f.rootKind = tagBinary ∨ f.rootKind = tagEdge ∨ f.rootKind = tagOpaque)

/-- Everything the fold produces, in table order. -/
def produced (c : Crypto) (f : Fold) : List SUnit :=
  f.leaves.map LeafSlot.entry ++ f.binaries.map (BinarySlot.out c) ++
    f.edges.map (EdgeSlot.out c) ++ f.siblings

/-- Everything the fold consumes: both children per binary, the bottom per edge, and the root
entry. -/
def consumed (f : Fold) : List SUnit :=
  f.binaries.flatMap (fun s => [s.left, s.right]) ++ f.edges.map EdgeSlot.bottom ++
    [f.rootEntry]

/-- A satisfying assignment of one fold: every local relation, and produced ≡ consumed as
multisets — the layer-1 lookup relation (`permute_units`). -/
def sat (c : Crypto) (f : Fold) : Prop :=
  (∀ s ∈ f.binaries, s.ok) ∧
  (∀ s ∈ f.edges, s.ok) ∧
  (∀ s ∈ f.siblings, siblingOk s) ∧
  f.rootOk ∧
  (f.produced c).Perm f.consumed

end Fold

/-- One update row `(key, prev_value, new_value)`; `0` values mean absent (P4). -/
structure Row where
  key : Nat
  prevValue : Nat
  newValue : Nat

/-- A row is live unless it is the `(0, 0)` no-op (P9). -/
def Row.isLive (r : Row) : Prop := ¬(r.prevValue = 0 ∧ r.newValue = 0)

/-- The update assignment: shared rows and shared siblings (P6 — the sharing is by
construction, mirroring the shared vars), and per-side slots and roots. -/
structure UpdateAssign where
  height : Nat
  rows : List Row
  siblings : List SUnit
  prevBinaries : List BinarySlot
  prevEdges : List EdgeSlot
  prevRoot : Nat
  prevRootKind : Nat
  newBinaries : List BinarySlot
  newEdges : List EdgeSlot
  newRoot : Nat
  newRootKind : Nat

namespace UpdateAssign

def prevFold (u : UpdateAssign) : Fold :=
  { height := u.height
    leaves := u.rows.map fun r => ⟨r.key, r.prevValue⟩
    binaries := u.prevBinaries
    edges := u.prevEdges
    siblings := u.siblings
    root := u.prevRoot
    rootKind := u.prevRootKind }

def newFold (u : UpdateAssign) : Fold :=
  { height := u.height
    leaves := u.rows.map fun r => ⟨r.key, r.newValue⟩
    binaries := u.newBinaries
    edges := u.newEdges
    siblings := u.siblings
    root := u.newRoot
    rootKind := u.newRootKind }

/-- A satisfying update assignment: both folds satisfied over the shared rows and siblings. -/
def sat (c : Crypto) (u : UpdateAssign) : Prop :=
  (u.prevFold).sat c ∧ (u.newFold).sat c

end UpdateAssign

end PatriciaCircuit
