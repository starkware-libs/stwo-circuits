import PatriciaCircuit.Spec

/-!
# Positions in a canonical trie

`nodeAtDepth t d q` is the node of `t` sitting `d` levels below its root at relative path `q`
(`none` off every walk or inside an edge run). This is the spec-side target of the pinning
argument: a unit at height `h` in a fold over a height-`H` trie is pinned to
`nodeAtDepth t (H - h) path`. The file proves the position calculus — composition, path
bounds, subtree validity — and how `at'` factors through nodes.
-/

namespace PatriciaCircuit

/-! ### Two-block path arithmetic -/

theorem two_pow_pos (n : Nat) : 0 < 2 ^ n := Nat.pow_pos (by omega)

theorem two_block_lt {q p d e : Nat} (hq : q < 2 ^ d) (hp : p < 2 ^ e) :
    q * 2 ^ e + p < 2 ^ (d + e) := by
  have h1 : q * 2 ^ e + p < (q + 1) * 2 ^ e := by rw [Nat.add_mul, Nat.one_mul]; omega
  have h2 : (q + 1) * 2 ^ e ≤ 2 ^ d * 2 ^ e := Nat.mul_le_mul_right _ hq
  rw [Nat.pow_add]; omega

theorem div_two_pow_div (k a b : Nat) : k / 2 ^ a / 2 ^ b = k / 2 ^ (a + b) := by
  rw [Nat.div_div_eq_div_mul, Nat.pow_add]

theorem lt_of_div_two_pow_eq_zero {k e : Nat} (h : k / 2 ^ e = 0) : k < 2 ^ e := by
  have hdm := Nat.div_add_mod k (2 ^ e)
  have hm := Nat.mod_lt k (y := 2 ^ e) (two_pow_pos e)
  rw [h] at hdm
  omega

theorem lt_of_div_two_pow_eq_one {k e : Nat} (h : k / 2 ^ e = 1) : k < 2 ^ (e + 1) := by
  have := (Nat.div_lt_iff_lt_mul (two_pow_pos e)).mp (by omega : k / 2 ^ e < 2)
  rw [Nat.pow_succ]; omega

/-- The lower block of a two-block path does not move the upper block's division. -/
theorem div_two_block {q r : Nat} (k e : Nat) (hr : r < 2 ^ e) :
    (q * 2 ^ e + r) / 2 ^ (k + e) = q / 2 ^ k := by
  rw [Nat.pow_add, Nat.mul_comm (2 ^ k), ← Nat.div_div_eq_div_mul]
  congr 1
  rw [Nat.mul_comm q, Nat.mul_add_div (two_pow_pos e), Nat.div_eq_of_lt hr, Nat.add_zero]

/-- The remainder of a two-block path keeps the lower block intact. -/
theorem mod_two_block {q r : Nat} (k e : Nat) (hr : r < 2 ^ e) :
    (q * 2 ^ e + r) % 2 ^ (k + e) = q % 2 ^ k * 2 ^ e + r := by
  have key : q * 2 ^ e + r = 2 ^ (k + e) * (q / 2 ^ k) + (q % 2 ^ k * 2 ^ e + r) := by
    conv => lhs; rw [← Nat.div_add_mod q (2 ^ k)]
    rw [Nat.add_mul, Nat.mul_right_comm, ← Nat.pow_add, Nat.add_assoc]
  rw [key, Nat.mul_add_mod,
    Nat.mod_eq_of_lt (two_block_lt (Nat.mod_lt _ (two_pow_pos k)) hr)]

