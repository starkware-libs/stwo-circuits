/-!
# The hash assumptions (A1 of `docs/patricia-circuit.md` §6.1)

Mirrors the `Crypto` structure of the cairo-lang Lean development
(`src/starkware/cairo/common/lean/patricia/crypto.lean`): the hash is a parameter, and
collision resistance is modeled as injectivity — including under bounded additive offsets,
which is exactly the *shifted-collision* assumption the additive edge length needs.

Two additions over cairo-lang's structure, both structural facts of our instantiation
(`hash2 = blake2s truncated to 251 bits`), not hardness assumptions:

* `hash_lt`: outputs are below `2^251`, so `hash + ℓ` never wraps the `2^256` word arithmetic
  of the circuit model;
* the offset bound `maxLength` is 251 in the intended instantiation.
-/

namespace PatriciaCircuit

/-- The word-arithmetic modulus of the circuit model: unit fields are eight `u32` words. -/
def wordModulus : Nat := 2 ^ 256

structure Crypto where
  hash : Nat → Nat → Nat
  /-- The largest additive offset an edge can apply — 251 in the intended instantiation. -/
  maxLength : Nat
  /-- A1, shifted variant: equal offset-adjusted hashes imply equal preimages and offsets.
  With `a = b = 0` this is plain collision resistance (as injectivity). -/
  hash_shift_inj :
    ∀ {x y a z w b : Nat}, a ≤ maxLength → b ≤ maxLength →
      hash x y + a = hash z w + b → x = z ∧ y = w ∧ a = b
  /-- An offset-adjusted hash is never a small value — in particular never `0`, the empty
  hash, so live inner nodes are separated from the empty encoding. -/
  hash_ne_small :
    ∀ {x y a b : Nat}, a ≤ maxLength → b ≤ maxLength → hash x y + a ≠ b
  /-- Outputs are truncated below `2^251` (a structural fact of `hash2`, not an assumption
  of hardness). -/
  hash_lt : ∀ x y, hash x y < 2 ^ 251
  /-- The offset bound is small (251 in the intended instantiation). Load-bearing beyond
  fitting the truncation: trie heights are bounded by `maxLength`, and the top-down pinning
  argument needs positions along a walk to stay below the `2^256` word modulus — which holds
  exactly because heights (hence path bit-lengths) are at most 255. -/
  maxLength_le : maxLength ≤ 255

namespace Crypto

variable (c : Crypto)

/-- Plain injectivity, the `a = b = 0` case of `hash_shift_inj`. -/
theorem hash_inj {x y z w : Nat} (h : c.hash x y = c.hash z w) : x = z ∧ y = w := by
  have := c.hash_shift_inj (a := 0) (b := 0) (Nat.zero_le _) (Nat.zero_le _)
    (by simpa using h)
  exact ⟨this.1, this.2.1⟩

/-- A hash is never zero (`a = b = 0` in `hash_ne_small`). -/
theorem hash_ne_zero (x y : Nat) : c.hash x y ≠ 0 := by
  have := c.hash_ne_small (x := x) (y := y) (a := 0) (b := 0)
    (Nat.zero_le _) (Nat.zero_le _)
  simpa using this

/-- An offset-adjusted hash stays below the word modulus, so the circuit's mod-`2^256`
arithmetic is exact on it. -/
theorem hash_add_lt_wordModulus {x y a : Nat} (ha : a ≤ c.maxLength) :
    c.hash x y + a < wordModulus := by
  have h1 := c.hash_lt x y
  have h2 := c.maxLength_le
  have h3 : (2 : Nat) ^ 252 ≤ 2 ^ 256 := Nat.pow_le_pow_right (by omega) (by omega)
  have h4 : (2 : Nat) ^ 252 = 2 ^ 251 + 2 ^ 251 := by
    have := Nat.pow_succ 2 251
    omega
  have h5 : (2 : Nat) ^ 8 ≤ 2 ^ 251 := Nat.pow_le_pow_right (by omega) (by omega)
  have h6 : (2 : Nat) ^ 8 = 256 := by decide
  unfold wordModulus
  omega

end Crypto

end PatriciaCircuit
