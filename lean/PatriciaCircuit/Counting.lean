import PatriciaCircuit.Pinning

/-!
# The counting identity (`docs/patricia-circuit.md` §5.3, §6.2 c–d)

For a committed fold and a key `k` in the keyspace, count the live units sitting at `k`'s
prefix positions (`path = k >> height`) on both sides of the permutation. Consumers pair
off against slot outputs one level up (each slot consumes at most one unit on `k`'s path,
and the position relations invert exactly on pinned units), so everything cancels except
the boundary terms:

`#present-leaf-slots-at-k + #diverging-edges-on-k + #siblings-covering-k = 1` (root ≠ 0).

This one identity is the duplicate-position lemma, the absence argument, and the frontier
exhaustiveness, all at once.
-/

namespace PatriciaCircuit

/-! ### `countP` toolkit -/

theorem perm_countP {α} {l₁ l₂ : List α} (p : α → Bool) (h : l₁.Perm l₂) :
    l₁.countP p = l₂.countP p := by
  induction h with
  | nil => rfl
  | cons x _ ih => simp [List.countP_cons, ih]
  | swap x y l => simp [List.countP_cons]; omega
  | trans _ _ ih₁ ih₂ => exact ih₁.trans ih₂

theorem countP_congr' {α} {l : List α} {p q : α → Bool}
    (h : ∀ a ∈ l, p a = q a) : l.countP p = l.countP q := by
  induction l with
  | nil => rfl
  | cons a t ih =>
      simp only [List.countP_cons, h a (List.mem_cons_self),
        ih fun b hb => h b (List.mem_cons_of_mem a hb)]

theorem countP_eq_zero_of {α} {l : List α} {p : α → Bool}
    (h : ∀ a ∈ l, ¬ p a = true) : l.countP p = 0 := by
  induction l with
  | nil => rfl
  | cons a t ih =>
      rw [List.countP_cons, ih fun b hb => h b (List.mem_cons_of_mem a hb)]
      simp [h a (List.mem_cons_self)]

theorem countP_pos_of_mem {α} {l : List α} {p : α → Bool} {a : α}
    (ha : a ∈ l) (hp : p a = true) : 0 < l.countP p := by
  induction l with
  | nil => cases ha
  | cons b t ih =>
      rw [List.countP_cons]
      rcases List.mem_cons.mp ha with h | h
      · subst h; simp [hp]
      · have := ih h; omega

theorem exists_of_countP_pos {α} {l : List α} {p : α → Bool}
    (h : 0 < l.countP p) : ∃ a ∈ l, p a = true := by
  induction l with
  | nil => simp [List.countP_nil] at h
  | cons a t ih =>
      cases hpa : p a
      · rw [List.countP_cons, hpa] at h
        have h' : 0 < List.countP p t := by
          rcases Nat.eq_zero_or_pos (List.countP p t) with hz | hp'
          · rw [hz] at h; simp at h
          · exact hp'
        obtain ⟨b, hb, hpb⟩ := ih h'
        exact ⟨b, List.mem_cons_of_mem a hb, hpb⟩
      · exact ⟨a, List.mem_cons_self, hpa⟩

theorem countP_or_split {α} {l : List α} {p q : α → Bool}
    (h : ∀ a ∈ l, ¬(p a = true ∧ q a = true)) :
    l.countP (fun a => p a || q a) = l.countP p + l.countP q := by
  induction l with
  | nil => rfl
  | cons a t ih =>
      have ht := ih fun b hb => h b (List.mem_cons_of_mem a hb)
      have ha := h a (List.mem_cons_self)
      simp only [List.countP_cons, ht]
      cases hpa : p a
      · cases hqa : q a
        · simp [hpa, hqa]
        · simp [hpa, hqa]; omega
      · cases hqa : q a
        · simp [hpa, hqa]; omega
        · exact absurd ⟨hpa, hqa⟩ ha

theorem countP_flatMap_pair {α β} {l : List α} {f g : α → β} {p : β → Bool} :
    (l.flatMap fun s => [f s, g s]).countP p
      = l.countP (fun s => p (f s)) + l.countP (fun s => p (g s)) := by
  induction l with
  | nil => rfl
  | cons a t ih =>
      simp only [List.flatMap_cons, List.countP_append, List.countP_cons, List.countP_nil, ih]
      omega

