# circuit-gui — design system

The single source of truth for the visualizer's visual language. **Feed this file to
the designer (and reviewer) on every run.** The designer proposes *within* this system
and **appends** a token here (with rationale) when a genuinely new role is needed —
it does not re-derive the palette from scratch or pick colors that collide with
existing roles.

Files: `viewer/app.js` (the `style()` array + `KIND_COLOR`/`KIND_SYMBOL`/`KIND_LABEL`
maps + layout/fit), `viewer/style.css` (`:root` tokens, toolbar, legend, overlays),
`viewer/index.html` (toolbar markup).

---

## 1. Color tokens (semantic roles — do not collide)

### Surfaces / text (`:root` in style.css)
| Token | Value | Role |
|---|---|---|
| `--bg` | `#fbfbfd` | app background |
| `--bg-graph` | `#f7f8fa` | cytoscape canvas |
| `--surface` | `#ffffff` | panels/buttons |
| `--border` / `--border-strong` | `#ebecf0` / `#dcdee5` | hairlines |
| `--text` / `--text-2` / `--text-3` | `#1c1e26` / `#5a6072` / `#9498a6` | primary / secondary / muted |
| `--accent` | `#5e6ad2` | the indigo accent (selection/focus/active) |
| `--accent-weak` | `#eceefb` | active-button fill |

### Reserved meaning colors — NEVER reuse one role's hue for another
| Color | Hex | Role | Channel |
|---|---|---|---|
| **Indigo accent** | `#5e6ad2` | focused group ring/wash/halo; active button | border + overlay |
| **Indigo (pick)** | `#4f5bd5` | a tapped/selected leaf node | border |
| **SIMD teal** | `#0ea5a4` | "this node is an expandable SIMD vector" | **outline** (outside border) |
| **Output orange** | `#d9912f` | "this gate's value is a public output" | border |
| (output legend swatch) | `#f59e0b` | the legend chip for output | — |
| **Guess periwinkle** | fill `#c5cae9`, border `#9aa1c4`, text `#3a3f63` | a prover-provided witness/guess node | fill+border |
| **Const sand** | fill `#eef0f5`, border `#cfc7bd`, text `#7a7468` | a fixed constant chip | fill+border |
| **Eq violet** | `#a78bfa` (edge), `#8a6fd4` (legend) | equality edge (dashed) | edge |

### Gate kind palette (`KIND_COLOR`) — solid node fills, equal-luminance, muted
`add #5e6ad2` · `sub #7c8af0` · `mul #d9912f` · `pmul #c1568f` · `eq #8a6fd4` ·
`blakeg #16a34a` · `xor #d4a72c` · `m2u #0891b2` · `perm #b5546b` · `out #3a3f4d` ·
`input #ffffff` · `witness #ffffff` (overridden by the guess style below). Fallback `#c5c9d3`.

### Group-family palette (`GROUP_PALETTE`, cycled by `familyColor`)
`#6366f1 #0ea5e9 #10b981 #f59e0b #ef4444 #8b5cf6 #ec4899 #14b8a6 #f97316 #84cc16 #0891b2`
— each group family (blake, round, fri_commit, …) gets a stable ring color from this list.

---

## 2. The ring-channel model (the rule that keeps a node readable)

A node can be several things at once (a guess that is also a public output and a SIMD
vector). Each fact gets its **own visual channel** so they never overwrite each other:

- **FILL = node KIND** — gate color / guess periwinkle / input white / const sand.
- **BORDER = STATE** — output orange `#d9912f` (w 2.5), or selection `#4f5bd5` (w 2.5),
  or focused-group `#5e6ad2` (w 3.5); otherwise the kind's own quiet border.
- **OUTLINE = SIMD** — teal `#0ea5a4` ring (w 3, offset 1.5, opacity .9), drawn *outside*
  the border. Only on **expandable** SIMD vectors. A broadcast const (`kbc#…`, one shared
  scalar) is **NOT** SIMD-marked (no teal ring); its hover reads "broadcast constant · all lanes".
- **OVERLAY = focus halo** — `#5e6ad2` opacity .10, padding 6, on the focused group only.

Example: the `lsb` node = periwinkle fill (guess) + orange border (output) + teal outline (SIMD).

---

## 3. Node taxonomy, sizes, labels

