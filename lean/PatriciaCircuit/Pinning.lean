import PatriciaCircuit.Node
import PatriciaCircuit.Model

/-!
# Top-down pinning (`docs/patricia-circuit.md` §5.1–§5.2, §6.2 a–b)

`Pinned`: a unit carries the height, position, and hash of a real node of the committed
tree. The main theorem `pinned_of_consumed` is the derivation argument's core: in a
satisfying fold every consumed unit with a non-zero hash is pinned — by descending
induction from the root entry, with `Crypto.hash_shift_inj` decomposing each hash step
and the slot relations inverting each position step exactly (the mod-`2^256` arithmetic
vanishes because pinned positions stay below `2^255`).
-/

namespace PatriciaCircuit

/-- Heights in a finite unit list are bounded. -/
theorem exists_height_bound (l : List SUnit) : ∃ B, ∀ u ∈ l, u.height ≤ B := by
  induction l with
  | nil => exact ⟨0, by simp⟩
  | cons a t ih =>
      obtain ⟨B, hB⟩ := ih
      refine ⟨max a.height B, ?_⟩
      intro u hu
      rcases List.mem_cons.mp hu with h | h
      · subst h; exact Nat.le_max_left _ _
      · exact Nat.le_trans (hB u h) (Nat.le_max_right _ _)

theorem lt_wordModulus_of_lt_pow {x a : Nat} (hx : x < 2 ^ a) (ha : a ≤ 256) :
    x < wordModulus :=
  Nat.lt_of_lt_of_le hx (Nat.pow_le_pow_right (by omega) ha)

/-- A unit is pinned to the committed tree: a real node sits at its position with its hash. -/
def Pinned (c : Crypto) (t : Tree) (H : Nat) (u : SUnit) : Prop :=
  u.height ≤ H ∧
    ∃ st : Tree, t.nodeAtDepth (H - u.height) u.path = some st ∧ st.hashOf c = u.hash

/-! ### Membership plumbing -/

theorem leaf_entry_mem_produced (c : Crypto) {f : Fold} {s : LeafSlot}
    (hs : s ∈ f.leaves) : s.entry ∈ f.produced c := by
  unfold Fold.produced
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl
    (List.mem_append.mpr (Or.inl (List.mem_map.mpr ⟨s, hs, rfl⟩))))))

theorem binary_out_mem_produced (c : Crypto) {f : Fold} {s : BinarySlot}
    (hs : s ∈ f.binaries) : s.out c ∈ f.produced c := by
  unfold Fold.produced
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl
    (List.mem_append.mpr (Or.inr (List.mem_map.mpr ⟨s, hs, rfl⟩))))))

theorem edge_out_mem_produced (c : Crypto) {f : Fold} {s : EdgeSlot}
    (hs : s ∈ f.edges) : s.out c ∈ f.produced c := by
  unfold Fold.produced
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr
    (List.mem_map.mpr ⟨s, hs, rfl⟩))))

theorem sibling_mem_produced (c : Crypto) {f : Fold} {u : SUnit}
    (hu : u ∈ f.siblings) : u ∈ f.produced c := by
  unfold Fold.produced
  exact List.mem_append.mpr (Or.inr hu)

theorem rootEntry_mem_consumed (f : Fold) : f.rootEntry ∈ f.consumed := by
  unfold Fold.consumed
  exact List.mem_append.mpr (Or.inr (by simp))

/-! ### Leaf entry field lemmas -/

theorem LeafSlot.entry_height {s : LeafSlot} (hval : s.value ≠ 0) : s.entry.height = 0 := by
  simp [LeafSlot.entry, hval]

theorem LeafSlot.entry_path {s : LeafSlot} (hval : s.value ≠ 0) : s.entry.path = s.key := by
  simp [LeafSlot.entry, hval]

theorem LeafSlot.entry_hash {s : LeafSlot} (hval : s.value ≠ 0) : s.entry.hash = s.value := by
  simp [LeafSlot.entry, hval]

/-! ### One descent step, per slot shape -/