theorem bool_eq_iff {a b : Bool} (h : a = true ↔ b = true) : a = b := by
  cases a <;> cases b <;> simp_all

/-! ### The position predicates -/

/-- Live and at `k`'s prefix position: `path = k >> height`. -/
def SUnit.onPath (k : Nat) (u : SUnit) : Bool :=
  u.hash != 0 && u.path == k / 2 ^ u.height

theorem SUnit.onPath_iff {k : Nat} {u : SUnit} :
    u.onPath k = true ↔ u.hash ≠ 0 ∧ u.path = k / 2 ^ u.height := by
  simp [SUnit.onPath]

/-- The slot claims `k` present: a non-zero value at exactly this key. -/
def LeafSlot.presentAt (k : Nat) (s : LeafSlot) : Bool :=
  s.value != 0 && s.key == k

theorem LeafSlot.presentAt_iff {k : Nat} {s : LeafSlot} :
    s.presentAt k = true ↔ s.value ≠ 0 ∧ s.key = k := by
  simp [LeafSlot.presentAt]

/-- The edge sits on `k`'s path but its compressed bits disagree with `k` — the shape of a
proven absence. -/
def EdgeSlot.divergesAt (c : Crypto) (k : Nat) (s : EdgeSlot) : Bool :=
  (s.out c).onPath k && !(s.edgePath == k / 2 ^ s.bottom.height % 2 ^ s.length)

theorem EdgeSlot.divergesAt_iff {c : Crypto} {k : Nat} {s : EdgeSlot} :
    s.divergesAt c k = true
      ↔ (s.out c).onPath k = true ∧ s.edgePath ≠ k / 2 ^ s.bottom.height % 2 ^ s.length := by
  simp [EdgeSlot.divergesAt]

theorem onPath_entry (k : Nat) (s : LeafSlot) :
    (s.entry).onPath k = s.presentAt k := by
  by_cases hval : s.value = 0
  · simp [SUnit.onPath, LeafSlot.entry, LeafSlot.presentAt, hval, SUnit.inert]
  · simp [SUnit.onPath, LeafSlot.entry, LeafSlot.presentAt, hval, Nat.pow_zero, Nat.div_one]

/-! ### The per-slot bridges -/

/-- A live binary out is on `k`'s path iff exactly one child is: the doubling relations
invert on pinned positions, and the two children sit at sibling positions. -/
theorem binary_bridge (c : Crypto) {t : Tree} {H : Nat} (hH : H ≤ c.maxLength)
    (hv : t.valid H) {s : BinarySlot} (hok : s.ok)
    (hpin : (s.out c).hash ≠ 0 → Pinned c t H (s.out c)) (k : Nat) :
    (s.out c).onPath k = (s.left.onPath k || s.right.onPath k) := by
  by_cases hlive : s.isLive
  · have houth : (s.out c).hash = c.hash s.left.hash s.right.hash := by
      simp [BinarySlot.out, hlive]
    have hne : (s.out c).hash ≠ 0 := by rw [houth]; exact c.hash_ne_zero _ _
    obtain ⟨hle, hqlt, hlp, hrp, -, -⟩ := pinned_binary c hH hv hok hlive (hpin hne)
    obtain ⟨-, -, hlhash, hrhash⟩ := hok.2.2.2 hlive
    have hout_h : (s.out c).height = s.left.height + 1 := by
      simp [BinarySlot.out, BinarySlot.liveBit, hlive]
    have hrh : s.right.height = s.left.height := hok.1.symm
    have hsplit := Nat.div_add_mod (k / 2 ^ s.left.height) 2
    rw [div_two_pow_succ k s.left.height] at hsplit
    have hbit : k / 2 ^ s.left.height % 2 < 2 := Nat.mod_lt _ (by omega)
    apply bool_eq_iff
    rw [Bool.or_eq_true]
    constructor
    · intro hL
      obtain ⟨-, hpath⟩ := SUnit.onPath_iff.mp hL
      rw [hout_h] at hpath
      have hpath' : s.outPath = k / 2 ^ (s.left.height + 1) := hpath
      rcases (by omega :
          2 * s.outPath = k / 2 ^ s.left.height ∨
          2 * s.outPath + 1 = k / 2 ^ s.left.height) with h | h
      · exact Or.inl (SUnit.onPath_iff.mpr ⟨hlhash, by rw [hlp, h]⟩)
      · exact Or.inr (SUnit.onPath_iff.mpr ⟨hrhash, by rw [hrp, hrh, h]⟩)
    · intro hLR
      rcases hLR with h | h
      · obtain ⟨-, hpath⟩ := SUnit.onPath_iff.mp h
        rw [hlp] at hpath
        exact SUnit.onPath_iff.mpr ⟨hne, by rw [hout_h]; show s.outPath = _; omega⟩
      · obtain ⟨-, hpath⟩ := SUnit.onPath_iff.mp h
        rw [hrp, hrh] at hpath
        exact SUnit.onPath_iff.mpr ⟨hne, by rw [hout_h]; show s.outPath = _; omega⟩
  · have hpair : s.left = SUnit.inert ∧ s.right = SUnit.inert := by
      by_cases h : s.left = SUnit.inert ∧ s.right = SUnit.inert
      · exact h
      · exact absurd h hlive
    have hout : (s.out c).hash = 0 := by simp [BinarySlot.out, hlive]
    have h1 : (s.out c).onPath k = false := by simp [SUnit.onPath, hout]
    have h2 : s.left.onPath k = false := by simp [SUnit.onPath, hpair.1, SUnit.inert]
    have h3 : s.right.onPath k = false := by simp [SUnit.onPath, hpair.2, SUnit.inert]
    rw [h1, h2, h3]
    rfl