| Kind | Shape / size | Fill | Border | Label |
|---|---|---|---|---|
| gate (add/sub/mul/pmul/…) | ellipse 32 | `KIND_COLOR[kind]` | white .85, w 1.5 | `KIND_SYMBOL` glyph, white bold |
| guess (witness) | ellipse 24 | `#c5cae9` | `#9aa1c4` solid 1.5 | source name (e.g. `lsb`) or `guess`, `#3a3f63` 9px bold |
| input | ellipse 15 | `#ffffff` | `#aeb3c0` 1.5 | name **above** the dot |
| const | round-rect, label-width × 18 | `#eef0f5` | `#cfc7bd` 1 | symbolic (`2⁻¹`, `7`, `2^k`); raw QM31 in hover |
| merged SIMD op-block (`simd-…`) | gate style | kind color | — + teal outline | `KIND_SYMBOL`; detail "Simd::Op · N×QM31" |
| merged SIMD value (`simdval-…`) | kind style | input/guess | — + teal outline | source name / `in` / `guess` |
| expanded group box | round-rect 14 | `data(gcolor)` @ `0.07+0.05·depth` | `data(gcolor)` @ .7 | family label, title chip on top |
| collapsed group token | ellipse 48 | `data(gcolor)` @ .12 | `data(gcolor)` 2.5 | `label\n<count>` |

Hover line: "SIMD · len N" for an expandable vector, "broadcast constant · all lanes"
for `kbc#`, else "QM31". Header guard: `KIND_LABEL[kind] || kind || "node"`.

---

## 4. Edges