/-- Reducing then dividing extracts a middle bit block: `(k % 2^(a+b)) >> a = (k >> a) % 2^b`. -/
theorem mod_two_pow_div (k a b : Nat) : k % 2 ^ (a + b) / 2 ^ a = k / 2 ^ a % 2 ^ b := by
  obtain ⟨c, m, hm, rfl⟩ : ∃ c m, m < 2 ^ (a + b) ∧ k = 2 ^ (a + b) * c + m :=
    ⟨k / 2 ^ (a + b), k % 2 ^ (a + b), Nat.mod_lt _ (two_pow_pos _),
      (Nat.div_add_mod k (2 ^ (a + b))).symm⟩
  have hdiv : m / 2 ^ a < 2 ^ b :=
    (Nat.div_lt_iff_lt_mul (two_pow_pos a)).mpr (by rw [Nat.mul_comm, ← Nat.pow_add]; exact hm)
  rw [Nat.mul_add_mod, Nat.mod_eq_of_lt hm, Nat.pow_add, Nat.mul_assoc,
    Nat.mul_add_div (two_pow_pos a), Nat.mul_add_mod, Nat.mod_eq_of_lt hdiv]

theorem mod_two_pow_mod (k a b : Nat) : k % 2 ^ (a + b) % 2 ^ a = k % 2 ^ a :=
  Nat.mod_mod_of_dvd k (Nat.pow_dvd_pow 2 (Nat.le_add_right a b))

theorem div_two_pow_succ (k h : Nat) : k / 2 ^ h / 2 = k / 2 ^ (h + 1) := by
  rw [Nat.div_div_eq_div_mul, ← Nat.pow_succ]

/-! ### Nodes at positions -/

namespace Tree

/-- The node `d` levels below `t`'s root at relative path `q`; `none` when the position is
inside an edge run, off every walk, or out of range (`q ≥ 2^d`). -/
def nodeAtDepth : Tree → Nat → Nat → Option Tree
  | t, 0, q => if q = 0 then some t else none
  | .leaf _, _ + 1, _ => none
  | .binary l r, d + 1, q =>
      if q / 2 ^ d = 0 then l.nodeAtDepth d (q % 2 ^ d)
      else if q / 2 ^ d = 1 then r.nodeAtDepth d (q % 2 ^ d)
      else none
  | .edge length path bottom, d + 1, q =>
      if length ≤ d + 1 ∧ q / 2 ^ (d + 1 - length) = path then
        bottom.nodeAtDepth (d + 1 - length) (q % 2 ^ (d + 1 - length))
      else none

theorem nodeAtDepth_zero_eq (t : Tree) (q : Nat) :
    t.nodeAtDepth 0 q = if q = 0 then some t else none := by
  cases t <;> simp [nodeAtDepth]

@[simp] theorem nodeAtDepth_zero (t : Tree) : t.nodeAtDepth 0 0 = some t := by
  rw [nodeAtDepth_zero_eq]; simp

/-- A node's relative path fits its depth. -/
theorem nodeAtDepth_path_lt {t s : Tree} {h d q : Nat}
    (hv : t.valid h) (hn : t.nodeAtDepth d q = some s) : q < 2 ^ d := by
  cases d with
  | zero =>
      rw [nodeAtDepth_zero_eq] at hn
      split at hn
      · subst ‹q = 0›; exact two_pow_pos 0
      · cases hn
  | succ d =>
      cases t with
      | leaf v => simp [nodeAtDepth] at hn
      | binary l r =>
          simp only [nodeAtDepth] at hn
          split at hn
          · have h0 := lt_of_div_two_pow_eq_zero ‹q / 2 ^ d = 0›
            have hle : (2 : Nat) ^ d ≤ 2 ^ (d + 1) := Nat.pow_le_pow_right (by omega) (by omega)
            omega
          · split at hn
            · exact lt_of_div_two_pow_eq_one ‹q / 2 ^ d = 1›
            · cases hn
      | edge length path bottom =>
          simp only [nodeAtDepth] at hn
          split at hn
          · obtain ⟨hlen, hpath⟩ := ‹length ≤ d + 1 ∧ q / 2 ^ (d + 1 - length) = path›
            have hplt : path < 2 ^ length := hv.2.2.1
            have hdm := Nat.div_add_mod q (2 ^ (d + 1 - length))
            rw [hpath] at hdm
            have hmod : q % 2 ^ (d + 1 - length) < 2 ^ (d + 1 - length) :=
              Nat.mod_lt _ (two_pow_pos _)
            have hq : q < (path + 1) * 2 ^ (d + 1 - length) := by
              calc q = 2 ^ (d + 1 - length) * path + q % 2 ^ (d + 1 - length) := hdm.symm
                _ < 2 ^ (d + 1 - length) * path + 2 ^ (d + 1 - length) := by omega
                _ = (path + 1) * 2 ^ (d + 1 - length) := by
                    rw [Nat.add_mul, Nat.one_mul, Nat.mul_comm]
            have hbound : (path + 1) * 2 ^ (d + 1 - length)
                ≤ 2 ^ length * 2 ^ (d + 1 - length) := Nat.mul_le_mul_right _ hplt
            rw [← Nat.pow_add, (by omega : length + (d + 1 - length) = d + 1)] at hbound
            omega
          · cases hn