/-- A live edge out on `k`'s path splits: either the compressed bits match `k` and the
bottom continues on the path, or they diverge (`divergesAt`) — never both. -/
theorem edge_bridge (c : Crypto) {t : Tree} {H : Nat} (hH : H ≤ c.maxLength)
    (hv : t.valid H) {s : EdgeSlot} (hok : s.ok)
    (hpin : (s.out c).hash ≠ 0 → Pinned c t H (s.out c)) (k : Nat) :
    (s.out c).onPath k = (s.bottom.onPath k || s.divergesAt c k) ∧
      ¬(s.bottom.onPath k = true ∧ s.divergesAt c k = true) := by
  by_cases hlive : s.isLive
  · have hlen1 : 1 ≤ s.length := hok.2.1 hlive
    have houth : (s.out c).hash = c.hash s.bottom.hash s.edgePath + s.length := by
      simp [EdgeSlot.out, hlive]
    have hne : (s.out c).hash ≠ 0 := by omega
    have hbhash : s.bottom.hash ≠ 0 := hok.2.2.2.2.2.1 hlive
    obtain ⟨b', hle, hqlt, hnode, -, hbpath, -⟩ := pinned_edge c hH hv hok hlive (hpin hne)
    have hout_h : (s.out c).height = s.bottom.height + s.length := rfl
    have hep : s.edgePath < 2 ^ s.length := hok.2.2.1
    have hYX : k / 2 ^ s.bottom.height / 2 ^ s.length
        = k / 2 ^ (s.bottom.height + s.length) := div_two_pow_div _ _ _
    have hXsplit := Nat.div_add_mod (k / 2 ^ s.bottom.height) (2 ^ s.length)
    have hXmod : k / 2 ^ s.bottom.height % 2 ^ s.length < 2 ^ s.length :=
      Nat.mod_lt _ (two_pow_pos _)
    -- bottom on path ⇒ out on path and bits match
    have hbot_pins : s.bottom.onPath k = true →
        s.outPath = k / 2 ^ (s.bottom.height + s.length)
          ∧ s.edgePath = k / 2 ^ s.bottom.height % 2 ^ s.length := by
      intro h
      obtain ⟨-, hpath⟩ := SUnit.onPath_iff.mp h
      rw [hbpath] at hpath
      have hdiv : k / 2 ^ s.bottom.height / 2 ^ s.length = s.outPath := by
        rw [← hpath, Nat.mul_comm s.outPath, Nat.mul_add_div (two_pow_pos _),
          Nat.div_eq_of_lt hep, Nat.add_zero]
      have hmod : k / 2 ^ s.bottom.height % 2 ^ s.length = s.edgePath := by
        rw [← hpath, Nat.mul_comm s.outPath, Nat.mul_add_mod, Nat.mod_eq_of_lt hep]
      rw [← hYX]
      exact ⟨hdiv.symm, hmod.symm⟩
    constructor
    · apply bool_eq_iff
      rw [Bool.or_eq_true]
      constructor
      · intro hL
        obtain ⟨-, hpath⟩ := SUnit.onPath_iff.mp hL
        rw [hout_h] at hpath
        have hq : s.outPath = k / 2 ^ (s.bottom.height + s.length) := hpath
        by_cases hmatch : s.edgePath = k / 2 ^ s.bottom.height % 2 ^ s.length
        · refine Or.inl (SUnit.onPath_iff.mpr ⟨hbhash, ?_⟩)
          rw [hbpath, hq, ← hYX, hmatch, Nat.mul_comm]
          exact hXsplit
        · exact Or.inr (EdgeSlot.divergesAt_iff.mpr ⟨hL, hmatch⟩)
      · intro hLR
        rcases hLR with h | h
        · exact SUnit.onPath_iff.mpr ⟨hne, by rw [hout_h]; exact (hbot_pins h).1⟩
        · exact (EdgeSlot.divergesAt_iff.mp h).1
    · intro ⟨hbot, hdiv⟩
      exact (EdgeSlot.divergesAt_iff.mp hdiv).2 (hbot_pins hbot).2
  · have htriple : s.bottom = SUnit.inert ∧ s.length = 0 ∧ s.edgePath = 0 := by
      by_cases h : s.bottom = SUnit.inert ∧ s.length = 0 ∧ s.edgePath = 0
      · exact h
      · exact absurd h hlive
    have hout : (s.out c).hash = 0 := by simp [EdgeSlot.out, hlive]
    have h1 : (s.out c).onPath k = false := by simp [SUnit.onPath, hout]
    have h2 : s.bottom.onPath k = false := by
      simp [SUnit.onPath, htriple.1, SUnit.inert]
    have h3 : s.divergesAt c k = false := by simp [EdgeSlot.divergesAt, h1]
    rw [h1, h2, h3]
    exact ⟨rfl, by simp⟩