Base: `curve-style: bezier`, `control-point-step-size: 24` (so parallel same-pair edges
fan out — a gate's 2 inputs + output to the same neighbor stay separable), width
`min(1.6, .6 + .22·count)`, `line-color #aab1c2`, `target-arrow #8e95a8`, `arrow-scale .8`,
`opacity .62`. Count chip ("×4") is quiet bare text (`#aab0c0` 8px, no box, `#f7f8fa` outline).
Eq edge: dashed `#a78bfa`, no arrows. Edges are **unselectable** (no pick/box-select) and
have no click/hover interaction. A binary gate shows **2 slot-distinct inputs + 1 edge per
consumer**, all visually separated; nothing overlaps (lines or labels).

---

## 5. Layout / spread / fit / zoom

- **Layout**: dagre `rankDir TB`, `nodeSep 30`, `rankSep 38`, `edgeSep 10`, `animate false`.
- **Fit** (`fitView`): `cy.fit(null, 60)` then clamp zoom to **[0.5, 3.4]** and **always center**
  (so small graphs fill the canvas and nothing floats off to one side). Fit fires on group
  expand/collapse — but **NOT** on the SIMD/eq toggles (those preserve zoom+pan and re-center
  on the inspected region; see §7).
- **Zoom slider**: fixed log range **[0.35, 8]** (not re-based on fit); slider 0→0.35,
  50→≈1.67 (comfortable), 100→8.
- **Wide-frontier wrap** (`wrapWideRanks`, interim until group-level/lazy render): when ≥80
  un-grouped orphan gates would spray one giant row, reflow each rank into a width-capped grid
  (`XS/YS 40`, `GAP 28`, `MAX_COLS 36` ≈ 1450px), centered on the group/content x. Gated so
  normal graphs are untouched.
- Inputs sit in a row above everything, outputs below — never inside a group box.

---

## 6. Toolbar & legend

- **Toolbar** (top-right): segmented `.btn-group`s — **depth** (`Collapse all` / `Expand all`)
  · **detail** (`Lanes` / `Equalities` — the SIMD / eq toggles, **per focused group**: tap a
  group first; **disabled** when there's nothing to merge) · zoom slider · `⤢` Fit (icon).
  There is **no Pan/Select** button (panning works from anywhere). `button.active` = pressed;
  `button:disabled` = greyed (`opacity .4`).
  *(Pending change: rename `Lanes`→`Simd`, `Equalities`→`Eq`, drop `Expand all`.)*
- **Legend** (`.overlay-right`, scrollable, min 170 / max 230px): "gates in view" (kind +
  `Simd::Op` rows, then guess / public-output / SIMD-vector / eq swatches) and "groups in view"
  (family rings, **capped at 14 + "+N more…"**).
- **Busy overlay** (`#busy`): centered "laying out…" during relayouts > 400 nodes.

---

## 7. Interaction model (UX advice must respect this)

- **Pan**: drag from **anywhere** — background, over a node, or inside a group box.
- **Tap** a leaf node → pin its spotlight (node + touching edges lit, info pinned bottom-left,
  never occluding). Tap a **group** → make it the focused toggle target (indigo halo). Tap a
  node hugging a group-box edge → targets the **node**, not the group. Tap background → clear.
- **Double-tap** a group/collapsed token → expand/collapse (fits after).
- **Hover** → transient spotlight; leaving restores the pinned node.
- **SIMD/eq toggle** → preserves zoom+pan AND re-centers on the node you were inspecting
  (so un-merging doesn't slide your region off-screen); skips the relayout entirely when it
  changes nothing.
- Tooltip is always pinned to the **bottom-left corner** — it must never cover what you're
  inspecting.

---

## 8. Change protocol (for the designer)

1. Read this file first. Stay within the tokens and the ring-channel model.
2. To propose a change, **preview it live** before recommending: inject the proposed styles
   into the running app via `drive.mjs` (`cy.style()` / element-`.style()` overrides, DOM
   tweaks — no file edits), screenshot, critique your own result, iterate. The returned spec
   must be visually validated, not only reasoned.
3. For subjective/taste choices, return **2–3 variants** with preview screenshots so the user
   picks — don't hand back a single take.
4. Tag each recommendation **necessity vs. opinion**.
5. If a new role genuinely needs a new color, pick one distinct from every row in §1 and
   **append it here** with its role + channel.
6. Honor the "already-decided / locked" list given in the run prompt — don't re-litigate.

---

## 9. Product-quality rubric (what "rounded, product-grade" means)

The designer evaluates the app against ALL of these — not just the asked dimension — and
surfaces gaps even when unprompted. The target feel is a polished dev tool (Linear /
Figma / Observable lineage; the codebase already aims "Linear-style: color carries
meaning, nothing shouts"). Score each 1–5 and justify with a screenshot.

1. **Visual hierarchy** — the eye lands on the graph first, then controls; primary vs
   secondary actions are visually ranked; nothing competes with the data.
2. **Typography** — one consistent type scale & weight set; labels aligned; no orphaned
   font sizes; numerals legible at working zoom.
3. **Spacing & rhythm** — consistent paddings/margins/gaps on a regular step (≈4/8px);
   even breathing room; aligned to an invisible grid; no cramped or lopsided clusters.
4. **Color & contrast** — semantic palette used consistently (per §1); text/icon contrast
   meets WCAG AA; the categorical gate palette is color-blind-distinguishable; no muddy
   overlaps.
5. **Interactive states** — every control has hover / active / disabled and a visible
   **keyboard focus** ring; nodes/groups have hover / selected / focused; no dead-looking
   or no-feedback controls.
6. **Motion** — subtle, fast (~120–180ms) transitions on hover, toggles, panel show, and
   zoom/fit; nothing janky or abrupt; respects `prefers-reduced-motion`.
7. **Feedback** — every action visibly acknowledges (busy overlay for slow ops, pressed
   state for toggles, a settle on fit). No silent multi-second freezes.
8. **Empty / edge / loading states** — sensible look for: nothing loaded, a huge graph,
   a graph with no SIMD/eq, mid-relayout. No raw/blank/again-off-screen states.
9. **Affordance & discoverability** — controls look clickable; non-obvious gestures are
   hinted (double-click to expand, tap a group to focus, drag anywhere to pan); a
   first-run/onboarding cue or a short legend of gestures.
10. **Consistency** — corner radii, shadows, border widths/colors, icon vs text, and
    button shapes are uniform across toolbar / legend / tooltip / minimap / nodes.
11. **Density & responsiveness** — legible and uncramped across window sizes; panels
    (legend/toolbar/minimap) don't overlap the graph or each other; graceful at small widths.
12. **Accessibility** — keyboard operability, `aria-label`/roles on controls, focus-visible,
    color is never the *only* signal (shape/label backs it up).
13. **Cohesion / identity** — a single coherent look; the `stwo-circuits` wordmark, panel
    chrome, and graph share one visual language; feels designed, not assembled.

## 10. Designer modes

- **Spec mode** (default for a specific ask): answer the question with a paste-ready spec,
  visually validated per §8. Keep scope tight.
- **Audit mode** (proactive — invoke when the user wants the overall look-and-feel pushed
  toward product-grade, or at milestones before a release): sweep the WHOLE app across the
  §9 rubric, score each dimension, and return a **prioritized backlog** of concrete,
  paste-ready improvements (highest user-visible polish first), each tagged
  necessity-vs-opinion and effort (S/M/L). Don't stop at the obvious — name the small
  details (a 1px misalignment, an inconsistent radius, a missing hover) that separate
  "works" from "feels finished." Record the backlog here under "§11 design backlog" so it
  compounds across runs and we burn it down over time, re-auditing as items land.

## 11. Design backlog (audit findings — burn down over time)
_(empty — populated by the first audit-mode run; each item: dimension · what · why · effort · status)_