/-- Subtrees at depth `d` of a height-`h` canonical tree are canonical at height `h − d`. -/
theorem nodeAtDepth_valid {t s : Tree} {h d q : Nat}
    (hv : t.valid h) (hn : t.nodeAtDepth d q = some s) : d ≤ h ∧ s.valid (h - d) := by
  induction t generalizing h d q with
  | leaf v =>
      cases d with
      | zero =>
          rw [nodeAtDepth_zero_eq] at hn
          split at hn
          · cases hn; simpa using hv
          · cases hn
      | succ d => simp [nodeAtDepth] at hn
  | binary l r ihl ihr =>
      cases d with
      | zero =>
          rw [nodeAtDepth_zero_eq] at hn
          split at hn
          · cases hn; simpa using hv
          · cases hn
      | succ d =>
          obtain ⟨h', rfl, hvl, hvr⟩ := hv
          simp only [nodeAtDepth] at hn
          split at hn
          · obtain ⟨hd, hs⟩ := ihl hvl hn
            refine ⟨by omega, ?_⟩
            rw [(by omega : h' + 1 - (d + 1) = h' - d)]
            exact hs
          · split at hn
            · obtain ⟨hd, hs⟩ := ihr hvr hn
              refine ⟨by omega, ?_⟩
              rw [(by omega : h' + 1 - (d + 1) = h' - d)]
              exact hs
            · cases hn
  | edge length path bottom ih =>
      cases d with
      | zero =>
          rw [nodeAtDepth_zero_eq] at hn
          split at hn
          · cases hn; simpa using hv
          · cases hn
      | succ d =>
          obtain ⟨hlen1, hlenh, hplt, hvb, _⟩ := hv
          simp only [nodeAtDepth] at hn
          split at hn
          · obtain ⟨hlen, _⟩ := ‹length ≤ d + 1 ∧ _›
            obtain ⟨hd, hs⟩ := ih hvb hn
            refine ⟨by omega, ?_⟩
            rw [(by omega : h - (d + 1) = (h - length) - (d + 1 - length))]
            exact hs
          · cases hn