/-- Two children of one binary slot never share a position: sibling paths differ. -/
theorem binary_excl {s : BinarySlot} (hok : s.ok) (k : Nat) :
    ¬(s.left.onPath k = true ∧ s.right.onPath k = true) := by
  intro ⟨hl, hr⟩
  obtain ⟨hlhash, hlp⟩ := SUnit.onPath_iff.mp hl
  obtain ⟨-, hrp⟩ := SUnit.onPath_iff.mp hr
  rw [hok.2.1] at hlp
  rw [hok.2.2.1] at hrp
  have hlive : s.isLive := by
    intro ⟨hli, _⟩
    exact hlhash (by rw [hli]; rfl)
  have hbit : s.liveBit = 1 := by simp [BinarySlot.liveBit, hlive]
  rw [hbit, ← hok.1] at hrp
  have hcontra : 2 * s.outPath % wordModulus = (2 * s.outPath + 1) % wordModulus :=
    hlp.trans hrp.symm
  unfold wordModulus at hcontra
  omega

/-! ### The master identity -/

/-- **The counting identity** (§5.3 + §6.2 c–d in one line): against a non-empty committed
root, each keyspace key is accounted for exactly once — by a present leaf slot, by a
diverging edge on its path, or by a covering sibling. -/
theorem master (c : Crypto) (f : Fold) {t : Tree}
    (hH : f.height ≤ c.maxLength) (hsat : f.sat c) (hv : t.valid f.height)
    (hroot : t.hashOf c = f.root) {k : Nat} (hk : k < 2 ^ f.height) (hne : f.root ≠ 0) :
    f.leaves.countP (LeafSlot.presentAt k)
      + f.edges.countP (EdgeSlot.divergesAt c k)
      + f.siblings.countP (SUnit.onPath k) = 1 := by
  have hperm := hsat.2.2.2.2
  have hcount := perm_countP (SUnit.onPath k) hperm
  unfold Fold.produced Fold.consumed at hcount
  simp only [List.countP_append] at hcount
  -- leaf entries are exactly the present-at-k claims
  have hA : (f.leaves.map LeafSlot.entry).countP (SUnit.onPath k)
      = f.leaves.countP (LeafSlot.presentAt k) := by
    rw [List.countP_map]
    exact countP_congr' fun s _ => onPath_entry k s
  -- binary outs bridge to their children, one per slot
  have hB : (f.binaries.map (BinarySlot.out c)).countP (SUnit.onPath k)
      = f.binaries.countP (fun s => s.left.onPath k)
        + f.binaries.countP (fun s => s.right.onPath k) := by
    rw [List.countP_map]
    refine (countP_congr' (q := fun s => s.left.onPath k || s.right.onPath k) ?_).trans
      (countP_or_split ?_)
    · exact fun s hs => binary_bridge c hH hv (hsat.1 s hs)
        (fun hne' => pinned_of_consumed c f hH hsat hv hroot (s.out c)
          (hperm.mem_iff.mp (binary_out_mem_produced c hs)) hne') k
    · exact fun s hs => binary_excl (hsat.1 s hs) k
  -- edge outs bridge to their bottoms plus the diverging ones
  have hC : (f.edges.map (EdgeSlot.out c)).countP (SUnit.onPath k)
      = f.edges.countP (fun s => s.bottom.onPath k)
        + f.edges.countP (EdgeSlot.divergesAt c k) := by
    rw [List.countP_map]
    refine (countP_congr' (q := fun s => s.bottom.onPath k || s.divergesAt c k) ?_).trans
      (countP_or_split ?_)
    · exact fun s hs => (edge_bridge c hH hv (hsat.2.1 s hs)
        (fun hne' => pinned_of_consumed c f hH hsat hv hroot (s.out c)
          (hperm.mem_iff.mp (edge_out_mem_produced c hs)) hne') k).1
    · exact fun s hs => (edge_bridge c hH hv (hsat.2.1 s hs)
        (fun hne' => pinned_of_consumed c f hH hsat hv hroot (s.out c)
          (hperm.mem_iff.mp (edge_out_mem_produced c hs)) hne') k).2
  -- consumed side: children per slot, bottoms, and the live root entry
  have hE : (f.binaries.flatMap fun s => [s.left, s.right]).countP (SUnit.onPath k)
      = f.binaries.countP (fun s => s.left.onPath k)
        + f.binaries.countP (fun s => s.right.onPath k) := by
    exact countP_flatMap_pair
  have hF : (f.edges.map EdgeSlot.bottom).countP (SUnit.onPath k)
      = f.edges.countP (fun s => s.bottom.onPath k) := by
    rw [List.countP_map]
    exact countP_congr' fun s _ => rfl
  have hG : ([f.rootEntry].countP (SUnit.onPath k)) = 1 := by
    have hon : (f.rootEntry).onPath k = true := by
      apply SUnit.onPath_iff.mpr
      refine ⟨hne, ?_⟩
      show (0 : Nat) = k / 2 ^ f.rootEntry.height
      have hh : f.rootEntry.height = f.height := by simp [Fold.rootEntry, hne]
      rw [hh, Nat.div_eq_of_lt hk]
    simp [List.countP_cons, hon]
  rw [hA, hB, hC, hE, hF, hG] at hcount
  omega

/-! ### Decoding the three master terms -/

/-- A diverging edge on `k`'s path proves `k` absent. -/
theorem diverges_absent (c : Crypto) (f : Fold) {t : Tree}
    (hH : f.height ≤ c.maxLength) (hsat : f.sat c) (hv : t.valid f.height)
    (hroot : t.hashOf c = f.root) {k : Nat} {s : EdgeSlot} (hs : s ∈ f.edges)
    (hdiv : s.divergesAt c k = true) : t.at' f.height k = 0 := by
  obtain ⟨hon, hmismatch⟩ := EdgeSlot.divergesAt_iff.mp hdiv
  obtain ⟨hhne, hpath⟩ := SUnit.onPath_iff.mp hon
  have hok := hsat.2.1 s hs
  have hlive : s.isLive := by
    by_cases h : s.isLive
    · exact h
    · exact absurd (by simp [EdgeSlot.out, h] : (s.out c).hash = 0) hhne
  have hpin := pinned_of_consumed c f hH hsat hv hroot (s.out c)
    (hsat.2.2.2.2.mem_iff.mp (edge_out_mem_produced c hs)) hhne
  obtain ⟨b', hle, hqlt, hnode, -, -, -⟩ := pinned_edge c hH hv hok hlive hpin
  have hout_h : (s.out c).height = s.bottom.height + s.length := rfl
  rw [hout_h] at hpath
  have hHd : f.height - (f.height - (s.bottom.height + s.length))
      = s.bottom.height + s.length := by omega
  refine Tree.at'_diverged (d := f.height - (s.bottom.height + s.length))
    (length := s.length) (path := s.edgePath) (bottom := b') hv (Nat.sub_le _ _) ?_ ?_ ?_
  · rw [hHd]
    omega
  · rw [hHd, ← hpath]
    exact hnode
  · rw [hHd, (by omega : s.bottom.height + s.length - s.length = s.bottom.height)]
    exact fun h => hmismatch h.symm

/-- A sibling covering `k` pins a real subtree there; the lookup factors through it. -/
theorem sibling_covers (c : Crypto) (f : Fold) {t : Tree}
    (hH : f.height ≤ c.maxLength) (hsat : f.sat c) (hv : t.valid f.height)
    (hroot : t.hashOf c = f.root) {k : Nat} {w : SUnit} (hw : w ∈ f.siblings)
    (hon : w.onPath k = true) :
    ∃ st : Tree, st.valid w.height ∧ w.height ≤ f.height ∧
      t.nodeAtDepth (f.height - w.height) (k / 2 ^ w.height) = some st ∧
      st.hashOf c = w.hash ∧
      t.at' f.height k = st.at' w.height (k % 2 ^ w.height) := by
  obtain ⟨hhne, hpath⟩ := SUnit.onPath_iff.mp hon
  have hpin := pinned_of_consumed c f hH hsat hv hroot w
    (hsat.2.2.2.2.mem_iff.mp (sibling_mem_produced c hw)) hhne
  obtain ⟨hle, st, hnode, hhash⟩ := hpin
  rw [hpath] at hnode
  obtain ⟨-, hstv⟩ := Tree.nodeAtDepth_valid hv hnode
  rw [(by omega : f.height - (f.height - w.height) = w.height)] at hstv
  have hHd : f.height - (f.height - w.height) = w.height := by omega
  have hat := Tree.at'_factor (d := f.height - w.height) (k := k) hv (Nat.sub_le _ _)
    (by rw [hHd]; exact hnode)
  rw [hHd] at hat
  exact ⟨st, hstv, hle, hnode, hhash, hat⟩

/-- Live rows have pairwise-distinct keys, so no *other* row can claim `k = r.key`; a
predicate implying "live at `r`'s key" but false at `r` counts zero. -/
theorem countP_eq_zero_of_pairwise_key {rows : List Row}
    (hdist : rows.Pairwise fun r₁ r₂ => r₁.isLive → r₂.isLive → r₁.key ≠ r₂.key)
    {r : Row} (hr : r ∈ rows) (hlive : r.isLive) {p : Row → Bool}
    (hp : ∀ r', p r' = true → r'.isLive ∧ r'.key = r.key)
    (hpr : p r = false) : rows.countP p = 0 := by
  induction rows with
  | nil => rfl
  | cons a l ih =>
      obtain ⟨ha, hl⟩ := List.pairwise_cons.mp hdist
      rcases List.mem_cons.mp hr with heq | hmem
      · subst heq
        rw [List.countP_cons, countP_eq_zero_of (fun b hb hpb => by
          obtain ⟨hbl, hbk⟩ := hp b hpb
          exact ha b hb hlive hbl hbk.symm)]
        simp [hpr]
      · have hpa : p a = false := by
          cases hq : p a
          · rfl
          · obtain ⟨hal, hak⟩ := hp a hq
            exact absurd hak (ha r hmem hal hlive)
        rw [List.countP_cons, ih hl hmem]
        simp [hpa]

end PatriciaCircuit
