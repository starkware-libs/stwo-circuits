import PatriciaCircuit.Spec
import PatriciaCircuit.Model
import PatriciaCircuit.Node
import PatriciaCircuit.Pinning
import PatriciaCircuit.Counting
import PatriciaCircuit.Reconstruct

/-!
# The soundness theorems (`docs/patricia-circuit.md` §1, §6)

The two main theorems, stated as the design document states them — conditional on the
caller's canonical prev tree (A2), existential in the new tree — over the `Crypto`
assumptions (A1, including the shifted variant) and the layer-1 circuit model.

Both are fully proven. The proof stack: `Node.lean` (positions and the `at'` calculus),
`Pinning.lean` (§5.1–5.2: every consumed live unit is a real node), `Counting.lean`
(§5.3–5.4 as one `countP` identity per key), `Reconstruct.lean` (§6.2 e: decoding `T_new`).
This file assembles them, plus the `climb`/emptiness lemmas for the zero-root cases.

`update_sound` carries one interface precondition beyond the design document's statement:
live rows have pairwise-distinct keys (see its docstring for the finding that forced it).
-/


namespace PatriciaCircuit

/-- **The climb lemma** — the formal core of "no cycles, one exit" (doc §5.2): over the empty
root, every live produced unit is consumed by a live slot whose output is a live produced unit
strictly higher up. Live outputs carry a non-zero derived kind, and the inert root entry
consumes nothing live. -/
theorem climb (c : Crypto) (f : Fold) (hsat : f.sat c) (hempty : f.root = 0)
    {u : SUnit} (hu : u ∈ f.produced c) (hlive : u ≠ SUnit.inert) :
    ∃ v ∈ f.produced c, v ≠ SUnit.inert ∧ u.height < v.height := by
  obtain ⟨hbin, hedge, _hsib, hroot, hperm⟩ := hsat
  have hu' : u ∈ f.consumed := hperm.mem_iff.mp hu
  unfold Fold.consumed at hu'
  rcases List.mem_append.mp hu' with hu' | hru
  rcases List.mem_append.mp hu' with hchild | hbot
  · -- consumed by a binary slot: its live output sits one level up.
    obtain ⟨s, hs, hmem⟩ := List.mem_flatMap.mp hchild
    have hok := hbin s hs
    simp only [List.mem_cons, List.not_mem_nil, or_false] at hmem
    have hslive : s.isLive := by
      intro ⟨hl, hr⟩
      rcases hmem with h | h
      · exact hlive (h ▸ hl)
      · exact hlive (h ▸ hr)
    have hout : BinarySlot.out c s ∈ f.produced c := by
      unfold Fold.produced
      exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl
        (List.mem_append.mpr (Or.inr (List.mem_map.mpr ⟨s, hs, rfl⟩))))))
    refine ⟨BinarySlot.out c s, hout, ?_, ?_⟩
    · intro h
      have hkind : (BinarySlot.out c s).kind = 0 := by rw [h]; rfl
      simp [BinarySlot.out, BinarySlot.liveBit, if_pos hslive, tagBinary] at hkind
    · have hbit : s.liveBit = 1 := by simp [BinarySlot.liveBit, if_pos hslive]
      have hheights : s.left.height = s.right.height := hok.1
      have huh : u.height = s.left.height := by
        rcases hmem with h | h
        · rw [h]
        · rw [h, hheights]
      simp only [BinarySlot.out, hbit]
      omega
  · -- consumed by an edge slot: its live output sits ℓ ≥ 1 levels up.
    obtain ⟨s, hs, hbot⟩ := List.mem_map.mp hbot
    have hok := hedge s hs
    have hslive : s.isLive := by
      intro ⟨hb, _, _⟩
      exact hlive (hbot ▸ hb)
    have hlen : 1 ≤ s.length := hok.2.1 hslive
    have hout : EdgeSlot.out c s ∈ f.produced c := by
      unfold Fold.produced
      exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr
        (List.mem_map.mpr ⟨s, hs, rfl⟩))))
    refine ⟨EdgeSlot.out c s, hout, ?_, ?_⟩
    · intro h
      have hkind : (EdgeSlot.out c s).kind = 0 := by rw [h]; rfl
      simp [EdgeSlot.out, EdgeSlot.liveBit, if_pos hslive, tagEdge] at hkind
    · have huh : u.height = s.bottom.height := by rw [← hbot]
      simp only [EdgeSlot.out]
      omega
  · -- the root entry over the empty root is the inert unit: it consumes nothing live.
    have hu0 : u = f.rootEntry := by simpa using hru
    have hkind : f.rootKind = tagPadding := hroot.1 hempty
    refine absurd ?_ hlive
    rw [hu0]
    simp [Fold.rootEntry, hempty, hkind, tagPadding, SUnit.inert]

