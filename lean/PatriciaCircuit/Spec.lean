import PatriciaCircuit.Crypto

/-!
# The Patricia trie specification (`docs/patricia-circuit.md` §2)

The abstract, circuit-independent object: canonical height-indexed Patricia tries, their
hashes, and their key → value semantics. Mirrors the cairo-lang Lean development's
`patricia/tree.lean`, restated over our node encoding:

```
binary = hash(left, right)
edge   = hash(bottom, path) + ℓ        (ℓ ADDED, not hashed)
leaf   = value,   empty = 0
```

The two theorems this file *proves* are the spec side of the soundness argument:

* `valid_hashOf_ne_zero` — a canonical tree never hashes to `0`, so the empty encoding is
  unambiguous (P5's spec-side half);
* `hashOf_inj` — canonical trees of one height with equal hashes are equal: canonicity makes
  the map → root correspondence injective (§6.2 e). This is the analogue of cairo-lang's
  `hash_inj`.
-/


namespace PatriciaCircuit

/-- A non-empty Patricia subtree. The empty trie is not representable — it exists only as the
all-zero root of `Option Tree` (see `rootHashOf`). -/
inductive Tree where
  | leaf (value : Nat)
  | binary (left right : Tree)
  | edge (length path : Nat) (bottom : Tree)
deriving DecidableEq

namespace Tree

def isEdge : Tree → Prop
  | .edge _ _ _ => True
  | _ => False

/-- Canonical validity at a height: leaves are non-zero values at height 0; a binary's
children are both non-empty one level down; an edge compresses `1 ≤ ℓ` levels with aligned
compressed bits and a maximally merged (non-edge) bottom. -/
def valid : Tree → Nat → Prop
  | .leaf value, h => h = 0 ∧ value ≠ 0
  | .binary left right, h => ∃ h', h = h' + 1 ∧ left.valid h' ∧ right.valid h'
  | .edge length path bottom, h =>
      1 ≤ length ∧ length ≤ h ∧ path < 2 ^ length ∧
      bottom.valid (h - length) ∧ ¬ bottom.isEdge

/-- The node hash, per the production convention. -/
def hashOf (c : Crypto) : Tree → Nat
  | .leaf value => value
  | .binary left right => c.hash (left.hashOf c) (right.hashOf c)
  | .edge length path bottom => c.hash (bottom.hashOf c) path + length

/-- The value at `key` in a tree of height `h` (`0` = absent). Bit `h − 1` of the key selects
the child at height `h`; an edge whose compressed bits disagree with the key's ends the walk.
Keys outside the keyspace `[0, 2^h)` read `0` — the tree denotes a map on the keyspace. -/
def at' : Tree → Nat → Nat → Nat
  | .leaf value, _, key => if key = 0 then value else 0
  | .binary left right, h, key =>
      if key / 2 ^ (h - 1) = 0 then left.at' (h - 1) (key % 2 ^ (h - 1))
      else if key / 2 ^ (h - 1) = 1 then right.at' (h - 1) (key % 2 ^ (h - 1))
      else 0
  | .edge length _path bottom, h, key =>
      if key / 2 ^ (h - length) = _path then bottom.at' (h - length) (key % 2 ^ (h - length))
      else 0

end Tree

/-- The root hash of a possibly-empty trie: the all-zero hash for the empty one. -/
def rootHashOf (c : Crypto) : Option Tree → Nat
  | none => 0
  | some t => t.hashOf c

/-- Validity of a possibly-empty trie. -/
def validOpt : Option Tree → Nat → Prop
  | none, _ => True
  | some t, h => t.valid h

/-- Lookup in a possibly-empty trie: every key is absent from the empty one. -/
def atOpt : Option Tree → Nat → Nat → Nat
  | none, _, _ => 0
  | some t, h, key => t.at' h key

/-- A canonical tree never hashes to the empty encoding `0`: leaves are non-zero by
canonicity, inner hashes by `hash_ne_small`. -/
theorem valid_hashOf_ne_zero (c : Crypto) :
    ∀ {t : Tree} {h : Nat}, t.valid h → h ≤ c.maxLength → t.hashOf c ≠ 0 := by
  intro t h hv hh
  cases t with
  | leaf value => exact hv.2
  | binary left right =>
      have := c.hash_ne_small (x := left.hashOf c) (y := right.hashOf c) (a := 0) (b := 0)
        (Nat.zero_le _) (Nat.zero_le _)
      simpa [Tree.hashOf] using this
  | edge length path bottom =>
      have hlen : length ≤ c.maxLength := Nat.le_trans hv.2.1 hh
      have := c.hash_ne_small (x := bottom.hashOf c) (y := path) (a := length) (b := 0)
        hlen (Nat.zero_le _)
      simpa [Tree.hashOf] using this

/-- **Canonicity makes hashing injective** (§6.2 e; cairo-lang's `hash_inj`): two canonical
trees of the same height with the same hash are the same tree. The proof is the hash
decomposition argument: `hash_shift_inj` separates the three node shapes (a binary's offset
is `0`, an edge's is `ℓ ≥ 1`, and heights separate leaves), then recurses. -/
theorem hashOf_inj (c : Crypto) :
    ∀ {t₁ : Tree} {t₂ : Tree} {h : Nat}, h ≤ c.maxLength →
      t₁.valid h → t₂.valid h → t₁.hashOf c = t₂.hashOf c → t₁ = t₂ := by
  intro t₁
  induction t₁ with
  | leaf v₁ =>
      intro t₂ h hh hv₁ hv₂ he
      -- a leaf forces `h = 0`; binaries and edges force `h ≥ 1`.
      cases t₂ with
      | leaf v₂ => simpa [Tree.hashOf] using he
      | binary l₂ r₂ =>
          obtain ⟨h', hh', _, _⟩ := hv₂
          exact absurd hv₁.1 (by omega)
      | edge ℓ₂ p₂ b₂ =>
          have h1 : 1 ≤ ℓ₂ := hv₂.1
          have h2 : ℓ₂ ≤ h := hv₂.2.1
          exact absurd hv₁.1 (by omega)
  | binary l₁ r₁ ihl ihr =>
      intro t₂ h hh hv₁ hv₂ he
      cases t₂ with
      | leaf v₂ =>
          obtain ⟨h', hh', _⟩ := hv₁
          exact absurd hv₂.1 (by omega)
      | binary l₂ r₂ =>
          obtain ⟨h₁', hh₁, hvl₁, hvr₁⟩ := hv₁
          obtain ⟨h₂', hh₂, hvl₂, hvr₂⟩ := hv₂
          have hsame : h₁' = h₂' := by omega
          subst hsame
          have hh' : h₁' ≤ c.maxLength := by omega
          have := c.hash_inj (x := l₁.hashOf c) (y := r₁.hashOf c)
            (z := l₂.hashOf c) (w := r₂.hashOf c) (by simpa [Tree.hashOf] using he)
          rw [ihl hh' hvl₁ hvl₂ this.1, ihr hh' hvr₁ hvr₂ this.2]
      | edge ℓ₂ p₂ b₂ =>
          -- a binary's offset is 0, an edge's is ℓ₂ ≥ 1: `hash_shift_inj` separates them.
          have hℓ : ℓ₂ ≤ c.maxLength := Nat.le_trans hv₂.2.1 hh
          have := c.hash_shift_inj (a := 0) (b := ℓ₂) (Nat.zero_le _) hℓ
            (by simpa [Tree.hashOf] using he)
          exact absurd this.2.2.symm (by have := hv₂.1; omega)
  | edge ℓ₁ p₁ b₁ ihb =>
      intro t₂ h hh hv₁ hv₂ he
      cases t₂ with
      | leaf v₂ =>
          have h1 : 1 ≤ ℓ₁ := hv₁.1
          have h2 : ℓ₁ ≤ h := hv₁.2.1
          exact absurd hv₂.1 (by omega)
      | binary l₂ r₂ =>
          obtain ⟨h', hh', _⟩ := hv₂
          have hℓ : ℓ₁ ≤ c.maxLength := Nat.le_trans hv₁.2.1 hh
          have := c.hash_shift_inj (a := ℓ₁) (b := 0) hℓ (Nat.zero_le _)
            (by simpa [Tree.hashOf] using he)
          exact absurd this.2.2 (by have := hv₁.1; omega)
      | edge ℓ₂ p₂ b₂ =>
          have hℓ₁ : ℓ₁ ≤ c.maxLength := Nat.le_trans hv₁.2.1 hh
          have hℓ₂ : ℓ₂ ≤ c.maxLength := Nat.le_trans hv₂.2.1 hh
          have hs := c.hash_shift_inj (a := ℓ₁) (b := ℓ₂) hℓ₁ hℓ₂
            (by simpa [Tree.hashOf] using he)
          obtain ⟨hb, hp, hℓ⟩ := hs
          subst hp; subst hℓ
          have hh' : h - ℓ₁ ≤ c.maxLength := by omega
          rw [ihb hh' hv₁.2.2.2.1 hv₂.2.2.2.1 hb]

end PatriciaCircuit
