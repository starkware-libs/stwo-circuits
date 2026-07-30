import PatriciaCircuit.Pinning

/-!
# Reconstructing the committed tree (`docs/patricia-circuit.md` §6.2 e)

A satisfying fold's consumed units decode into canonical trees, by strong induction on
height: each unit's producer is a leaf entry (a leaf), a live binary or edge out (glue the
recursively decoded children — the slot's canonicity rules make the glue canonical), or a
sibling (glue the subtree the caller pins behind its hash — for the update circuit, the
prev tree's subtree there). Applied to the root entry this constructs `T_new`.
-/

namespace PatriciaCircuit

/-- Producer analysis: a produced unit with a non-zero hash is a present leaf entry, a live
binary out, a live edge out, or a sibling. -/
theorem produced_cases (c : Crypto) (f : Fold) {u : SUnit}
    (hu : u ∈ f.produced c) (hne : u.hash ≠ 0) :
    (∃ s ∈ f.leaves, s.value ≠ 0 ∧ u = ⟨0, s.key, tagLeaf, s.value⟩) ∨
    (∃ s ∈ f.binaries, s.isLive ∧ u = s.out c) ∨
    (∃ s ∈ f.edges, s.isLive ∧ u = s.out c) ∨
    u ∈ f.siblings := by
  unfold Fold.produced at hu
  rcases List.mem_append.mp hu with hu1 | hD
  · rcases List.mem_append.mp hu1 with hu2 | hC
    · rcases List.mem_append.mp hu2 with hA | hB
      · obtain ⟨s, hs, hentry⟩ := List.mem_map.mp hA
        by_cases hval : s.value = 0
        · exact absurd (by rw [← hentry]; simp [LeafSlot.entry, hval, SUnit.inert]) hne
        · exact Or.inl ⟨s, hs, hval, by rw [← hentry]; simp [LeafSlot.entry, hval]⟩
      · obtain ⟨s, hs, hout⟩ := List.mem_map.mp hB
        by_cases hlive : s.isLive
        · exact Or.inr (Or.inl ⟨s, hs, hlive, hout.symm⟩)
        · exact absurd (by rw [← hout]; simp [BinarySlot.out, hlive]) hne
    · obtain ⟨s, hs, hout⟩ := List.mem_map.mp hC
      by_cases hlive : s.isLive
      · exact Or.inr (Or.inr (Or.inl ⟨s, hs, hlive, hout.symm⟩))
      · exact absurd (by rw [← hout]; simp [EdgeSlot.out, hlive]) hne
  · exact Or.inr (Or.inr (Or.inr hD))

/-- **Decoding** (§6.2 e): every consumed unit with a non-zero hash opens to a canonical
tree of its height hashing to its hash; the tree is an edge only for edge-out or sibling
units. Siblings are glued through `hsib` (the caller pins them, e.g. to prev subtrees). -/
theorem reconstruct_consumed (c : Crypto) (f : Fold) (hsat : f.sat c)
    (hsib : ∀ w ∈ f.siblings, w.hash ≠ 0 →
      ∃ st : Tree, st.valid w.height ∧ st.hashOf c = w.hash) :
    ∀ w ∈ f.consumed, w.hash ≠ 0 →
      ∃ st : Tree, st.valid w.height ∧ st.hashOf c = w.hash ∧
        (st.isEdge → w.kind = tagEdge ∨ w.kind = tagOpaque) := by
  have hperm := hsat.2.2.2.2
  suffices key : ∀ n, ∀ w ∈ f.consumed, w.hash ≠ 0 → w.height < n →
      ∃ st : Tree, st.valid w.height ∧ st.hashOf c = w.hash ∧
        (st.isEdge → w.kind = tagEdge ∨ w.kind = tagOpaque) by
    intro w hw hne
    exact key (w.height + 1) w hw hne (Nat.lt_succ_self _)
  intro n
  induction n with
  | zero =>
      intro w hw hne hlt
      omega
  | succ n ih =>
      intro w hw hne hlt
      have hwp : w ∈ f.produced c := hperm.mem_iff.mpr hw
      rcases produced_cases c f hwp hne with
        ⟨s, hs, hval, hEq⟩ | ⟨s, hs, hlive, hEq⟩ | ⟨s, hs, hlive, hEq⟩ | hsibmem
      · -- a present leaf entry decodes to a leaf.
        subst hEq
        exact ⟨.leaf s.value, ⟨rfl, hval⟩, rfl, by simp [Tree.isEdge]⟩
      · -- a live binary out glues the decoded children.
        have hok := hsat.1 s hs
        obtain ⟨-, -, hlhash, hrhash⟩ := hok.2.2.2 hlive
        have hlc : s.left ∈ f.consumed := by
          unfold Fold.consumed
          exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl
            (List.mem_flatMap.mpr ⟨s, hs, by simp⟩))))
        have hrc : s.right ∈ f.consumed := by
          unfold Fold.consumed
          exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl
            (List.mem_flatMap.mpr ⟨s, hs, by simp⟩))))
        have hwh : w.height = s.left.height + 1 := by
          rw [hEq]
          simp [BinarySlot.out, BinarySlot.liveBit, hlive]
        have hrh : s.right.height = s.left.height := hok.1.symm
        obtain ⟨stL, hvL, hhL, -⟩ := ih s.left hlc hlhash (by omega)
        obtain ⟨stR, hvR, hhR, -⟩ := ih s.right hrc hrhash (by omega)
        rw [hrh] at hvR
        refine ⟨.binary stL stR, ⟨s.left.height, hwh, hvL, hvR⟩, ?_, by simp [Tree.isEdge]⟩
        show c.hash (stL.hashOf c) (stR.hashOf c) = w.hash
        rw [hhL, hhR, hEq]
        simp [BinarySlot.out, hlive]
      · -- a live edge out glues the decoded bottom; canonicity rules make it an edge node.
        have hok := hsat.2.1 s hs
        have hlen1 : 1 ≤ s.length := hok.2.1 hlive
        have hbne : s.bottom.hash ≠ 0 := hok.2.2.2.2.2.1 hlive
        have hbc : s.bottom ∈ f.consumed := by
          unfold Fold.consumed
          exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr
            (List.mem_map.mpr ⟨s, hs, rfl⟩))))
        have hwh : w.height = s.bottom.height + s.length := by rw [hEq]; rfl
        obtain ⟨stB, hvB, hhB, hinvB⟩ := ih s.bottom hbc hbne (by omega)
        have hnotE : ¬ stB.isEdge := by
          intro hE
          rcases hinvB hE with h3 | h4
          · exact hok.2.2.2.1 hlive h3
          · have h0 : s.bottom.height = 0 := hok.2.2.2.2.1 h4
            rw [h0] at hvB
            cases stB with
            | leaf v => exact hE
            | binary a b => exact hE
            | edge l p b =>
                obtain ⟨h1, h2, -, -, -⟩ := hvB
                omega
        refine ⟨.edge s.length s.edgePath stB,
          ⟨hlen1, by omega, hok.2.2.1, ?_, hnotE⟩, ?_, fun _ => Or.inl ?_⟩
        · rw [hwh, (by omega : s.bottom.height + s.length - s.length = s.bottom.height)]
          exact hvB
        · show c.hash (stB.hashOf c) s.edgePath + s.length = w.hash
          rw [hhB, hEq]
          simp [EdgeSlot.out, hlive]
        · rw [hEq]
          simp [EdgeSlot.out, EdgeSlot.liveBit, hlive, tagEdge]
      · -- a sibling is glued through the caller's pin.
        obtain ⟨st, hv', hh'⟩ := hsib w hsibmem hne
        have hkind : w.kind = tagOpaque := by
          rcases hsat.2.2.1 w hsibmem with hin | ⟨hk, -⟩
          · exact absurd (by rw [hin]; rfl) hne
          · exact hk
        exact ⟨st, hv', hh', fun _ => Or.inr hkind⟩

/-- The decoded root: a non-empty committed root opens to a canonical tree of the fold's
height hashing to it — the existence half of `T_new`. -/
theorem root_tree_exists (c : Crypto) (f : Fold) (hsat : f.sat c) (hne : f.root ≠ 0)
    (hsib : ∀ w ∈ f.siblings, w.hash ≠ 0 →
      ∃ st : Tree, st.valid w.height ∧ st.hashOf c = w.hash) :
    ∃ st : Tree, st.valid f.height ∧ st.hashOf c = f.root := by
  obtain ⟨st, hv', hh', -⟩ := reconstruct_consumed c f hsat hsib f.rootEntry
    (rootEntry_mem_consumed f) hne
  have hht : f.rootEntry.height = f.height := by simp [Fold.rootEntry, hne]
  rw [hht] at hv'
  exact ⟨st, hv', hh'⟩

end PatriciaCircuit