/-- Over the empty root every produced unit is inert: a live one would climb forever in a
finite list. -/
theorem empty_root_all_inert (c : Crypto) (f : Fold) (hsat : f.sat c) (hempty : f.root = 0) :
    ∀ u ∈ f.produced c, u = SUnit.inert := by
  intro u₀ hu₀
  if hlive₀ : u₀ = SUnit.inert then
    exact hlive₀
  else
    exfalso
    have ascent : ∀ k : Nat, ∃ u ∈ f.produced c, u ≠ SUnit.inert ∧ u₀.height + k ≤ u.height := by
      intro k
      induction k with
      | zero => exact ⟨u₀, hu₀, hlive₀, by omega⟩
      | succ k ih =>
          obtain ⟨u, hu, hlive, hk⟩ := ih
          obtain ⟨v, hv, hvlive, hlt⟩ := climb c f hsat hempty hu hlive
          exact ⟨v, hv, hvlive, by omega⟩
    obtain ⟨B, hB⟩ := exists_height_bound (f.produced c)
    obtain ⟨u, hu, _, hge⟩ := ascent (B + 1)
    have := hB u hu
    omega

/-- **P5, proven** (doc §6.2, the emptiness claim): against the zero root the fold can claim
nothing present — a present leaf's entry would be a live produced unit. -/
theorem empty_root_all_absent (c : Crypto) (f : Fold)
    (hsat : f.sat c)
    (hempty : f.root = 0) :
    ∀ s ∈ f.leaves, ¬ s.isPresent := by
  intro s hs hpresent
  have hentry : s.entry ∈ f.produced c := by
    unfold Fold.produced
    exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl
      (List.mem_append.mpr (Or.inl (List.mem_map.mpr ⟨s, hs, rfl⟩))))))
  have hinert := empty_root_all_inert c f hsat hempty s.entry hentry
  have hkind : s.entry.kind = tagLeaf := by
    simp [LeafSlot.entry, if_neg hpresent]
  rw [hinert] at hkind
  simp [SUnit.inert, tagLeaf] at hkind

/-- **Skeleton soundness** (doc §6.2 a–b; the statement of `verify_patricia_skeleton`):
given the committed canonical trie (A2 — the caller binds `f.root` to its hash), a satisfying
fold proves every present-claiming leaf slot: the trie holds exactly `value` at `key`.

Proven by the top-down pinning induction (`pinned_of_consumed` + `present_reads`); the empty
root is `empty_root_all_absent`. -/
theorem fold_reads_the_tree (c : Crypto) (f : Fold) (t : Option Tree)
    (hheight : f.height ≤ c.maxLength)
    (hsat : f.sat c)
    (hvalid : validOpt t f.height)
    (hroot : rootHashOf c t = f.root) :
    ∀ s ∈ f.leaves, s.isPresent → atOpt t f.height s.key = s.value := by
  intro s hs hpresent
  cases t with
  | none => exact absurd hpresent (empty_root_all_absent c f hsat hroot.symm s hs)
  | some tree => exact (present_reads c f hheight hsat hvalid hroot hs hpresent).2.2

/-- **Update soundness** (doc §1 and §6.2 c–e; the statement of `verify_patricia_update`):
given the committed canonical prev trie, a satisfying update assignment proves there is a
canonical `T_new` hashing to `newRoot` such that

* every **live** row's key maps to `prevValue` in the prev trie and to `newValue` in `T_new`
  — `0` meaning *proven absent* on that side (the false-absence closure, §6.2 c);
* every key that is no live row's key maps identically in both tries (the shared-sibling
  agreement, §6.2 d);
* `(0, 0)` rows assert nothing beyond the second bullet (P9) — they simply are not live.

Uniqueness of `T_new` is `hashOf_inj` (proven): any canonical trie hashing to `newRoot` equals
this one.