/-- Composition: the node at `e` below the node at `d` is the node at `d + e`. -/
theorem nodeAtDepth_trans {t s s' : Tree} {d q e r : Nat}
    (hr : r < 2 ^ e)
    (h1 : t.nodeAtDepth d q = some s) (h2 : s.nodeAtDepth e r = some s') :
    t.nodeAtDepth (d + e) (q * 2 ^ e + r) = some s' := by
  induction t generalizing d q with
  | leaf v =>
      cases d with
      | zero =>
          rw [nodeAtDepth_zero_eq] at h1
          split at h1
          · cases h1
            subst ‹q = 0›
            simpa using h2
          · cases h1
      | succ d => simp [nodeAtDepth] at h1
  | binary l r' ihl ihr =>
      cases d with
      | zero =>
          rw [nodeAtDepth_zero_eq] at h1
          split at h1
          · cases h1
            subst ‹q = 0›
            simpa using h2
          · cases h1
      | succ d =>
          rw [(by omega : d + 1 + e = (d + e) + 1)]
          simp only [nodeAtDepth] at h1 ⊢
          rw [div_two_block d e hr, mod_two_block d e hr]
          split at h1
          · rw [if_pos ‹q / 2 ^ d = 0›]
            exact ihl h1
          · split at h1
            · rw [if_neg ‹¬q / 2 ^ d = 0›, if_pos ‹q / 2 ^ d = 1›]
              exact ihr h1
            · cases h1
  | edge length path bottom ih =>
      cases d with
      | zero =>
          rw [nodeAtDepth_zero_eq] at h1
          split at h1
          · cases h1
            subst ‹q = 0›
            simpa using h2
          · cases h1
      | succ d =>
          rw [(by omega : d + 1 + e = (d + e) + 1)]
          simp only [nodeAtDepth] at h1 ⊢
          split at h1
          · obtain ⟨hlen, hpath⟩ := ‹length ≤ d + 1 ∧ q / 2 ^ (d + 1 - length) = path›
            have hsub : d + e + 1 - length = (d + 1 - length) + e := by omega
            rw [if_pos (by
              refine ⟨by omega, ?_⟩
              rw [hsub, div_two_block _ e hr, hpath])]
            rw [hsub, mod_two_block _ e hr]
            exact ih h1
          · cases h1

theorem nodeAtDepth_binary_left {t l r : Tree} {d q : Nat}
    (hn : t.nodeAtDepth d q = some (.binary l r)) :
    t.nodeAtDepth (d + 1) (2 * q) = some l := by
  have hchild : (Tree.binary l r).nodeAtDepth 1 0 = some l := by
    simp [nodeAtDepth]
  have := nodeAtDepth_trans (e := 1) (r := 0) (by omega) hn hchild
  simpa [Nat.pow_one, Nat.mul_comm] using this

theorem nodeAtDepth_binary_right {t l r : Tree} {d q : Nat}
    (hn : t.nodeAtDepth d q = some (.binary l r)) :
    t.nodeAtDepth (d + 1) (2 * q + 1) = some r := by
  have hchild : (Tree.binary l r).nodeAtDepth 1 1 = some r := by
    simp [nodeAtDepth]
  have := nodeAtDepth_trans (e := 1) (r := 1) (by omega) hn hchild
  simpa [Nat.pow_one, Nat.mul_comm] using this

theorem nodeAtDepth_edge_bottom {t bottom : Tree} {length path d q : Nat}
    (hlen : 1 ≤ length) (hp : path < 2 ^ length)
    (hn : t.nodeAtDepth d q = some (.edge length path bottom)) :
    t.nodeAtDepth (d + length) (q * 2 ^ length + path) = some bottom := by
  have hchild : (Tree.edge length path bottom).nodeAtDepth length path = some bottom := by
    cases length with
    | zero => omega
    | succ m =>
        simp only [nodeAtDepth]
        rw [if_pos (by simp)]
        simp [Nat.mod_one]
  exact nodeAtDepth_trans hp hn hchild

/-! ### `at'` through nodes -/

/-- The lookup factors through any node on the key's walk (additive form). -/
theorem at'_factor_add {t s : Tree} {d e k : Nat}
    (hv : t.valid (d + e)) (hn : t.nodeAtDepth d (k / 2 ^ e) = some s) :
    t.at' (d + e) k = s.at' e (k % 2 ^ e) := by
  induction t generalizing d e k s with
  | leaf v =>
      cases d with
      | zero =>
          rw [nodeAtDepth_zero_eq] at hn
          split at hn
          · cases hn
            have hk : k < 2 ^ e := lt_of_div_two_pow_eq_zero ‹k / 2 ^ e = 0›
            rw [Nat.zero_add, Nat.mod_eq_of_lt hk]
          · cases hn
      | succ d => simp [nodeAtDepth] at hn
  | binary l r ihl ihr =>
      cases d with
      | zero =>
          rw [nodeAtDepth_zero_eq] at hn
          split at hn
          · cases hn
            have hk : k < 2 ^ e := lt_of_div_two_pow_eq_zero ‹k / 2 ^ e = 0›
            rw [Nat.zero_add, Nat.mod_eq_of_lt hk]
          · cases hn
      | succ d =>
          obtain ⟨h', heq, hvl, hvr⟩ := hv
          have hh' : h' = d + e := by omega
          subst hh'
          simp only [nodeAtDepth] at hn
          have hsub : d + 1 + e - 1 = d + e := by omega
          have hbit : k / 2 ^ (d + e) = k / 2 ^ e / 2 ^ d := by
            rw [div_two_pow_div, Nat.add_comm e d]
          have hmid : k % 2 ^ (d + e) / 2 ^ e = k / 2 ^ e % 2 ^ d := by
            rw [Nat.add_comm d e, mod_two_pow_div]
          have hmm : k % 2 ^ (d + e) % 2 ^ e = k % 2 ^ e := by
            rw [Nat.add_comm d e, mod_two_pow_mod]
          split at hn
          · have hb : k / 2 ^ (d + e) = 0 := by rw [hbit, ‹k / 2 ^ e / 2 ^ d = 0›]
            have hstep := ihl hvl (d := d) (e := e) (k := k % 2 ^ (d + e))
              (by rw [hmid]; exact hn)
            show Tree.at' _ _ _ = _
            simp only [at', hsub]
            split
            · rw [hstep, hmm]
            · exact absurd hb ‹¬_›
          · split at hn
            · have hb0 : ¬ k / 2 ^ (d + e) = 0 := by rw [hbit]; omega
              have hb1 : k / 2 ^ (d + e) = 1 := by rw [hbit, ‹k / 2 ^ e / 2 ^ d = 1›]
              have hstep := ihr hvr (d := d) (e := e) (k := k % 2 ^ (d + e))
                (by rw [hmid]; exact hn)
              show Tree.at' _ _ _ = _
              simp only [at', hsub]
              split
              · exact absurd ‹_› hb0
              · rw [hstep, hmm]
            · cases hn
  | edge length path bottom ih =>
      cases d with
      | zero =>
          rw [nodeAtDepth_zero_eq] at hn
          split at hn
          · cases hn
            have hk : k < 2 ^ e := lt_of_div_two_pow_eq_zero ‹k / 2 ^ e = 0›
            rw [Nat.zero_add, Nat.mod_eq_of_lt hk]
          · cases hn
      | succ d =>
          obtain ⟨hlen1, hlenh, hplt, hvb, _⟩ := hv
          simp only [nodeAtDepth] at hn
          split at hn
          · obtain ⟨hlen, hpath⟩ := ‹length ≤ d + 1 ∧ k / 2 ^ e / 2 ^ (d + 1 - length) = path›
            have hsub : d + 1 + e - length = (d + 1 - length) + e := by omega
            have htop : k / 2 ^ ((d + 1 - length) + e) = path := by
              rw [Nat.add_comm (d + 1 - length) e, ← div_two_pow_div k e (d + 1 - length), hpath]
            have hvb' : bottom.valid ((d + 1 - length) + e) := by
              rw [(by omega : (d + 1 - length) + e = d + 1 + e - length)]
              exact hvb
            have hmid : k % 2 ^ ((d + 1 - length) + e) / 2 ^ e
                = k / 2 ^ e % 2 ^ (d + 1 - length) := by
              rw [Nat.add_comm _ e, mod_two_pow_div]
            have hmm : k % 2 ^ ((d + 1 - length) + e) % 2 ^ e = k % 2 ^ e := by
              rw [Nat.add_comm _ e, mod_two_pow_mod]
            have hstep := ih hvb' (d := d + 1 - length) (e := e)
              (k := k % 2 ^ ((d + 1 - length) + e)) (by rw [hmid]; exact hn)
            show Tree.at' _ _ _ = _
            simp only [at', hsub]
            split
            · rw [hstep, hmm]
            · exact absurd htop ‹¬_›
          · cases hn

/-- The lookup factors through any node on the key's walk. -/
theorem at'_factor {t s : Tree} {h d k : Nat}
    (hv : t.valid h) (hd : d ≤ h)
    (hn : t.nodeAtDepth d (k / 2 ^ (h - d)) = some s) :
    t.at' h k = s.at' (h - d) (k % 2 ^ (h - d)) := by
  obtain ⟨e, rfl⟩ : ∃ e, h = d + e := ⟨h - d, by omega⟩
  have hde : d + e - d = e := by omega
  rw [hde] at hn ⊢
  exact at'_factor_add hv hn

/-- A leaf node at full depth is the lookup's value. -/
theorem at'_of_leaf_node {t : Tree} {h k v : Nat}
    (hv : t.valid h) (hn : t.nodeAtDepth h k = some (.leaf v)) :
    t.at' h k = v := by
  have := at'_factor hv (Nat.le_refl h)
    (by rw [Nat.sub_self, Nat.pow_zero, Nat.div_one]; exact hn)
  rw [Nat.sub_self, Nat.pow_zero, Nat.mod_one] at this
  simpa [at'] using this

/-- A pinned edge whose compressed bits disagree with the key kills the lookup. -/
theorem at'_diverged {t bottom : Tree} {h d length path k : Nat}
    (hv : t.valid h) (hd : d ≤ h) (hlen : length ≤ h - d)
    (hn : t.nodeAtDepth d (k / 2 ^ (h - d)) = some (.edge length path bottom))
    (hne : k / 2 ^ (h - d - length) % 2 ^ length ≠ path) :
    t.at' h k = 0 := by
  rw [at'_factor hv hd hn]
  have hmid : k % 2 ^ (h - d) / 2 ^ ((h - d) - length)
      = k / 2 ^ (h - d - length) % 2 ^ length := by
    obtain ⟨m, hm⟩ : ∃ m, h - d - length = m := ⟨_, rfl⟩
    rw [hm, (by omega : h - d = m + length), mod_two_pow_div]
  simp only [at']
  rw [hmid]
  split
  · exact absurd ‹_› hne
  · rfl

/-- Out-of-keyspace keys read `0` in every canonical tree. -/
theorem at'_out_of_range {t : Tree} {h k : Nat}
    (hv : t.valid h) (hk : 2 ^ h ≤ k) : t.at' h k = 0 := by
  cases t with
  | leaf v =>
      have h0 : h = 0 := hv.1
      subst h0
      have hkne : k ≠ 0 := by
        have h1 : (2 : Nat) ^ 0 = 1 := Nat.pow_zero 2
        omega
      show Tree.at' _ _ _ = _
      simp only [at']
      split
      · exact absurd ‹_› hkne
      · rfl
  | binary l r =>
      obtain ⟨h', rfl, _, _⟩ := hv
      have h2 : 2 ≤ k / 2 ^ h' := by
        rw [Nat.le_div_iff_mul_le (two_pow_pos h')]
        rw [Nat.pow_succ] at hk
        omega
      have hne0 : ¬ k / 2 ^ h' = 0 := by omega
      have hne1 : ¬ k / 2 ^ h' = 1 := by omega
      show Tree.at' _ _ _ = _
      simp only [at', (by omega : h' + 1 - 1 = h')]
      split
      · exact absurd ‹_› hne0
      · rfl
  | edge length path bottom =>
      obtain ⟨hlen1, hlenh, hplt, _, _⟩ := hv
      have hge : 2 ^ length ≤ k / 2 ^ (h - length) := by
        rw [Nat.le_div_iff_mul_le (two_pow_pos _), ← Nat.pow_add,
          (by omega : length + (h - length) = h)]
        exact hk
      have hne : ¬ k / 2 ^ (h - length) = path := by omega
      show Tree.at' _ _ _ = _
      simp only [at']
      split
      · exact absurd ‹_› hne
      · rfl

end Tree

end PatriciaCircuit