/-- A pinned live binary out opens a real binary node: exact child positions, pinned children. -/
theorem pinned_binary (c : Crypto) {t : Tree} {H : Nat} (hH : H ≤ c.maxLength)
    (hv : t.valid H) {s : BinarySlot} (hok : s.ok) (hlive : s.isLive)
    (hp : Pinned c t H (s.out c)) :
    s.left.height + 1 ≤ H ∧
    s.outPath < 2 ^ (H - (s.left.height + 1)) ∧
    s.left.path = 2 * s.outPath ∧
    s.right.path = 2 * s.outPath + 1 ∧
    Pinned c t H s.left ∧ Pinned c t H s.right := by
  have hbit : s.liveBit = 1 := by simp [BinarySlot.liveBit, hlive]
  have hout_h : (s.out c).height = s.left.height + 1 := by
    simp [BinarySlot.out, hbit]
  have hout_p : (s.out c).path = s.outPath := rfl
  have hout_hash : (s.out c).hash = c.hash s.left.hash s.right.hash := by
    simp [BinarySlot.out, hlive]
  obtain ⟨hhle, st, hnode, hhash⟩ := hp
  rw [hout_h] at hhle
  rw [hout_h, hout_p] at hnode
  rw [hout_hash] at hhash
  have hqlt : s.outPath < 2 ^ (H - (s.left.height + 1)) := Tree.nodeAtDepth_path_lt hv hnode
  obtain ⟨-, hstv⟩ := Tree.nodeAtDepth_valid hv hnode
  rw [(by omega : H - (H - (s.left.height + 1)) = s.left.height + 1)] at hstv
  cases st with
  | leaf v => exact absurd hstv.1 (by omega)
  | edge ℓ' p' b' =>
      have hℓ'max : ℓ' ≤ c.maxLength := by
        have h1 : 1 ≤ ℓ' := hstv.1
        have h2 : ℓ' ≤ s.left.height + 1 := hstv.2.1
        omega
      have := c.hash_shift_inj (a := ℓ') (b := 0) hℓ'max (Nat.zero_le _)
        (by rw [Nat.add_zero]; exact hhash)
      exact absurd this.2.2 (by have := hstv.1; omega)
  | binary L R =>
      have hinj := c.hash_inj (show c.hash (L.hashOf c) (R.hashOf c)
        = c.hash s.left.hash s.right.hash from hhash)
      have hnodeL := Tree.nodeAtDepth_binary_left hnode
      have hnodeR := Tree.nodeAtDepth_binary_right hnode
      have hdepth : H - (s.left.height + 1) + 1 = H - s.left.height := by omega
      have hd255 : H - s.left.height ≤ 256 := by
        have := c.maxLength_le
        omega
      have hplt : 2 * s.outPath + 1 < 2 ^ (H - s.left.height) := by
        rw [← hdepth, Nat.pow_succ]
        omega
      have hlp : s.left.path = 2 * s.outPath := by
        rw [hok.2.1, Nat.mod_eq_of_lt (lt_wordModulus_of_lt_pow (by omega) hd255)]
      have hrp : s.right.path = 2 * s.outPath + 1 := by
        rw [hok.2.2.1, hbit, Nat.mod_eq_of_lt (lt_wordModulus_of_lt_pow hplt hd255)]
      have hrh : s.right.height = s.left.height := hok.1.symm
      refine ⟨hhle, hqlt, hlp, hrp, ⟨by omega, L, ?_, hinj.1⟩, ⟨by omega, R, ?_, hinj.2⟩⟩
      · rw [hlp, (by omega : H - s.left.height = H - (s.left.height + 1) + 1)]
        exact hnodeL
      · rw [hrp, hrh, (by omega : H - s.left.height = H - (s.left.height + 1) + 1)]
        exact hnodeR

/-- A pinned live edge out opens the real edge node — same length and compressed bits
(`hash_shift_inj`, the shifted variant) — with exact bottom position and pinned bottom. -/
theorem pinned_edge (c : Crypto) {t : Tree} {H : Nat} (hH : H ≤ c.maxLength)
    (hv : t.valid H) {s : EdgeSlot} (hok : s.ok) (hlive : s.isLive)
    (hp : Pinned c t H (s.out c)) :
    ∃ b' : Tree,
      s.bottom.height + s.length ≤ H ∧
      s.outPath < 2 ^ (H - (s.bottom.height + s.length)) ∧
      t.nodeAtDepth (H - (s.bottom.height + s.length)) s.outPath
        = some (.edge s.length s.edgePath b') ∧
      b'.hashOf c = s.bottom.hash ∧
      s.bottom.path = s.outPath * 2 ^ s.length + s.edgePath ∧
      Pinned c t H s.bottom := by
  have hlen1 : 1 ≤ s.length := hok.2.1 hlive
  have hout_h : (s.out c).height = s.bottom.height + s.length := rfl
  have hout_p : (s.out c).path = s.outPath := rfl
  have hout_hash : (s.out c).hash = c.hash s.bottom.hash s.edgePath + s.length := by
    simp [EdgeSlot.out, hlive]
  obtain ⟨hhle, st, hnode, hhash⟩ := hp
  rw [hout_h] at hhle
  rw [hout_h, hout_p] at hnode
  rw [hout_hash] at hhash
  have hℓmax : s.length ≤ c.maxLength := by omega
  have hqlt : s.outPath < 2 ^ (H - (s.bottom.height + s.length)) :=
    Tree.nodeAtDepth_path_lt hv hnode
  obtain ⟨-, hstv⟩ := Tree.nodeAtDepth_valid hv hnode
  rw [(by omega : H - (H - (s.bottom.height + s.length)) = s.bottom.height + s.length)]
    at hstv
  cases st with
  | leaf v => exact absurd hstv.1 (by omega)
  | binary L R =>
      have := c.hash_shift_inj (a := 0) (b := s.length) (Nat.zero_le _) hℓmax
        (by rw [Nat.add_zero]; exact hhash)
      exact absurd this.2.2.symm (by omega)
  | edge ℓ' p' b' =>
      obtain ⟨hℓ'1, hℓ'le, hp'lt, hb'v, _⟩ := hstv
      have hℓ'max : ℓ' ≤ c.maxLength := by omega
      obtain ⟨hbh, hbp, hbl⟩ := c.hash_shift_inj (a := ℓ') (b := s.length) hℓ'max hℓmax hhash
      subst hbp
      subst hbl
      have hnodeB := Tree.nodeAtDepth_edge_bottom hℓ'1 hok.2.2.1 hnode
      have hdepth : H - (s.bottom.height + s.length) + s.length = H - s.bottom.height := by
        omega
      have hd255 : H - s.bottom.height ≤ 256 := by
        have := c.maxLength_le
        omega
      have hblock : s.outPath * 2 ^ s.length + s.edgePath < 2 ^ (H - s.bottom.height) := by
        rw [← hdepth]
        exact two_block_lt hqlt hok.2.2.1
      have hbpath : s.bottom.path = s.outPath * 2 ^ s.length + s.edgePath := by
        rw [hok.2.2.2.2.2.2, Nat.mod_eq_of_lt (lt_wordModulus_of_lt_pow hblock hd255)]
      refine ⟨b', hhle, hqlt, hnode, hbh, hbpath, by omega, b', ?_, hbh⟩
      rw [hbpath,
        (by omega : H - s.bottom.height = H - (s.bottom.height + s.length) + s.length)]
      exact hnodeB

/-- The root entry is pinned to the committed root. -/
theorem pinned_rootEntry (c : Crypto) {t : Tree} (f : Fold)
    (hroot : t.hashOf c = f.root) (hne : f.root ≠ 0) : Pinned c t f.height f.rootEntry := by
  have hh : f.rootEntry.height = f.height := by simp [Fold.rootEntry, hne]
  refine ⟨Nat.le_of_eq hh, t, ?_, ?_⟩
  · rw [hh, Nat.sub_self]
    exact Tree.nodeAtDepth_zero t
  · exact hroot

/-- **Pinning** (§6.2 a): in a satisfying fold over the committed tree, every consumed unit
with a non-zero hash carries the position and hash of a real node — by descending induction
from the root entry, one hash-decomposition step per consuming slot. -/
theorem pinned_of_consumed (c : Crypto) (f : Fold) {t : Tree}
    (hH : f.height ≤ c.maxLength) (hsat : f.sat c) (hv : t.valid f.height)
    (hroot : t.hashOf c = f.root) :
    ∀ u ∈ f.consumed, u.hash ≠ 0 → Pinned c t f.height u := by
  obtain ⟨hbin, hedge, _hsib, _hrootOk, hperm⟩ := hsat
  obtain ⟨B, hB⟩ := exists_height_bound f.consumed
  suffices key : ∀ n, ∀ u ∈ f.consumed, u.hash ≠ 0 → B < u.height + n →
      Pinned c t f.height u by
    intro u hu hne
    exact key (B + 1) u hu hne (by have := hB u hu; omega)
  intro n
  induction n with
  | zero =>
      intro u hu hne hlt
      have := hB u hu
      omega
  | succ n ih =>
      intro u hu hne hlt
      unfold Fold.consumed at hu
      rcases List.mem_append.mp hu with hu' | hroot_mem
      · rcases List.mem_append.mp hu' with hchild | hbot
        · -- a binary slot's child: pin its out one level up, then descend.
          obtain ⟨s, hs, hmem⟩ := List.mem_flatMap.mp hchild
          simp only [List.mem_cons, List.not_mem_nil, or_false] at hmem
          have hok := hbin s hs
          have hui : u ≠ SUnit.inert := fun h => hne (by rw [h]; rfl)
          have hslive : s.isLive := by
            intro ⟨hl, hr⟩
            rcases hmem with h | h
            · exact hui (h ▸ hl)
            · exact hui (h ▸ hr)
          have houtc : s.out c ∈ f.consumed :=
            hperm.mem_iff.mp (binary_out_mem_produced c hs)
          have houth : (s.out c).hash ≠ 0 := by
            have : (s.out c).hash = c.hash s.left.hash s.right.hash := by
              simp [BinarySlot.out, hslive]
            rw [this]
            exact c.hash_ne_zero _ _
          have hout_h : (s.out c).height = s.left.height + 1 := by
            simp [BinarySlot.out, BinarySlot.liveBit, hslive]
          have huh : u.height = s.left.height := by
            rcases hmem with h | h
            · rw [h]
            · rw [h, hok.1.symm]
          have hpin_out : Pinned c t f.height (s.out c) :=
            ih (s.out c) houtc houth (by omega)
          obtain ⟨-, -, -, -, hpl, hpr⟩ := pinned_binary c hH hv hok hslive hpin_out
          rcases hmem with h | h
          · rw [h]; exact hpl
          · rw [h]; exact hpr
        · -- an edge slot's bottom: pin its out `ℓ ≥ 1` levels up, then descend.
          obtain ⟨s, hs, hbot_eq⟩ := List.mem_map.mp hbot
          have hok := hedge s hs
          have hui : u ≠ SUnit.inert := fun h => hne (by rw [h]; rfl)
          have hslive : s.isLive := by
            intro ⟨hb, _, _⟩
            exact hui (hbot_eq.symm.trans hb)
          have hlen1 : 1 ≤ s.length := hok.2.1 hslive
          have houtc : s.out c ∈ f.consumed :=
            hperm.mem_iff.mp (edge_out_mem_produced c hs)
          have houth : (s.out c).hash ≠ 0 := by
            have : (s.out c).hash = c.hash s.bottom.hash s.edgePath + s.length := by
              simp [EdgeSlot.out, hslive]
            omega
          have hout_h : (s.out c).height = s.bottom.height + s.length := rfl
          have huh : u.height = s.bottom.height := by rw [← hbot_eq]
          have hpin_out : Pinned c t f.height (s.out c) :=
            ih (s.out c) houtc houth (by omega)
          obtain ⟨b', -, -, -, -, -, hpb⟩ := pinned_edge c hH hv hok hslive hpin_out
          rw [← hbot_eq]
          exact hpb
      · -- the root entry: pinned to the committed root itself.
        have hu_eq : u = f.rootEntry := by simpa using hroot_mem
        have hrne : f.root ≠ 0 := by
          rw [hu_eq] at hne
          exact hne
        rw [hu_eq]
        exact pinned_rootEntry c f hroot hrne

/-- **Present claims read the tree** (§6.2 b): a present leaf slot's `(key, value)` is a real
leaf of the committed tree, and the key is inside the keyspace. -/
theorem present_reads (c : Crypto) (f : Fold) {t : Tree}
    (hH : f.height ≤ c.maxLength) (hsat : f.sat c) (hv : t.valid f.height)
    (hroot : t.hashOf c = f.root) {s : LeafSlot} (hs : s ∈ f.leaves) (hval : s.value ≠ 0) :
    s.key < 2 ^ f.height ∧ t.nodeAtDepth f.height s.key = some (.leaf s.value) ∧
      t.at' f.height s.key = s.value := by
  have hperm := hsat.2.2.2.2
  have hcons : s.entry ∈ f.consumed := hperm.mem_iff.mp (leaf_entry_mem_produced c hs)
  have hpin := pinned_of_consumed c f hH hsat hv hroot s.entry hcons
    (by rw [s.entry_hash hval]; exact hval)
  obtain ⟨-, st, hnode, hhash⟩ := hpin
  rw [s.entry_height hval, s.entry_path hval, Nat.sub_zero] at hnode
  rw [s.entry_hash hval] at hhash
  obtain ⟨-, hstv⟩ := Tree.nodeAtDepth_valid hv hnode
  rw [Nat.sub_self] at hstv
  cases st with
  | binary L R =>
      obtain ⟨h', heq, -, -⟩ := hstv
      exact absurd heq (by omega)
  | edge ℓ' p' b' =>
      have h1 : 1 ≤ ℓ' := hstv.1
      have h2 : ℓ' ≤ 0 := hstv.2.1
      exact absurd h1 (by omega)
  | leaf v =>
      have hveq : v = s.value := hhash
      subst hveq
      exact ⟨Tree.nodeAtDepth_path_lt hv hnode, hnode, Tree.at'_of_leaf_node hv hnode⟩

end PatriciaCircuit