**Interface precondition** (`hdistinct`): no two live rows share a key. The circuit does not
enforce this, and the statement is false without it: a delete-shaped and an insert-shaped row
on one key (`(k, v, 0)` and `(k, 0, w)`) satisfy both folds as an ordinary overwrite witness
while each row's absence claim is wrong — the duplicate-position rejection never fires
because each fold sees only one live entry at position `(0, k)`. The batch producers
guarantee the precondition: dict squashing emits one row per key, and nonce batches are
insert-only (same-key insert-shaped rows do collide in the new fold and are rejected). A
`(0, 0)` row may share its key freely — the guards make the pairwise condition vacuous. -/
theorem update_sound (c : Crypto) (u : UpdateAssign) (tPrev : Option Tree)
    (hheight : u.height ≤ c.maxLength)
    (hdistinct : u.rows.Pairwise fun r₁ r₂ => r₁.isLive → r₂.isLive → r₁.key ≠ r₂.key)
    (hsat : u.sat c)
    (hvalid : validOpt tPrev u.height)
    (hroot : rootHashOf c tPrev = u.prevRoot) :
    ∃ tNew : Option Tree,
      validOpt tNew u.height ∧
      rootHashOf c tNew = u.newRoot ∧
      (∀ r ∈ u.rows, r.isLive →
        atOpt tPrev u.height r.key = r.prevValue ∧
        atOpt tNew u.height r.key = r.newValue) ∧
      (∀ k : Nat, (∀ r ∈ u.rows, r.isLive → k ≠ r.key) →
        atOpt tPrev u.height k = atOpt tNew u.height k) := by
  obtain ⟨hsatP, hsatN⟩ := hsat
  -- The shared siblings are pinned by the prev fold (inert when the prev trie is empty).
  have hsib : ∀ w ∈ u.siblings, w.hash ≠ 0 →
      ∃ st : Tree, st.valid w.height ∧ st.hashOf c = w.hash := by
    intro w hw hne
    cases tPrev with
    | none =>
        exact absurd (by
          rw [empty_root_all_inert c u.prevFold hsatP hroot.symm w
            (sibling_mem_produced c hw)]
          rfl) hne
    | some tp =>
        have hpin := pinned_of_consumed c u.prevFold hheight hsatP hvalid hroot w
          (hsatP.2.2.2.2.mem_iff.mp (sibling_mem_produced c hw)) hne
        obtain ⟨hle, st, hnode, hhash⟩ := hpin
        have hle' : w.height ≤ u.height := hle
        have hnode' : tp.nodeAtDepth (u.height - w.height) w.path = some st := hnode
        obtain ⟨-, hstv⟩ := Tree.nodeAtDepth_valid hvalid hnode'
        have hstv' : st.valid w.height := by
          rw [(by omega : u.height - (u.height - w.height) = w.height)] at hstv
          exact hstv
        exact ⟨st, hstv', hhash⟩
  -- Construct the new tree from the new fold (the decoding argument).
  obtain ⟨tNew, hvN, hrN⟩ :
      ∃ tN : Option Tree, validOpt tN u.height ∧ rootHashOf c tN = u.newRoot := by
    by_cases hnr : u.newRoot = 0
    · exact ⟨none, trivial, hnr.symm⟩
    · obtain ⟨tn, htnv, htnh⟩ := root_tree_exists c u.newFold hsatN hnr hsib
      exact ⟨some tn, htnv, htnh⟩
  refine ⟨tNew, hvN, hrN, ?_, ?_⟩
  · -- The live-row read clauses.
    intro r hr hlive
    have hmemP : (⟨r.key, r.prevValue⟩ : LeafSlot) ∈ u.prevFold.leaves :=
      List.mem_map.mpr ⟨r, hr, rfl⟩
    have hmemN : (⟨r.key, r.newValue⟩ : LeafSlot) ∈ u.newFold.leaves :=
      List.mem_map.mpr ⟨r, hr, rfl⟩
    constructor
    · -- Prev side.
      by_cases hpv : r.prevValue = 0
      · -- Claimed absent in the prev trie: the false-absence closure.
        rw [hpv]
        cases tPrev with
        | none => rfl
        | some tp =>
            have hnv : r.newValue ≠ 0 := fun h => hlive ⟨hpv, h⟩
            cases tNew with
            | none =>
                exact absurd hnv
                  (empty_root_all_absent c u.newFold hsatN hrN.symm _ hmemN)
            | some tn =>
                have hklt : r.key < 2 ^ u.height :=
                  (present_reads c u.newFold hheight hsatN hvN hrN hmemN hnv).1
                have hrpne : u.prevRoot ≠ 0 := by
                  intro h
                  rw [h] at hroot
                  exact valid_hashOf_ne_zero c hvalid hheight hroot
                have hnrne : u.newRoot ≠ 0 := by
                  intro h
                  rw [h] at hrN
                  exact valid_hashOf_ne_zero c hvN hheight hrN
                have mP := master c u.prevFold hheight hsatP hvalid hroot hklt hrpne
                have mN := master c u.newFold hheight hsatN hvN hrN hklt hnrne
                have hLP : u.prevFold.leaves.countP (LeafSlot.presentAt r.key) = 0 := by
                  show (u.rows.map fun r' => (⟨r'.key, r'.prevValue⟩ : LeafSlot)).countP
                    (LeafSlot.presentAt r.key) = 0
                  rw [List.countP_map]
                  refine countP_eq_zero_of_pairwise_key hdistinct hr hlive ?_ ?_
                  · intro r' hp'
                    obtain ⟨hv0, hk0⟩ := LeafSlot.presentAt_iff.mp hp'
                    exact ⟨fun hpair => hv0 hpair.1, hk0⟩
                  · simp [Function.comp, LeafSlot.presentAt, hpv]
                have hLN : 0 < u.newFold.leaves.countP (LeafSlot.presentAt r.key) :=
                  countP_pos_of_mem hmemN (LeafSlot.presentAt_iff.mpr ⟨hnv, rfl⟩)
                have hSP : u.prevFold.siblings.countP (SUnit.onPath r.key)
                    = u.newFold.siblings.countP (SUnit.onPath r.key) := rfl
                have hBP : 0 < u.prevFold.edges.countP (EdgeSlot.divergesAt c r.key) := by
                  omega
                obtain ⟨s, hs, hdiv⟩ := exists_of_countP_pos hBP
                exact diverges_absent c u.prevFold hheight hsatP hvalid hroot hs hdiv
      · -- Present in the prev trie: pinning reads it.
        cases tPrev with
        | none =>
            exact absurd hpv (empty_root_all_absent c u.prevFold hsatP hroot.symm _ hmemP)
        | some tp =>
            exact (present_reads c u.prevFold hheight hsatP hvalid hroot hmemP hpv).2.2
    · -- New side (mirror image).
      by_cases hnv : r.newValue = 0
      · rw [hnv]
        cases tNew with
        | none => rfl
        | some tn =>
            have hpv : r.prevValue ≠ 0 := fun h => hlive ⟨h, hnv⟩
            cases tPrev with
            | none =>
                exact absurd hpv
                  (empty_root_all_absent c u.prevFold hsatP hroot.symm _ hmemP)
            | some tp =>
                have hklt : r.key < 2 ^ u.height :=
                  (present_reads c u.prevFold hheight hsatP hvalid hroot hmemP hpv).1
                have hrpne : u.prevRoot ≠ 0 := by
                  intro h
                  rw [h] at hroot
                  exact valid_hashOf_ne_zero c hvalid hheight hroot
                have hnrne : u.newRoot ≠ 0 := by
                  intro h
                  rw [h] at hrN
                  exact valid_hashOf_ne_zero c hvN hheight hrN
                have mP := master c u.prevFold hheight hsatP hvalid hroot hklt hrpne
                have mN := master c u.newFold hheight hsatN hvN hrN hklt hnrne
                have hLN : u.newFold.leaves.countP (LeafSlot.presentAt r.key) = 0 := by
                  show (u.rows.map fun r' => (⟨r'.key, r'.newValue⟩ : LeafSlot)).countP
                    (LeafSlot.presentAt r.key) = 0
                  rw [List.countP_map]
                  refine countP_eq_zero_of_pairwise_key hdistinct hr hlive ?_ ?_
                  · intro r' hp'
                    obtain ⟨hv0, hk0⟩ := LeafSlot.presentAt_iff.mp hp'
                    exact ⟨fun hpair => hv0 hpair.2, hk0⟩
                  · simp [Function.comp, LeafSlot.presentAt, hnv]
                have hLP : 0 < u.prevFold.leaves.countP (LeafSlot.presentAt r.key) :=
                  countP_pos_of_mem hmemP (LeafSlot.presentAt_iff.mpr ⟨hpv, rfl⟩)
                have hSP : u.prevFold.siblings.countP (SUnit.onPath r.key)
                    = u.newFold.siblings.countP (SUnit.onPath r.key) := rfl
                have hBN : 0 < u.newFold.edges.countP (EdgeSlot.divergesAt c r.key) := by
                  omega
                obtain ⟨s, hs, hdiv⟩ := exists_of_countP_pos hBN
                exact diverges_absent c u.newFold hheight hsatN hvN hrN hs hdiv
      · cases tNew with
        | none =>
            exact absurd hnv (empty_root_all_absent c u.newFold hsatN hrN.symm _ hmemN)
        | some tn =>
            exact (present_reads c u.newFold hheight hsatN hvN hrN hmemN hnv).2.2
  · -- Off-row agreement: every keyspace key is behind a shared sibling or diverges on
    -- both sides; out-of-keyspace keys read 0 everywhere.
    intro k hk
    by_cases hklt : k < 2 ^ u.height
    · have hLP : u.prevFold.leaves.countP (LeafSlot.presentAt k) = 0 := by
        show (u.rows.map fun r' => (⟨r'.key, r'.prevValue⟩ : LeafSlot)).countP
          (LeafSlot.presentAt k) = 0
        rw [List.countP_map]
        apply countP_eq_zero_of
        intro r' hr' hp'
        obtain ⟨hv0, hk0⟩ := LeafSlot.presentAt_iff.mp hp'
        exact hk r' hr' (fun hpair => hv0 hpair.1) hk0.symm
      have hLN : u.newFold.leaves.countP (LeafSlot.presentAt k) = 0 := by
        show (u.rows.map fun r' => (⟨r'.key, r'.newValue⟩ : LeafSlot)).countP
          (LeafSlot.presentAt k) = 0
        rw [List.countP_map]
        apply countP_eq_zero_of
        intro r' hr' hp'
        obtain ⟨hv0, hk0⟩ := LeafSlot.presentAt_iff.mp hp'
        exact hk r' hr' (fun hpair => hv0 hpair.2) hk0.symm
      cases tPrev with
      | none =>
          have hS0 : u.newFold.siblings.countP (SUnit.onPath k) = 0 := by
            apply countP_eq_zero_of
            intro w hw hon
            have hin := empty_root_all_inert c u.prevFold hsatP hroot.symm w
              (sibling_mem_produced c hw)
            exact (SUnit.onPath_iff.mp hon).1 (by rw [hin]; rfl)
          cases tNew with
          | none => rfl
          | some tn =>
              have hnrne : u.newRoot ≠ 0 := by
                intro h
                rw [h] at hrN
                exact valid_hashOf_ne_zero c hvN hheight hrN
              have mN := master c u.newFold hheight hsatN hvN hrN hklt hnrne
              have hBN : 0 < u.newFold.edges.countP (EdgeSlot.divergesAt c k) := by omega
              obtain ⟨s, hs, hdiv⟩ := exists_of_countP_pos hBN
              have habs : tn.at' u.height k = 0 :=
                diverges_absent c u.newFold hheight hsatN hvN hrN hs hdiv
              exact habs.symm
      | some tp =>
          have hrpne : u.prevRoot ≠ 0 := by
            intro h
            rw [h] at hroot
            exact valid_hashOf_ne_zero c hvalid hheight hroot
          have mP := master c u.prevFold hheight hsatP hvalid hroot hklt hrpne
          cases tNew with
          | none =>
              have hS0 : u.prevFold.siblings.countP (SUnit.onPath k) = 0 := by
                apply countP_eq_zero_of
                intro w hw hon
                have hin := empty_root_all_inert c u.newFold hsatN hrN.symm w
                  (sibling_mem_produced c hw)
                exact (SUnit.onPath_iff.mp hon).1 (by rw [hin]; rfl)
              have hBP : 0 < u.prevFold.edges.countP (EdgeSlot.divergesAt c k) := by omega
              obtain ⟨s, hs, hdiv⟩ := exists_of_countP_pos hBP
              exact diverges_absent c u.prevFold hheight hsatP hvalid hroot hs hdiv
          | some tn =>
              have hnrne : u.newRoot ≠ 0 := by
                intro h
                rw [h] at hrN
                exact valid_hashOf_ne_zero c hvN hheight hrN
              have mN := master c u.newFold hheight hsatN hvN hrN hklt hnrne
              have hSP : u.prevFold.siblings.countP (SUnit.onPath k)
                  = u.newFold.siblings.countP (SUnit.onPath k) := rfl
              by_cases hS : 0 < u.newFold.siblings.countP (SUnit.onPath k)
              · -- A shared sibling covers `k`: equal subtrees on both sides.
                obtain ⟨w, hw, hon⟩ := exists_of_countP_pos hS
                obtain ⟨stP, hstvP, hleP, hnodeP, hhashP, hatP⟩ :=
                  sibling_covers c u.prevFold hheight hsatP hvalid hroot hw hon
                obtain ⟨stN, hstvN, hleN, hnodeN, hhashN, hatN⟩ :=
                  sibling_covers c u.newFold hheight hsatN hvN hrN hw hon
                have hleP' : w.height ≤ u.height := hleP
                have hwle : w.height ≤ c.maxLength := by omega
                have hst_eq : stP = stN :=
                  hashOf_inj c hwle hstvP hstvN (hhashP.trans hhashN.symm)
                have hatP' : tp.at' u.height k = stP.at' w.height (k % 2 ^ w.height) := hatP
                have hatN' : tn.at' u.height k = stN.at' w.height (k % 2 ^ w.height) := hatN
                show tp.at' u.height k = tn.at' u.height k
                rw [hatP', hatN', hst_eq]
              · -- No sibling covers `k`: both folds hold a diverging edge on its path.
                have hBP : 0 < u.prevFold.edges.countP (EdgeSlot.divergesAt c k) := by
                  omega
                have hBN : 0 < u.newFold.edges.countP (EdgeSlot.divergesAt c k) := by
                  omega
                obtain ⟨sP, hsP, hdivP⟩ := exists_of_countP_pos hBP
                obtain ⟨sN, hsN, hdivN⟩ := exists_of_countP_pos hBN
                have h1 : tp.at' u.height k = 0 :=
                  diverges_absent c u.prevFold hheight hsatP hvalid hroot hsP hdivP
                have h2 : tn.at' u.height k = 0 :=
                  diverges_absent c u.newFold hheight hsatN hvN hrN hsN hdivN
                show tp.at' u.height k = tn.at' u.height k
                rw [h1, h2]
    · -- Out of the keyspace: every canonical tree reads 0.
      have hout : 2 ^ u.height ≤ k := by omega
      cases tPrev with
      | none =>
          cases tNew with
          | none => rfl
          | some tn => exact (Tree.at'_out_of_range hvN hout).symm
      | some tp =>
          cases tNew with
          | none => exact Tree.at'_out_of_range hvalid hout
          | some tn =>
              have h1 : tp.at' u.height k = 0 := Tree.at'_out_of_range hvalid hout
              have h2 : tn.at' u.height k = 0 := Tree.at'_out_of_range hvN hout
              show tp.at' u.height k = tn.at' u.height k
              rw [h1, h2]

/-- The uniqueness corollary, from the proven `hashOf_inj`: two canonical tries with the
update's new root are equal — `newRoot` is a *commitment*, not just a hash. Fully proven
(the empty case uses `valid_hashOf_ne_zero`). -/
theorem newRoot_commits (c : Crypto) {h : Nat} (hh : h ≤ c.maxLength)
    {t₁ t₂ : Option Tree}
    (hv₁ : validOpt t₁ h) (hv₂ : validOpt t₂ h)
    (he : rootHashOf c t₁ = rootHashOf c t₂) : t₁ = t₂ := by
  cases t₁ with
  | none =>
      cases t₂ with
      | none => rfl
      | some t =>
          exact absurd he.symm (valid_hashOf_ne_zero c hv₂ hh)
  | some t =>
      cases t₂ with
      | none => exact absurd he (valid_hashOf_ne_zero c hv₁ hh)
      | some t' =>
          have := hashOf_inj c hh hv₁ hv₂ he
          rw [this]

end PatriciaCircuit
