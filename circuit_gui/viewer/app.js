"use strict";

// Muted, roughly equal-luminance categorical palette per gate kind
// (Linear-style: color carries meaning, nothing shouts).
const KIND_COLOR = {
  add: "#5e6ad2", sub: "#7c8af0", mul: "#d9912f", pmul: "#c1568f",
  eq: "#8a6fd4", blakeg: "#16a34a", xor: "#d4a72c", m2u: "#0891b2",
  perm: "#b5546b", out: "#3a3f4d", input: "#ffffff",
  witness: "#ffffff", const: "#eef0f5",
};
const KIND_SYMBOL = {
  add: "+", sub: "−", mul: "×", pmul: "⊙", eq: "=", blakeg: "G",
  xor: "⊕", m2u: "m→u", perm: "⇄", out: "out", input: "in",
  witness: "w", const: "k",
};
const KIND_LABEL = {
  add: "Add", sub: "Sub", mul: "Mul", pmul: "PointwiseMul", eq: "Eq",
  blakeg: "BlakeG", xor: "TripleXor", m2u: "M31ToU32", perm: "Permutation",
  out: "Output", input: "input", witness: "guess", const: "constant",
};

// Distinct colors for group families (blake / block / round / M31ToU32 / ...).
const GROUP_PALETTE = [
  "#6366f1", "#0ea5e9", "#10b981", "#f59e0b", "#ef4444",
  "#8b5cf6", "#ec4899", "#14b8a6", "#f97316", "#84cc16", "#0891b2",
];
function groupFamily(label) {
  return String(label).split("#")[0];
}
function familyColor(label) {
  const f = groupFamily(label);
  let h = 0;
  for (let i = 0; i < f.length; i++) h = (h * 31 + f.charCodeAt(i)) >>> 0;
  return GROUP_PALETTE[h % GROUP_PALETTE.length];
}
// Darken (amt<0) or lighten (amt>0) a #rrggbb color; returns an rgb() string.
function shade(hex, amt) {
  const s = String(hex).replace("#", "");
  const n = s.length === 3 ? s.split("").map((x) => x + x).join("") : s;
  const ch = (i) => {
    const v = parseInt(n.slice(i, i + 2), 16);
    return Math.round(amt < 0 ? v * (1 + amt) : v + (255 - v) * amt);
  };
  return `rgb(${ch(0)},${ch(2)},${ch(4)})`;
}

// Absolute zoom limits for cytoscape (generous, so fit never clamps).
const ABS_MIN_ZOOM = 0.02, ABS_MAX_ZOOM = 8;
// Slider mapping bounds: left end = "fit / whole graph", right end = 8x in.
// Anchored to the fitted zoom on load so the slider always starts on the left.
let zMin = 0.35, zMax = 8; // FIXED zoom-slider range; mid-slider ≈ 1.7× (comfortable working zoom)

let cy = null;
// Compatibility shim for the headless driver's injected helpers, which call
// ec.isExpandable / ec.isCollapsible. In the frontier model "expandable" means a
// collapsed token that has children; "collapsible" means an expanded group box.
const ec = {
  isExpandable: (n) =>
    !!n.data("collapsed") &&
    (((childGroups[n.id()] || []).length + (childNodes[n.id()] || []).length) > 0),
  isCollapsible: (n) => !!n.data("isGroup") && !n.data("collapsed"),
};
let selectMode = false;
let pinnedId = null; // id of a node clicked to pin its highlight (so its edges stay visible)
let syncingZoom = false;
let maxDepth = 1; // group nesting depth of the loaded circuit

// ---- merge pipeline state ----
// The raw, untransformed graph as loaded from data.js. The "effective" graph fed
// to buildModel is derived from this via applyMerges() and depends on the two
// toggles below (both default OFF = merged).
let rawGraph = null;
// Per-group merge state. Each entry is an EXPLICIT override for that group id;
// groups with no entry inherit from their nearest ancestor that has one, falling
// back to `rootMerge` (the whole-graph default). false = merged (the default).
let groupMerge = {}; // gid -> { simd?:bool, eq?:bool }  (sparse; only set keys)
let rootMerge = { simd: false, eq: false }; // applies where no group override exists
// The currently focused/selected group (a group id), or null = whole graph/root.
// The global SIMD/eq buttons act on this group's state (or rootMerge when null).
let focusedGroup = null;

// Parent-of map for groups, rebuilt from the RAW graph so merge decisions can be
// made before buildModel runs (which builds parentOf over the EFFECTIVE graph).
let rawGroupParent = {}; // gid -> parent gid or null

// Resolve a group id's effective merge state for `kind` ("simd"/"eq"): walk up
// the group ancestry to the nearest group with an explicit override; else the
// root default. A null gid (ungrouped element) uses the root default.
function mergeStateFor(gid, kind) {
  let g = gid;
  while (g != null) {
    const m = groupMerge[g];
    if (m && m[kind] !== undefined) return m[kind];
    g = rawGroupParent[g] != null ? rawGroupParent[g] : null;
  }
  return !!rootMerge[kind];
}

// ---- frontier model (plain JS; the full graph never enters cytoscape) ----
let G = null; // the loaded graph
let groupById = {}; // gid -> group record
let nodeById = {}; // nid -> node record
let parentOf = {}; // element id (group or node) -> parent group id or null
let childGroups = {}; // gid -> [child group ids]
let childNodes = {}; // gid -> [child node ids]
let expanded = new Set(); // set of expanded group ids (virtual root always expanded)
// Cached element positions by id, so expand/collapse keeps the layout stable.
let posCache = {};

// Union-find over a fixed id space (used to merge eq-equivalent nodes).
function makeUF() {
  const parent = new Map();
  const find = (x) => {
    if (!parent.has(x)) { parent.set(x, x); return x; }
    let r = x;
    while (parent.get(r) !== r) r = parent.get(r);
    // path-compress
    let c = x;
    while (parent.get(c) !== r) { const n = parent.get(c); parent.set(c, r); c = n; }
    return r;
  };
  const union = (a, b) => { const ra = find(a), rb = find(b); if (ra !== rb) parent.set(ra, rb); };
  return { find, union };
}

// Re-point every edge through a node-id remap (oldId -> newId), drop self-edges,
// then bundle parallel edges of the same (source,target,rel) into one carrying a
// summed `count` and concatenated `vars`. Pure: returns a fresh edge array.
function remapAndBundleEdges(edges, map) {
  const id = (x) => (map.get(x) !== undefined ? map.get(x) : x);
  const agg = new Map();
  for (const e of edges) {
    const s = id(e.source), t = id(e.target);
    if (s === t) continue; // self-edge after merge
    const rel = e.rel === "eq" ? "eq" : "wire";
    const key = `${s} ${t} ${rel}`;
    const skey = rel === "eq" ? key : key + " #s" + (e.slot !== undefined ? e.slot : "");
    let m = agg.get(skey);
    if (!m) { m = { id:`e ${key}`, source: s, target: t, rel, count: 0, vars: [], slot: e.slot, constraint: false }; agg.set(skey, m); }
    m.count += e.count || 1;
    if (e.constraint) m.constraint = true; // a constraint eq is never merged
    if (e.vars && m.vars.length < 12) m.vars = m.vars.concat(e.vars);
  }
  return [...agg.values()];
}

// Resolve the group a set of gate ids share (the simd_block's gate_ids all live
// in the same group per the data contract); take the first member's group.
function blockGroup(g, gateIds, nodeIndex) {
  for (const gid of gateIds) {
    const n = nodeIndex.get(gid);
    if (n && n.group) return n.group;
  }
  return null;
}

// SIMD merge, per-group. `expandFor(gid)` decides, per group id, whether SIMD is
// EXPANDED there (keep raw lane gates) or MERGED (collapse into one node each).
// A block/value is merged unless its own group resolves to expanded.
function applySimd(g, expandFor) {
  const blocks = g.simd_blocks || [];
  const values = g.simd_values || [];
  if (blocks.length === 0 && values.length === 0) {
    // Nothing to do: pass nodes/edges through unchanged, but still bundle
    // parallels so the renderer stays light.
    return {
      meta: g.meta, groups: g.groups,
      nodes: g.nodes.slice(),
      edges: remapAndBundleEdges(g.edges, new Map()),
    };
  }
  const nodeIndex = new Map(g.nodes.map((n) => [n.id, n]));
  const map = new Map(); // member gate / lane id -> merged node id
  const blockNodes = [];
  const dropped = new Set();
  for (const b of blocks) {
    const ids = b.gate_ids || [];
    if (!ids.length) continue;
    // Skip (keep raw lane gates) if this block's group is expanded.
    if (expandFor(blockGroup(g, ids, nodeIndex))) continue;
    // Derive the merged node's gate kind from a member gate (most reliable),
    // falling back to "pmul" for simd::mul-style labels.
    let kind = null;
    for (const gid of ids) { const n = nodeIndex.get(gid); if (n) { kind = n.kind; break; } }
    if (!kind) kind = "pmul";
    const sym = (String(b.label).split("::").pop() || "").trim();
    const grp = blockGroup(g, ids, nodeIndex);
    // Op-block length is lanes*4 M31 values (≤bound: each QM31 packs up to 4).
    const len = b.lanes * 4;
    // Preserve a public-output marker if any member lane is an output — so a
    // gate-block whose value is a public output (e.g. extract_bits' MSB, the last
    // `·inv_two` pmul) shows the output ring in MERGED mode too, consistent with
    // the expanded lanes. (Without this, merged mode silently dropped it.)
    // Also carry the MIN output ordinal of the members, so a merged output block
    // sorts into the output row by declaration order (same as its lanes do).
    let isOut = false, outIdx;
    for (const gid of ids) {
      const n = nodeIndex.get(gid);
      if (!n) continue;
      if (n.is_output) isOut = true;
      if (n.out_index != null) outIdx = outIdx == null ? n.out_index : Math.min(outIdx, n.out_index);
    }
    blockNodes.push({
      id: b.id, kind,
      label: KIND_SYMBOL[kind] || sym || b.label,
      detail: `${b.label} · ${b.lanes}×QM31 (≤${len} M31 values)`,
      group: grp, simd: true, len, is_output: isOut, out_index: outIdx,
    });
    for (const gid of ids) { map.set(gid, b.id); dropped.add(gid); }
  }
  // Collapse each input/witness SIMD VALUE vector into ONE merged lane node,
  // mirroring how op-blocks collapse. Inputs keep a null group (top row);
  // witnesses inherit their member lanes' shared group (near their consumers).
  for (const sv of values) {
    const ids = sv.member_ids || [];
    if (!ids.length) continue;
    // The shared group of the member lane nodes (witnesses sit near consumers;
    // top-level inputs have none).
    let grp = null;
    for (const mid of ids) { const n = nodeIndex.get(mid); if (n && n.group) { grp = n.group; break; } }
    // Skip (keep raw lane nodes) if this value vector's group is expanded. Inputs
    // are ungrouped, so they follow the root/whole-graph default.
    if (expandFor(grp)) continue;
    if (sv.kind === "input") grp = null; // inputs always render in the top row
    // Preserve a public-output marker + the MIN output ordinal of the members.
    let isOut = false, outIdx;
    for (const mid of ids) {
      const n = nodeIndex.get(mid);
      if (!n) continue;
      if (n.is_output) isOut = true;
      if (n.out_index != null) outIdx = outIdx == null ? n.out_index : Math.min(outIdx, n.out_index);
    }
    blockNodes.push({
      id: sv.id, kind: sv.kind,
      // A recognized motif role uses its source-level name (e.g. "lsb"); the
      // teal SIMD-vector ring (not a "[]" suffix) marks it as a vector.
      label: sv.name ? sv.name : (sv.kind === "input" ? "in" : "guess"),
      detail: `${sv.kind} Simd · ${sv.len} (len)`,
      group: grp, is_output: isOut, simd: true, len: sv.len, out_index: outIdx,
    });
    for (const mid of ids) { map.set(mid, sv.id); dropped.add(mid); }
  }
  // Broadcast-const nodes (`kbc#…`) are a SINGLE shared scalar fed to every lane
  // — there is no per-lane const vector, so they can never expand. They are NOT
  // tagged `simd` (no teal "expandable SIMD vector" ring); the const chip styling
  // already describes them, and the hover reads "broadcast constant · all lanes".
  const nodes = g.nodes
    .filter((n) => !dropped.has(n.id))
    .concat(blockNodes);
  return {
    meta: g.meta, groups: g.groups,
    nodes,
    edges: remapAndBundleEdges(g.edges, map),
  };
}

// EQ merge, per-group, operating on the POST-SIMD graph. Rule (ii): an eq whose
// endpoints share a group G is MERGED into one node (eq edge dropped) unless G's
// eq state resolves to expanded via `expandFor(G)`. eq edges across DIFFERENT
// groups always stay dashed (never merged), regardless of state.
function applyEq(g, expandFor) {
  const nodeIndex = new Map(g.nodes.map((n) => [n.id, n]));
  const groupOf = (id) => { const n = nodeIndex.get(id); return n ? (n.group || null) : null; };
  const uf = makeUF();
  // Union eq endpoints that are in the same group, unless that group is expanded.
  // A `constraint` eq (one side computed from the other — `x²==x`, `q·b==a`) is
  // NEVER merged: merging would close a cycle into a self-loop and drop the
  // producing gate to an edgeless node. Only INDEPENDENT equal values merge.
  for (const e of g.edges) {
    if (e.rel !== "eq" || e.constraint) continue;
    const gs = groupOf(e.source), gt = groupOf(e.target);
    if (gs !== null && gs === gt && !expandFor(gs)) uf.union(e.source, e.target);
  }
  // Build oldId -> representative map for any node touched by a same-group union.
  const map = new Map();
  for (const n of g.nodes) {
    const r = uf.find(n.id);
    if (r !== n.id) map.set(n.id, r);
  }
  // Surviving nodes are the union-find representatives.
  const nodes = g.nodes.filter((n) => !map.has(n.id));
  // Re-point edges; same-group eq edges become self-edges and are dropped by the
  // bundler. Cross-group eq edges survive as dashed.
  const edges = remapAndBundleEdges(g.edges, map);
  return { meta: g.meta, groups: g.groups, nodes, edges };
}

// Build the effective graph the frontier model renders, from the raw graph and
// the per-group merge state: effective = applyEq( applySimd(raw, simdFor), eqFor ),
// where each predicate resolves a group id to its effective expand state.
function applyMerges() {
  rawGroupParent = {};
  for (const g of rawGraph.groups || []) rawGroupParent[g.id] = g.parent || null;
  const simdFor = (gid) => mergeStateFor(gid, "simd");
  const eqFor = (gid) => mergeStateFor(gid, "eq");
  return applyEq(applySimd(rawGraph, simdFor), eqFor);
}

function buildModel(graph) {
  G = graph;
  groupById = {}; nodeById = {}; parentOf = {}; childGroups = {}; childNodes = {};
  for (const g of graph.groups) {
    groupById[g.id] = g;
    childGroups[g.id] = childGroups[g.id] || [];
    childNodes[g.id] = childNodes[g.id] || [];
  }
  for (const g of graph.groups) {
    parentOf[g.id] = g.parent || null;
    if (g.parent) (childGroups[g.parent] || (childGroups[g.parent] = [])).push(g.id);
  }
  for (const n of graph.nodes) {
    nodeById[n.id] = n;
    parentOf[n.id] = n.group || null;
    if (n.group) (childNodes[n.group] || (childNodes[n.group] = [])).push(n.id);
  }
}

// Walk a leaf node's ancestor-group chain upward; return the SHALLOWEST ancestor
// group that is NOT expanded (the collapsed token standing in for it). If every
// ancestor is expanded (or the node is ungrouped), return the leaf id itself.
function rep(leafId) {
  // Build the ancestor chain (root-most last), then find the shallowest collapsed one.
  let p = parentOf[leafId];
  let token = null;
  while (p) {
    if (!expanded.has(p)) token = p; // shallowest-so-far collapsed ancestor
    p = parentOf[p];
  }
  return token || leafId;
}

// A group/node is rendered iff its parent group is expanded (null parent = root).
function isParentExpanded(id) {
  const p = parentOf[id];
  return p === null || expanded.has(p);
}

// Compute the desired visible cytoscape elements for the current `expanded` set.
function computeVisible() {
  const els = [];
  // Groups: render those whose parent is expanded.
  for (const g of G.groups) {
    if (!isParentExpanded(g.id)) continue;
    const open = expanded.has(g.id);
    const d = {
      id: g.id, label: g.label, depth: g.depth,
      count: g.count || 0, gcolor: familyColor(g.label),
    };
    d.isGroup = true; // flags a group element (expanded box or collapsed token)
    // Parent it to its group box whenever that parent is expanded — for BOTH
    // open boxes and collapsed tokens, so a collapsed child stays inside its
    // expanded parent's container (and its column layout) instead of orphaning.
    if (g.parent && expanded.has(g.parent)) d.parent = g.parent;
    if (!open) d.collapsed = true; // collapsed token leaf (styled as a ring)
    els.push({ data: d });
  }
  // Nodes: render those whose parent group is expanded (or ungrouped).
  for (const n of G.nodes) {
    if (!isParentExpanded(n.id)) continue;
    // Inlined consts (per-use scalars folded onto the gate, no node) get a small,
    // uniform visible cue on the label — a `ₖ` subscript — so you can SEE at a
    // glance which gates fold a constant, without crowding the glyph with the
    // value (some values are long raw QM31 tuples). The value(s) stay in the hover
    // tooltip. Compact short symbolic names (≤4 chars, e.g. `2⁻¹`, `7`) are shown
    // inline since they're informative and tidy; longer ones collapse to `ₖ`.
    const cs = n.consts || [];
    let label = n.label;
    if (cs.length === 1 && cs[0].name.length <= 4) label = `${n.label} ${cs[0].name}`;
    else if (cs.length >= 1) label = `${n.label}ₖ${cs.length > 1 ? cs.length : ""}`;
    const d = {
      id: n.id, label, kind: n.kind, detail: n.detail || "",
      isOutput: !!n.is_output, consts: cs,
      simd: !!n.simd, len: n.len,
    };
    if (n.out_index != null) d.outIndex = n.out_index; // output declaration order
    if (n.group && expanded.has(n.group)) d.parent = n.group;
    if (n.bk) { d.bk = n.bk; d.bcol = n.bcol; d.brow = n.brow; }
    els.push({ data: d });
  }
  // Meta-edges: aggregate gate edges by (rep(source), rep(target)); drop
  // self-loops; sum count; keep eq as a separate dashed meta-edge.
  const agg = new Map(); // key -> {source,target,rel,count,vars}
  for (const e of G.edges) {
    const s = rep(e.source), t = rep(e.target);
    if (s === t) continue; // self-loop inside a collapsed token
    const rel = e.rel === "eq" ? "eq" : "wire";
    const slotKey = (rel === "wire" && !groupById[t] && e.slot !== undefined) ? " #s" + e.slot : "";
    const key = `${s} ${t} ${rel}` + slotKey;
    let m = agg.get(key);
    if (!m) { m = { source: s, target: t, rel, count: 0, vars: [] }; agg.set(key, m); }
    m.count += e.count || 1;
    if (e.vars && m.vars.length < 12) m.vars = m.vars.concat(e.vars);
  }
  for (const [key, m] of agg) {
    els.push({
      data: {
        id: `me ${key}`, source: m.source, target: m.target, rel: m.rel,
        vars: m.vars, count: m.count,
        label: m.rel === "eq" ? "=" : m.count > 1 ? `×${m.count}` : "",
      },
    });
  }
  return els;
}

// Diff the desired visible elements against the live cy: add new ids, remove
// vanished ones. Caches/restores positions so the layout stays stable.
function rebuild(fit = true, onDone) {
  if (!cy) return;
  const desired = computeVisible();
  // Signature captures the parts of an element that, if changed, require it to
  // be torn down and re-added (its id alone staying the same isn't enough): a
  // group flipping collapsed↔expanded, a node re-parenting, an edge's count.
  const sig = (d) => `${d.collapsed ? "c" : "o"}|${d.parent || ""}|${d.count || ""}|${d.label || ""}`;
  const desiredById = new Map(desired.map((el) => [el.data.id, el]));
  // Cache current leaf positions before mutating, so re-adding an element later
  // (collapse → re-expand) can restore its slot instead of re-scattering.
  cy.nodes().forEach((n) => {
    if (!n.isParent()) { const p = n.position(); posCache[n.id()] = { x: p.x, y: p.y }; }
  });
  // Track which leaves already existed (keep their layout); only brand-new
  // leaves need dagre to place them.
  const existing = new Set();
  cy.nodes().forEach((n) => existing.add(n.id()));
  let nRemoved = 0, nAdded = 0;
  cy.batch(() => {
    // Remove anything no longer wanted, or whose role/data signature changed.
    const toRemove = cy.elements().filter((el) => {
      const want = desiredById.get(el.id());
      if (!want) return true;
      return sig(want.data) !== sig(el.data());
    });
    if (toRemove.nonempty()) {
      nRemoved = toRemove.length;
      toRemove.forEach((el) => existing.delete(el.id()));
      // Strip classes + selection BEFORE removing, so cytoscape doesn't try to
      // recompute style hints on an element mid-removal (a real double-click pins/
      // selects a child via the tap handler, then the collapse removes it — that
      // raced into a "reading 'index'" crash in updateStyleHints).
      toRemove.removeClass("hl-node hl-edge faded focused-group");
      toRemove.unselect();
      cy.remove(toRemove);
    }
    const present = new Set();
    cy.elements().forEach((el) => present.add(el.id()));
    const toAdd = desired.filter((el) => !present.has(el.data.id));
    nAdded = toAdd.length;
    if (toAdd.length) cy.add(toAdd);
  });
  // Nothing added/removed (e.g. a no-op SIMD/eq toggle, or a toggle whose blocks
  // are all inside collapsed groups) → skip the expensive dagre relayout that
  // otherwise froze big graphs for seconds with no visible change.
  if (nRemoved === 0 && nAdded === 0) {
    if (focusedGroup && !groupById[focusedGroup]) focusedGroup = null;
    cy.nodes().removeClass("focused-group");
    if (focusedGroup) cy.getElementById(focusedGroup).addClass("focused-group");
    updateLegend();
    showBusy(false);
    if (onDone) onDone();
    return;
  }
  // Restore cached positions for leaves we have seen before (stable layout).
  cy.batch(() => {
    cy.nodes().forEach((n) => {
      if (n.isParent()) return;
      const p = posCache[n.id()];
      if (p && existing.has(n.id())) n.position(p);
    });
  });
  // Re-apply the focus highlight (a rebuild may have re-created the focused
  // group element, dropping its class). Drop focus if the group no longer exists.
  if (focusedGroup && !groupById[focusedGroup]) focusedGroup = null;
  cy.nodes().removeClass("focused-group");
  if (focusedGroup) cy.getElementById(focusedGroup).addClass("focused-group");
  runLayout(fit, onDone);
}

function style() {
  return [
    // All gate nodes are small perfect circles (fixed equal width/height).
    {
      selector: "node",
      style: {
        "background-color": (n) => KIND_COLOR[n.data("kind")] || "#c5c9d3",
        label: "data(label)", "text-wrap": "none", "text-valign": "center",
        "text-halign": "center", "font-size": 9, "font-weight": "bold",
        color: "#fff", "text-outline-width": 1,
        "text-outline-color": (n) => shade(KIND_COLOR[n.data("kind")] || "#c5c9d3", -0.2),
        shape: "ellipse", width: 32, height: 32,
        "border-width": 1.5, "border-color": "rgba(255,255,255,.85)",
      },
    },
    // Inputs are hollow source dots with the variable label above them (so the
    // outgoing wires below don't cross the text).
    { selector: 'node[kind = "input"]',
      style: {
        "background-color": "#ffffff", "border-color": "#aeb3c0", "border-width": 1.5,
        width: 15, height: 15, color: "#5a6072", "text-outline-width": 0,
        label: "data(label)", "text-valign": "top", "text-margin-y": -3,
        "font-size": 9, "font-weight": "normal",
        "text-background-color": "#f7f8fa", "text-background-opacity": 0.85, "text-background-padding": 2,
      },
    },
    // Witness: a prover-provided value — hollow, dashed, muted ring (hint style),
    // sized between an input dot and a gate so it reads as "not a computed gate".
    { selector: 'node[kind = "witness"]',
      style: {
        // A calm, solid light-lavender token (not a washed-out white hole): reads
        // as "prover-provided", legible on its own, and sits quietly UNDER the
        // teal SIMD / orange output / indigo selection rings rather than fighting
        // them. Solid border (the old dashed grey was invisible and ragged).
        "background-color": "#c5cae9", "background-opacity": 1,
        "border-color": "#9aa1c4", "border-width": 1.5, "border-style": "solid",
        shape: "ellipse",
        width: 24, height: 24, color: "#3a3f63",
        "text-outline-width": 1.5, "text-outline-color": "#c5cae9",
        label: "data(label)", "text-valign": "center", "text-halign": "center",
        "font-size": 9, "font-weight": "bold",
      },
    },
    // Const: a small lighter labeled chip showing the constant's name.
    { selector: 'node[kind = "const"]',
      style: {
        // Warmer-neutral border/text so a fixed CONSTANT reads as materially
        // different from the cool-lavender prover GUESS token.
        "background-color": "#eef0f5", "border-color": "#cfc7bd", "border-width": 1,
        shape: "round-rectangle", "corner-radius": 6,
        width: "label", height: 18, padding: "4px",
        color: "#7a7468", "text-outline-width": 0,
        label: "data(label)", "text-valign": "center", "text-halign": "center",
        "font-size": 8.5, "font-weight": "normal",
      },
    },
    { selector: "node[?isOutput]",
      style: { "border-width": 2.5, "border-color": "#d9912f" } },
    // SIMD vector: one drawn node standing for N packed lanes. A teal OUTLINE
    // (drawn OUTSIDE the border) marks it as a vector — distinct from the indigo
    // selection/hover border and the orange input/output ring, and composes with
    // both without overwriting them. Replaces the old "[]" label suffix.
    { selector: "node[?simd]",
      style: {
        "outline-width": 3, "outline-color": "#0ea5a4",
        "outline-offset": 1.5, "outline-opacity": 0.9,
      } },
    // Expanded group: a softly tinted container with a title chip on top.
    {
      selector: "node[?isGroup]",
      style: {
        "background-color": "data(gcolor)",
        "background-opacity": (n) => 0.07 + 0.05 * (n.data("depth") || 0),
        shape: "round-rectangle", "corner-radius": 14,
        "border-width": 1, "border-style": "solid",
        "border-color": "data(gcolor)", "border-opacity": 0.7,
        label: "data(label)", "text-valign": "top", "text-halign": "center",
        "font-size": 10.5, "font-weight": "bold", color: "data(gcolor)",
        "text-margin-y": -9, "text-background-color": "#ffffff",
        "text-background-opacity": 0.92, "text-background-padding": 3,
        "text-background-shape": "round-rectangle",
        padding: "16px", width: "label", height: "label",
      },
    },
    // Collapsed group: a container token — white fill, family-colored ring,
    // dark label + gate count (distinct from solid-colored gate circles).
    {
      selector: "node[?collapsed]",
      style: {
        "background-color": "data(gcolor)", "background-opacity": 0.12,
        shape: "ellipse", "border-width": 2.5, "border-color": "data(gcolor)",
        "border-style": "solid",
        label: (n) => (n.data("count") ? `${n.data("label")}\n${n.data("count")}` : n.data("label")),
        "text-wrap": "wrap", "text-valign": "center", "text-halign": "center",
        "text-background-opacity": 0, "text-margin-y": 0,
        color: (n) => shade(n.data("gcolor") || "#5e6ad2", -0.35),
        "text-outline-width": 2, "text-outline-color": "#ffffff",
        "font-size": 9, "font-weight": "bold", width: 48, height: 48,
      },
    },
    { selector: "node:selected",
      style: { "border-width": 2.5, "border-color": "#4f5bd5", "border-opacity": 1 } }, // crisp momentary pick
    // Focused group (the target of the SIMD/eq toggle buttons): a subtle, slightly
    // stronger ring + faint fill so it reads as "selected" without shouting.
    { selector: "node.focused-group",
      style: {
        // The "armed" toggle target: a thick indigo ring + indigo wash + soft
        // halo, PINNED to indigo (not the group family's gcolor) so "focused"
        // always reads the same. The halo uses `overlay` (teal SIMD uses
        // `outline`), so they compose without collision.
        "border-width": 3.5, "border-color": "#5e6ad2", "border-opacity": 1,
        "background-color": "#5e6ad2", "background-opacity": 0.18,
        "overlay-color": "#5e6ad2", "overlay-opacity": 0.10, "overlay-padding": 6,
      } },
    {
      selector: "edge",
      style: {
        // Edges are PURELY VISUAL — they receive no pointer events at all, so an
        // edge can never be tapped/selected/marked, and a click on an edge passes
        // straight through to the node or background behind it (no dead clicks
        // where a long edge crosses a node). Highlighting still works (it's driven
        // by node.connectedEdges(), not edge events).
        events: "no",
        // Bezier (not straight) so multiple edges between the SAME node pair fan
        // out into distinct curves instead of stacking on one line — e.g. a gate's
        // slot-0 input, slot-1 input, and its output to that same neighbor all stay
        // visibly separate (the square gate's lsb²==lsb has all 3 between one pair).
        "curve-style": "bezier",
        "control-point-step-size": 24,
        width: (e) => Math.min(1.6, 0.6 + 0.22 * (e.data("count") || 1)),
        // Stronger lines/arrows so the graph reads as connected (not floating
        // circles) at fit-zoom.
        "line-color": "#aab1c2", "target-arrow-color": "#8e95a8",
        "target-arrow-shape": "triangle", "arrow-scale": 0.8,
        "line-cap": "round", opacity: 0.62,
        // Count chips (e.g. "×4") as quiet bare text riding the edge — no white
        // box (the box made them read as nodes and out-shout the real nodes).
        label: "data(label)", "font-size": 8, color: "#aab0c0",
        "text-outline-width": 2, "text-outline-color": "#f7f8fa",
        "text-background-opacity": 0,
      },
    },
    {
      selector: 'edge[rel = "eq"]',
      style: {
        "line-style": "dashed", "line-color": "#a78bfa",
        "target-arrow-shape": "none", "source-arrow-shape": "none",
        "font-weight": "bold", color: "#7c3aed",
      },
    },
    // Hover highlight: fade everything, then spotlight the hovered node + its wires.
    { selector: ".faded", style: { opacity: 0.22 } },
    { selector: ".hl-node", style: { opacity: 1, "border-width": 2.5, "border-color": "#5e6ad2", "z-index": 20 } },
    { selector: ".hl-edge", style: { opacity: 1, "line-color": "#5e6ad2", "target-arrow-color": "#5e6ad2", "z-index": 999, width: 2.2 } },
  ];
}

// Interim mitigation for huge UN-GROUPED frontiers (e.g. the fibonacci verifier's
// hundreds of top-level witness/const/pmul gates): dagre lays each rank in one row
// that runs far off-screen. Reflow the orphan middle-gate ranks into width-capped
// grids (stacked top-to-bottom) so they stay on-screen. Gated to large orphan
// counts so normal graphs (demo/blake) are untouched. The real fix is group-level
// / lazy rendering (deferred-B); this just keeps the overview usable.
function wrapWideRanks() {
  if (!cy) return;
  const orphans = cy.nodes().filter((n) =>
    !n.isParent() && !n.data("isGroup") && n.isOrphan() &&
    n.data("kind") !== "input" && n.data("kind") !== "out" && !n.data("isOutput"));
  if (orphans.length < 80) return; // only kicks in for very large frontiers
  const ranks = new Map();
  orphans.forEach((n) => {
    const r = Math.round(n.position("y") / 20) * 20;
    if (!ranks.has(r)) ranks.set(r, []);
    ranks.get(r).push(n);
  });
  const XS = 40, YS = 40, GAP = 28, MAX_COLS = 36; // ≤ ~1450px wide per grid (denser, squarer)
  // Center the grids on the main graph content (groups + the node they feed), not
  // the orphans' own scattered average, so the block frames cleanly with the rest.
  const anchors = cy.nodes().filter((n) => n.data("isGroup"));
  let cx = 0;
  if (anchors.length) { anchors.forEach((g) => { cx += g.position("x"); }); cx /= anchors.length; }
  else { orphans.forEach((n) => { cx += n.position("x"); }); cx /= orphans.length; }
  let y = Math.min(...ranks.keys());
  cy.batch(() => {
    [...ranks.keys()].sort((a, b) => a - b).forEach((rk) => {
      const nodes = ranks.get(rk).sort((a, b) => a.position("x") - b.position("x"));
      const n = nodes.length;
      const cols = Math.min(n, MAX_COLS);
      const span = (cols - 1) * XS, x0 = cx - span / 2;
      nodes.forEach((nd, i) => {
        const c = i % cols, r = Math.floor(i / cols);
        nd.position({ x: cols === 1 ? cx : x0 + XS * c, y: y + r * YS });
      });
      y += Math.ceil(n / cols) * YS + GAP;
    });
  });
}

// CANONICAL-LAYOUT-STAMP — step 1 (mechanism proof on the simplest motif: a SIMD
// lane-block). A SIMD block is N same-kind gates, lane-aligned, no inter-lane
// wiring; its canonical shape is a clean horizontal row. We STAMP that shape onto
// every visible (un-merged) block: order lanes by their intrinsic LANE INDEX
// (position in the block's `gate_ids` — deterministic, identical across
// instances, independent of var numbering), and place them at fixed spacing
// centered on the block's dagre centroid. So every same-size block becomes the
// same row (a translate), proving capture(trivial row)+key-by-index+stamp. Later
// steps compose these rigid rows into higher motifs (extract_bits, …).
const SIMD_LANE_SPACING = 64;
function stampSimdBlocks() {
  if (!cy || !rawGraph) return;
  for (const b of rawGraph.simd_blocks || []) {
    const ids = b.gate_ids || [];
    if (ids.length < 2) continue;
    // Visible lane gates (skip if merged into one node or only partly present).
    const lanes = ids.map((id) => cy.getElementById(id)).filter((n) => n && n.nonempty() && !n.isParent());
    if (lanes.length < 2) continue;
    let cx = 0, cy0 = 0;
    lanes.forEach((n) => { const p = n.position(); cx += p.x; cy0 += p.y; });
    cx /= lanes.length; cy0 /= lanes.length;
    const x0 = cx - ((lanes.length - 1) * SIMD_LANE_SPACING) / 2;
    cy.batch(() => lanes.forEach((n) => {
      const idx = ids.indexOf(n.id()); // intrinsic lane index → stable offset
      n.position({ x: x0 + idx * SIMD_LANE_SPACING, y: cy0 });
    }));
  }
}

// The canonical per-motif LADDER ENGINE. Registry of motif group labels whose
// interiors are laid out by `stampMotifLadders` (rather than left to dagre). This
// is the seam where new motifs opt into the canonical layout — add a label here
// (eventually derived from the catalog). The ENGINE is motif-agnostic; it depends
// only on a group's own subgraph, so every motif/instance lays out deterministically
// and identically.
const LADDER_MOTIFS = new Set(["extract_bits"]);

// CANONICAL-LAYOUT-STAMP engine — lay each registered MOTIF group's members out as
// a deterministic DATAFLOW LADDER: rank every member by its topological depth in the
// group's INTERNAL wire DAG (longest path from an internal source — intrinsic to the
// subgraph, so two instances rank identically → identical layout, no var-id
// dependence), then stack rank-rows top→bottom. Within a row, a SIMD block's lanes
// stay contiguous and in lane order (the step-1 row), loose gates follow. Adjacent
// rungs feed each other → short edges (objective §1). Overrides the generic
// stampSimdBlocks for these members; non-motif blocks keep their step-1 rows.
const LADDER_ROW_H = 92;
function stampMotifLadders() {
  if (!cy || !rawGraph) return;
  // gate id -> [blockId, laneIndex] for ordering lanes within a rank row.
  const laneOf = new Map();
  for (const b of rawGraph.simd_blocks || []) {
    (b.gate_ids || []).forEach((id, i) => laneOf.set(id, [b.id, i]));
  }
  cy.nodes().filter((n) => n.isParent() && LADDER_MOTIFS.has(n.data("label"))).forEach((g) => {
    const members = g.children().filter((n) => !n.isParent());
    if (members.length < 4) return;
    const ids = new Set(members.map((n) => n.id()));
    const preds = new Map();
    members.forEach((n) => preds.set(n.id(), []));
    cy.edges('[rel = "wire"]').forEach((e) => {
      const s = e.source().id(), t = e.target().id();
      if (ids.has(s) && ids.has(t)) preds.get(t).push(s);
    });
    // Longest-path rank (memoized; cycle-guarded — eq constraints can loop).
    const rank = new Map();
    const compute = (id, seen) => {
      if (rank.has(id)) return rank.get(id);
      if (seen.has(id)) return 0;
      seen.add(id);
      const ps = preds.get(id) || [];
      const r = ps.length ? 1 + Math.max(...ps.map((p) => compute(p, seen))) : 0;
      seen.delete(id);
      rank.set(id, r);
      return r;
    };
    members.forEach((n) => compute(n.id(), new Set()));
    // Pull SOURCE nodes (no internal predecessor — the lsb guesses, const) DOWN to
    // just above their earliest consumer instead of stranding them all at rank 0
    // with long edges (objective §1: short edges). Sources feeding nothing internal
    // stay at 0. High-fan-out sources (a broadcast const) still land one above
    // their nearest consumer — fine, their other edges are exempt from the cost.
    const succ = new Map();
    members.forEach((n) => succ.set(n.id(), []));
    cy.edges('[rel = "wire"]').forEach((e) => {
      const s = e.source().id(), t = e.target().id();
      if (ids.has(s) && ids.has(t)) succ.get(s).push(t);
    });
    members.forEach((n) => {
      const id = n.id();
      if ((preds.get(id) || []).length === 0) {
        const outs = succ.get(id) || [];
        if (outs.length) rank.set(id, Math.max(0, Math.min(...outs.map((c) => rank.get(c))) - 1));
      }
    });
    // Keep OUTPUTS at the bottom (user rule): force every output member into a
    // dedicated bottom row below the deepest rank, regardless of its dataflow
    // depth (extract_bits' output bits are also consumed mid-chain — their wires
    // run up from this row, the accepted tradeoff).
    const isOut = (n) => n.data("isOutput") || n.data("kind") === "out";
    const outIds = new Set(members.filter(isOut).map((n) => n.id()));
    // TAIL = the pure side-check downstream of outputs only: the lsb² assert-square
    // (consumes its lsb, an output) and its eq (consumes lsb + the square). Found
    // by fixpoint: a non-output node all of whose internal preds are outputs/tail.
    const tail = new Set();
    let grew = true;
    while (grew) {
      grew = false;
      members.forEach((n) => {
        const id = n.id();
        if (outIds.has(id) || tail.has(id)) return;
        const ps = preds.get(id) || [];
        // Must be downstream of outputs only AND be a SINK (no live wire-successor
        // — feeds nothing but other tail nodes / its eq). The latter is essential:
        // the FIRST chain `sub`'s only INTERNAL pred is its lsb (an output; its real
        // data input is a top-level input outside the group), but it FEEDS the chain
        // `mul`, so it is NOT tail and must stay near its inputs — not pushed to the
        // bottom (which both overlapped a square and stranded it from its inputs).
        const ss = succ.get(id) || [];
        const isSink = ss.every((s) => tail.has(s));
        if (ps.length && isSink && ps.every((p) => outIds.has(p) || tail.has(p))) { tail.add(id); grew = true; }
      });
    }
    // The reduction chain occupies ranks 0..chainMax. Place the TAIL just above the
    // output row, and OUTPUTS as the ABSOLUTE bottom row (user rule). So the square
    // sits one row above its lsb (short edge DOWN to the output), and outputs stay
    // bottom-most. (lsb also feeds the chain `sub` far above — that long upward edge
    // is the accepted dual-role / outputs-at-bottom tradeoff.)
    let chainMax = 0;
    members.forEach((n) => { if (!isOut(n) && !tail.has(n.id())) chainMax = Math.max(chainMax, rank.get(n.id())); });
    members.forEach((n) => {
      const id = n.id();
      if (outIds.has(id)) rank.set(id, chainMax + 2);
      else if (tail.has(id)) rank.set(id, chainMax + 1);
    });
    let cx = 0, minY = Infinity;
    members.forEach((n) => { cx += n.position("x"); minY = Math.min(minY, n.position("y")); });
    cx /= members.length;
    // High-fan-out constants (e.g. inv_two, feeding every reduction mul) go to a
    // LEFT side column rather than inline in a dataflow row — they can't be near
    // all consumers anyway (DESIGN.md §9; their edges are already cost-exempt §1).
    const HIGH_FANOUT = 3;
    const sideConsts = members.filter((n) => n.data("kind") === "const" && (succ.get(n.id()) || []).length >= HIGH_FANOUT);
    const sideSet = new Set(sideConsts.map((n) => n.id()));
    const byRank = new Map();
    members.forEach((n) => {
      if (sideSet.has(n.id())) return; // parked on the left, not in a rank row
      const r = rank.get(n.id());
      if (!byRank.has(r)) byRank.set(r, []);
      byRank.get(r).push(n);
    });
    // Stable within-row order. OUTPUTS sort by their DECLARATION ordinal (the
    // order the circuit `output()`-ed them — lsb..msb, not node id), so the bottom
    // row reads meaningfully. Everything else: block lanes (by blockId, lane index)
    // first, then loose gates by kind+id — deterministic, id-stable across instances.
    const sortKey = (n) => {
      const oi = n.data("outIndex");
      if (oi != null) return `0|${String(oi).padStart(6, "0")}`;
      const l = laneOf.get(n.id());
      return l ? `1|${l[0]}|${String(l[1]).padStart(4, "0")}` : `2|${n.data("kind")}|${n.id()}`;
    };
    let maxHalf = 0;
    cy.batch(() => {
      [...byRank.keys()].sort((a, b) => a - b).forEach((r) => {
        const row = byRank.get(r).sort((a, b) => (sortKey(a) < sortKey(b) ? -1 : 1));
        const half = ((row.length - 1) * SIMD_LANE_SPACING) / 2;
        if (half > maxHalf) maxHalf = half;
        const x0 = cx - half;
        row.forEach((nd, j) => nd.position({ x: x0 + j * SIMD_LANE_SPACING, y: minY + r * LADDER_ROW_H }));
      });
      // Park the side consts in a left column, each centered on the mean rank of
      // its consumers (so its fan of edges spreads rightward across the body).
      const leftX = cx - maxHalf - SIMD_LANE_SPACING * 1.8;
      sideConsts.forEach((n, i) => {
        const cs = (succ.get(n.id()) || []).map((c) => rank.get(c));
        const meanR = cs.length ? cs.reduce((a, b) => a + b, 0) / cs.length : 0;
        n.position({ x: leftX - i * SIMD_LANE_SPACING, y: minY + meanR * LADDER_ROW_H });
      });
      // eq-connected nodes as CLOSE as possible (DESIGN §12 #10): column-align each
      // eq pair so the dashed link is a short straight edge — e.g. the lsb² square
      // sits directly above its lsb. Move the non-output endpoint onto the other's
      // x (outputs keep their bottom-row x); if neither is an output, meet halfway.
      cy.edges('[rel = "eq"]').forEach((e) => {
        const s = e.source(), t = e.target();
        if (!ids.has(s.id()) || !ids.has(t.id()) || sideSet.has(s.id()) || sideSet.has(t.id())) return;
        const sOut = outIds.has(s.id()), tOut = outIds.has(t.id());
        if (sOut && !tOut) t.position({ x: s.position("x") });
        else if (tOut && !sOut) s.position({ x: t.position("x") });
        else { const mx = (s.position("x") + t.position("x")) / 2; s.position({ x: mx }); t.position({ x: mx }); }
      });
    });
  });
}

// EXPERIMENT (opt-in via window.__layoutMode === "edgemin"; default is the ladder
// above, kept as the FALLBACK). The generalized GENERIC layout: NO ladder, NO
// tail-detection / source-pull / eq-align. Pin the boundary (outputs bottom in
// declaration order, high-fan-out const left); SEED interior
// from dagre's positions (non-degenerate, deterministic); then alternate
// (a) edge-length ATTRACTION — each interior node eased toward the average of its
// wire-neighbors — with (b) a SEPARATION pass that pushes any too-close pair apart,
// so single-neighbor nodes (e.g. an assert-square whose only neighbor is its lsb)
// don't collapse onto a pinned node and the chain doesn't crowd. Deterministic
// (no randomness). Array-based for speed. Lets us compare a principled
// edge-length-min against the hand-tuned ladder.
const EDGEMIN_MIN_DIST = 64;     // hard min-distance gap between any two nodes (tunable)
const EDGEMIN_MAX_NODES = 120;   // skip edge-min on groups larger than this (the O(n³)/iter swaps would be slow); they keep their dagre layout (tunable)
const EDGEMIN_LEN_BUDGET = 0.05; // PER-secondary: each phase-2 objective may grow length up to +5% above its start (tunable)
function stampMotifEdgeMin() {
  if (!cy || !rawGraph) return;
  cy.nodes().filter((n) => n.isParent()).forEach((g) => {
    const members = g.children().filter((n) => !n.isParent());
    if (members.length < 4 || members.length > EDGEMIN_MAX_NODES) return;
    const ids = new Set(members.map((n) => n.id()));
    const idx = new Map(members.map((n, i) => [n.id(), i]));
    const adjI = members.map(() => []);
    const dir = []; // directed member→member wire edges [producerIdx, consumerIdx]
    const edgesU = []; // unique member→member edges [i,j] (wire+eq) for uniform + clearance terms
    cy.edges('[rel = "wire"]').forEach((e) => {
      const s = e.source().id(), t = e.target().id();
      if (ids.has(s) && ids.has(t)) { adjI[idx.get(s)].push(idx.get(t)); adjI[idx.get(t)].push(idx.get(s)); dir.push([idx.get(s), idx.get(t)]); edgesU.push([idx.get(s), idx.get(t)]); }
    });
    // eq-edges count toward the OBJECTIVE (attraction) but NOT toward no-up (eq is
    // symmetric, not producer→consumer).
    cy.edges('[rel = "eq"]').forEach((e) => {
      const s = e.source().id(), t = e.target().id();
      if (ids.has(s) && ids.has(t)) { adjI[idx.get(s)].push(idx.get(t)); adjI[idx.get(t)].push(idx.get(s)); edgesU.push([idx.get(s), idx.get(t)]); }
    });
    const isOut = (n) => n.data("isOutput") || n.data("kind") === "out";
    const isConst = (n) => n.data("kind") === "const" && adjI[idx.get(n.id())].length >= 3;
    const consts = members.filter(isConst);
    const outputs = members.filter(isOut).sort((a, b) => (a.data("outIndex") ?? 0) - (b.data("outIndex") ?? 0));
    const outSet = new Set(outputs.map((n) => idx.get(n.id())));
    const constIdxSet = new Set(consts.map((n) => idx.get(n.id())));
    // Index lists for the relational boundary constraints (#2, #4). NO node is pinned
    // to a hand-picked coordinate — the boundary is enforced purely by ordering.
    const all = members.map((_, i) => i);
    const nonOutIdx = all.filter((i) => !outSet.has(i));
    const nonConstIdx = all.filter((i) => !constIdxSet.has(i));
    // "the body" outputs sit below = every non-output, non-const node (consts are placed
    // after the solve). Pendant assert-squares ARE included: a SYMMETRIC below-by-D
    // inequality keeps them ABOVE the row, so they can't drag it (no pendant special-case).
    const solveBody = nonOutIdx.filter((i) => !constIdxSet.has(i));
    const D = EDGEMIN_MIN_DIST;
    const upExempt = (s) => outSet.has(s) || constIdxSet.has(s);
    // Per-node no-up adjacency: for each non-exempt producer edge p→c, c sits ≥ D below p.
    const prodOf = members.map(() => []), consOf = members.map(() => []);
    for (const [p, c] of dir) { if (upExempt(p)) continue; prodOf[c].push(p); consOf[p].push(c); }
    // SEED = OPTION A — a feasible LAYERED layout built from the graph's OWN structure, so it
    // works for ANY DAG (not just recognized motifs): rank each node by its longest path over
    // NON-EXEMPT wire edges (so a dual-role output doesn't drag its consumers below it), force
    // outputs to a dedicated bottom rank, then place y = rank·ROW, x = (index-in-rank)·COL with
    // ROW,COL ≥ D. Feasible by construction — no-up & below-body from the ordering, min-distance
    // from the spacing — so the line-search below only maintains it; the seed needs no repair.
    const ROW = Math.round(D * 1.3), COL = Math.round(D * 1.3);
    const rank = new Array(members.length).fill(-1);
    const rankOf = (i, stk) => {
      if (rank[i] >= 0) return rank[i];
      if (stk.has(i)) return 0;               // cycle guard (wire graph is a DAG; this is just safety)
      stk.add(i);
      let r = 0;
      for (const p of prodOf[i]) { const rp = rankOf(p, stk) + 1; if (rp > r) r = rp; }
      stk.delete(i); rank[i] = r; return r;
    };
    for (let i = 0; i < members.length; i++) if (!constIdxSet.has(i)) rankOf(i, new Set());
    let maxBodyRank = 0;
    for (const i of solveBody) if (rank[i] > maxBodyRank) maxBodyRank = rank[i];
    for (const o of outSet) rank[o] = maxBodyRank + 1; // outputs → a dedicated bottom rank
    const px = new Array(members.length).fill(0), py = new Array(members.length).fill(0);
    const perRank = new Map();
    for (let i = 0; i < members.length; i++) {
      if (constIdxSet.has(i)) continue;
      const r = rank[i], n = perRank.get(r) || 0; perRank.set(r, n + 1);
      px[i] = n * COL; py[i] = r * ROW;
    }
    // CONSTRAINED OPTIMIZER — FEASIBLE COORDINATE DESCENT. Each non-const node, in turn,
    // eases toward its neighbor centroid (shorten edges = the objective) and is PLACED only
    // where it stays feasible. So the layout is always legal and there is NO move-then-correct
    // overshoot (which caused the earlier artifacts). NO global projection step.
    // HARD constraints, each a per-node feasibility check on the move:
    //   #1 min-distance ≥ D from every other node (resolved by X-separation; the chain is
    //      already Y-separated by no-up, so X handles same-row clusters),
    //   #2 outputs below the body: every output ≥ D below every (non-const) body node — a
    //      per-node y-bound (output's floor / body's ceiling),
    //   #3 no upward edges: consumer ≥ producer + D, EXEMPT edges whose producer is an output
    //      or a high-fan-out const (dual-role / broadcast).
    // Two deterministic POST-steps (cosmetic placements, NOT during-solve constraints):
    //   - outputs CO-LINEAR: after the solve, move every output DOWN to the lowest output's y
    //     (down only adds below-body slack → always feasible) and re-space in x → a straight
    //     strict-bottom row,
    //   - high-fan-out consts placed at (leftmost − D), centered on their consumers (excluded
    //     from the solve — broadcast fans, not locality).
    // RE-CENTERED each pass (no absolute anchor → kill the translation-neutral drift).
    // OBJECTIVE: minimize total edge length (the centroid pull). [Secondaries — axis-exact,
    // clearance — to be re-added on this feasible-move model.]
    // Feasible y-band for node i: #3 no-up (below its producers, above its consumers) + #2
    // outputs-below-body (an output ≥ D below every body node; a body node ≥ D above every output).
    const yRange = (i) => {
      let lo = -Infinity, hi = Infinity;
      for (const p of prodOf[i]) if (py[p] + D > lo) lo = py[p] + D;
      for (const c of consOf[i]) if (py[c] - D < hi) hi = py[c] - D;
      if (outSet.has(i)) { for (const n of solveBody) if (py[n] + D > lo) lo = py[n] + D; }
      else { for (const o of outSet) if (py[o] - D < hi) hi = py[o] - D; }
      return [lo, hi];
    };
    // Edge length of node n placed at (x,y), summed over its (non-const) neighbours, with one
    // neighbour optionally skipped (a swap partner — its edge is invariant under the swap).
    const elen = (n, x, y, skip) => {
      let s = 0;
      for (const j of adjI[n]) if (j !== skip && !constIdxSet.has(j)) s += Math.hypot(x - px[j], y - py[j]);
      return s;
    };
    // NOTE: no separate feasibility-init — we seed from the LADDER, which is already feasible
    // (clean rows, min-distance = D, correct ordering). The per-node line-search below only
    // maintains feasibility; it never needs to repair the seed.
    // Seed centroid (non-const) — re-center target (kills the translation-neutral drift).
    let mx0 = 0, my0 = 0; for (const i of nonConstIdx) { mx0 += px[i]; my0 += py[i]; }
    mx0 /= nonConstIdx.length; my0 /= nonConstIdx.length;
    for (let it = 0; it < 160; it++) {
      for (let i = 0; i < members.length; i++) {
        if (constIdxSet.has(i)) continue; // consts placed after the solve
        // SWAP move — exchange FULL positions with the node j that most reduces their combined
        // edge length, if feasible. A swap only relabels two occupied positions, so min-distance
        // is automatically preserved; the only check is that each node's y is valid in the
        // other's band (no-up / below-body) — yRange catches no-up- or output-violating swaps.
        // Accepted only if strictly shorter ⇒ monotone, no oscillation. This is what escapes
        // ordering local minima: a node can't translate past a min-dist neighbour, but it can
        // swap with it.
        {
          const [loI, hiI] = yRange(i);
          let bestGain = 0.5, bestJ = -1;
          for (let j = 0; j < members.length; j++) {
            if (j === i || constIdxSet.has(j)) continue;
            if (py[j] < loI || py[j] > hiI) continue;       // i can't sit at j's y
            const [loJ, hiJ] = yRange(j);
            if (py[i] < loJ || py[i] > hiJ) continue;        // j can't sit at i's y
            const gain = (elen(i, px[i], py[i], j) + elen(j, px[j], py[j], i))
                       - (elen(i, px[j], py[j], j) + elen(j, px[i], py[i], i));
            if (gain > bestGain) { bestGain = gain; bestJ = j; }
          }
          if (bestJ >= 0) {
            const tx = px[i], ty = py[i];
            px[i] = px[bestJ]; py[i] = py[bestJ]; px[bestJ] = tx; py[bestJ] = ty;
          }
        }
        // (objective) LINE-SEARCH toward the neighbor centroid: step as far along the ray to
        // the centroid as stays feasible — capped by the y-band (#2/#3) and by every other
        // node's min-distance disk (#1). Feasibility holds by construction (never step past a
        // boundary); a fully boxed node simply doesn't move (f→0).
        let sx = 0, sy = 0, k = 0;
        for (const j of adjI[i]) { if (constIdxSet.has(j)) continue; sx += px[j]; sy += py[j]; k++; }
        if (k) {
          const dx = sx / k - px[i], dy = sy / k - py[i]; // full vector to the centroid
          let f = 1; // 1 = reach the centroid; capped below
          const [lo, hi] = yRange(i);
          if (dy > 1e-9) f = Math.min(f, (hi - py[i]) / dy);       // moving down → cap at band floor
          else if (dy < -1e-9) f = Math.min(f, (lo - py[i]) / dy); // moving up → cap at band ceiling
          const a = dx * dx + dy * dy;
          if (a > 1e-12) for (let j = 0; j < members.length; j++) {
            if (j === i || constIdxSet.has(j)) continue;
            const ex = px[i] - px[j], ey = py[i] - py[j];
            const b = 2 * (ex * dx + ey * dy), c = ex * ex + ey * ey - D * D;
            if (b >= 0) continue;                     // moving away from j → can't enter its disk
            const disc = b * b - 4 * a * c;
            if (disc <= 0) continue;                  // ray never reaches distance D of j
            // first f where it would enter j's disk; clamp ≥0 so a pair sitting AT D (the ladder
            // packs at exactly D) and moving inward is capped to 0 instead of slipping through.
            const t1 = (-b - Math.sqrt(disc)) / (2 * a), cap = t1 < 0 ? 0 : t1;
            if (cap < f) f = cap;
          }
          if (f > 0) { px[i] += f * dx; py[i] += f * dy; }
        }
      }
      // RE-CENTER (non-const): shift so the body's centroid returns to its seed value.
      let mx = 0, my = 0; for (const i of nonConstIdx) { mx += px[i]; my += py[i]; }
      mx /= nonConstIdx.length; my /= nonConstIdx.length;
      const shx = mx0 - mx, shy = my0 - my;
      for (const i of nonConstIdx) { px[i] += shx; py[i] += shy; }
    }
    // POST-STEP — outputs co-linear at the strict bottom: move every output DOWN to the
    // lowest output's y (down only adds below-body slack → always feasible; pendant squares
    // stay above), then re-space them in x to keep ≥ D, preserving left-right order.
    if (outSet.size) {
      let lowest = -Infinity; for (const o of outSet) if (py[o] > lowest) lowest = py[o];
      for (const o of outSet) py[o] = lowest;
      const ord = [...outSet].sort((a, b) => px[a] - px[b]);
      for (let pass = 0; pass < 3; pass++) for (let q = 1; q < ord.length; q++)
        if (px[ord[q]] - px[ord[q - 1]] < D) px[ord[q]] = px[ord[q - 1]] + D;
    }
    // PLACE high-fan-out consts AFTER the solve: one min-distance LEFT of the leftmost body
    // node, vertically centered on their own consumers. Multiple consts step further left.
    if (consts.length) {
      let minX = Infinity; for (const i of nonConstIdx) if (px[i] < minX) minX = px[i];
      consts.forEach((c, s) => {
        const ci = idx.get(c.id());
        let sy = 0, k = 0; for (const j of adjI[ci]) { sy += py[j]; k++; }
        px[ci] = minX - D - s * D;
        py[ci] = k ? sy / k : my0;
      });
    }
    cy.batch(() => members.forEach((n, i) => n.position({ x: px[i], y: py[i] })));
  });
}

// Place every group's output-flagged nodes in a row at the BOTTOM of the group,
// below its other gates (inputs sit at the top — see alignInputsOutputs). Applies
// to ALL circuits. For an entangled group like extract_bits (whose output bits
// are also consumed mid-chain) this stretches those wires up from the bottom row;
// that tradeoff is accepted — outputs always read at the bottom.
function moveOutputsToGroupBottom() {
  if (!cy) return;
  cy.nodes().filter((n) => n.isParent()).forEach((g) => {
    const kids = g.children().filter((n) => !n.isParent());
    const isOut = (n) => n.data("isOutput") || n.data("kind") === "out";
    const outs = kids.filter(isOut);
    const rest = kids.filter((n) => !isOut(n));
    if (!outs.length || !rest.length) return;
    let maxY = -Infinity, minX = Infinity, maxX = -Infinity;
    rest.forEach((n) => { maxY = Math.max(maxY, n.position("y")); });
    kids.forEach((n) => { minX = Math.min(minX, n.position("x")); maxX = Math.max(maxX, n.position("x")); });
    if (!isFinite(maxY)) return;
    const sorted = outs.sort((a, b) => a.position("x") - b.position("x"));
    const n = sorted.length, y = maxY + 52;
    const span = Math.max(maxX - minX, (n - 1) * 46), x0 = (minX + maxX) / 2 - span / 2;
    cy.batch(() => sorted.forEach((nd, i) =>
      nd.position({ x: n === 1 ? (minX + maxX) / 2 : x0 + (span * i) / (n - 1), y })));
  });
}

function runLayout(fit = true, onDone) {
  // Align inputs/outputs and fit only after the layout has committed positions
  // (the layout applies positions asynchronously).
  const layout = cy.layout({
    name: "dagre", rankDir: "TB", nodeSep: 30, rankSep: 38,
    edgeSep: 10, animate: false, fit: false, padding: 40,
  });
  layout.one("layoutstop", () => {
    wrapWideRanks();
    stampSimdBlocks();
    moveOutputsToGroupBottom();
    // Default = ladder (the motif fallback). Edge-min is self-contained — it builds its own
    // feasible layered seed (option A) from each group's structure, for ANY group (not just
    // registered motifs), so it doesn't need the ladder.
    if (window.__layoutMode === "edgemin") stampMotifEdgeMin();
    else stampMotifLadders();
    alignInputsOutputs();
    arrangeBlakeBlocks();
    arrangeBlakeReduction();
    if (fit) fitView();
    updateLegend();
    showBusy(false);
    if (onDone) onDone();
  });
  // dagre runs synchronously (animate:false) and can block for seconds on large
  // frontiers. For big graphs, show a "laying out…" indicator and yield a frame
  // so it actually paints before the blocking run.
  if (cy.nodes().length > 400) {
    showBusy(true);
    requestAnimationFrame(() => requestAnimationFrame(() => layout.run()));
  } else {
    layout.run();
  }
}
// A lightweight "laying out…" overlay shown during expensive relayouts.
function showBusy(on) {
  let el = document.getElementById("busy");
  if (!el) {
    if (!on) return;
    el = document.createElement("div");
    el.id = "busy";
    el.textContent = "laying out…";
    document.body.appendChild(el);
  }
  el.style.display = on ? "block" : "none";
}

// Place each Blake's reduction/finalizer gates (tagged with bk/bcol/brow by the
// exporter) on a fixed mirrored grid below the blocks, so the out0 (units 0–3)
// and out1 (units 4–7) sub-circuits are laid out identically and symmetrically.
function arrangeBlakeReduction() {
  const byBlake = {};
  cy.nodes().forEach((n) => {
    const bk = n.data("bk");
    if (bk) (byBlake[bk] || (byBlake[bk] = [])).push(n);
  });
  const DX = 46, DY = 44;
  for (const bk in byBlake) {
    const grp = cy.getElementById(bk);
    if (grp.empty()) continue;
    const centerX = grp.position("x");
    let baseY = -Infinity;
    grp.descendants().forEach((d) => {
      if (!d.isParent() && !d.data("bk")) baseY = Math.max(baseY, d.position("y"));
    });
    if (!isFinite(baseY)) baseY = grp.position("y");
    baseY += DY * 1.5;
    cy.batch(() => {
      byBlake[bk].forEach((n) => {
        n.position({ x: centerX + (n.data("bcol") - 3.5) * DX, y: baseY + n.data("brow") * DY });
      });
    });
  }
}

function familyIndex(label, prefix) {
  const m = new RegExp(`${prefix}#(\\d+)`).exec(label || "");
  return m ? +m[1] : 0;
}

// Lay each "blake block" out identically: its "blake round" children in a single
// straight vertical column, ordered round#0..n, and all blocks aligned to the
// same top y so the blocks look the same and sit at the same height. Only
// applies when the rounds are collapsed (not themselves expanded into gates).
function arrangeBlakeBlocks() {
  const blocks = cy.nodes().filter(
    (n) => n.data("isGroup") && groupFamily(n.data("label")) === "blake block",
  );
  if (blocks.length === 0) return;
  const ROUND_DY = 64;
  let commonTop = null;
  blocks.forEach((block) => {
    const rounds = block.children().filter((c) => groupFamily(c.data("label")) === "blake round");
    if (rounds.length === 0) return;
    const sorted = rounds.sort((a, b) => familyIndex(a.data("label"), "round") - familyIndex(b.data("label"), "round"));
    const cx = block.position("x");
    let top = block.position("y") - ((sorted.length - 1) * ROUND_DY) / 2;
    if (commonTop === null) commonTop = top; else top = commonTop;
    cy.batch(() => {
      sorted.forEach((r, i) => {
        const tx = cx, ty = top + i * ROUND_DY;
        if (r.isParent()) {
          // Expanded round: translate its whole subtree so the box keeps its
          // slot in the column (a parent's own position can't be set directly).
          const cur = r.position();
          const dx = tx - cur.x, dy = ty - cur.y;
          r.descendants().forEach((d) => {
            const p = d.position();
            d.position({ x: p.x + dx, y: p.y + dy });
          });
        } else {
          r.position({ x: tx, y: ty });
        }
      });
    });
  });
}

// Cap how far "fit" may zoom IN, so nodes stay a reasonable size even when only
// a few are on screen (we just leave whitespace instead of magnifying them).
const MAX_FIT_ZOOM = 3.4; // small graphs fill more of the canvas (less whitespace)
const MIN_FIT_ZOOM = 0.5;  // keep nodes/edges legible at fit; pan/minimap for the rest
function fitView() {
  if (!cy) return;
  cy.fit(null, 60);
  const z = cy.zoom();
  if (z > MAX_FIT_ZOOM) { cy.zoom(MAX_FIT_ZOOM); cy.center(); }
  else if (z < MIN_FIT_ZOOM) { cy.zoom(MIN_FIT_ZOOM); cy.center(); }
  else cy.center(); // always center, so an in-range fit doesn't sit off to one side
}

// Snap all top-level input nodes to a common top row and all top-level output
// nodes to a common bottom row (nodes inside groups are left in place so group
// boxes stay intact).
function alignInputsOutputs() {
  const leaves = cy.nodes().filter((n) => !n.isParent());
  if (!leaves.length) return;
  const isIn = (n) => n.data("kind") === "input";
  const isOut = (n) => n.data("kind") === "out" || n.data("isOutput");
  // Reference rows from the "middle" gates (everything that isn't an in/out).
  const middle = leaves.filter((n) => !isIn(n) && !isOut(n));
  const ref = middle.length ? middle : leaves;
  let topRef = Infinity, botRef = -Infinity;
  ref.forEach((n) => {
    const y = n.position("y");
    if (y < topRef) topRef = y;
    if (y > botRef) botRef = y;
  });
  // Also clear every group BOX: fold each visible parent's bounding-box top/
  // bottom into the refs, so inputs/outputs never land INSIDE an expanded group
  // box (their middle-gate children sit inside, but the box's padded rectangle
  // extends past them).
  cy.nodes().filter((n) => n.isParent()).forEach((g) => {
    const bb = g.boundingBox();
    if (bb.y1 < topRef) topRef = bb.y1;
    if (bb.y2 > botRef) botRef = bb.y2;
  });
  const gap = 70;
  // Horizontal span to spread the rows across, from the middle gates' extent.
  let minX = Infinity, maxX = -Infinity;
  ref.forEach((n) => {
    const x = n.position("x");
    if (x < minX) minX = x;
    if (x > maxX) maxX = x;
  });
  if (!isFinite(minX)) { minX = 0; maxX = 0; }
  const spread = (nodes, y) => {
    const sorted = nodes.sort((a, b) => a.position("x") - b.position("x"));
    const n = sorted.length;
    if (n === 0) return;
    const span = Math.max(maxX - minX, (n - 1) * 60);
    const x0 = (minX + maxX) / 2 - span / 2;
    sorted.forEach((node, i) => {
      node.position({ x: n === 1 ? (minX + maxX) / 2 : x0 + (span * i) / (n - 1), y });
    });
  };
  const ins = cy.nodes().filter((n) => !n.isParent() && n.isOrphan() && isIn(n));
  const outs = cy.nodes().filter((n) => !n.isParent() && n.isOrphan() && isOut(n));
  cy.batch(() => {
    spread(ins, topRef - gap); // inputs in one row above everything
    spread(outs, botRef + gap); // outputs in one row below everything
  });
}

// Set the frontier to show everything up to a nesting depth: expand all groups
// with depth < level (level 0 = collapse all; level maxDepth = expand all).
function setLevel(level) {
  if (!G) return;
  expanded = new Set(
    G.groups.filter((g) => g.depth < level).map((g) => g.id),
  );
  rebuild(true);
}

// Set (or clear, with gid=null) the focused group: the target the global
// SIMD/eq toggle buttons act on. Applies a subtle highlight to the focused group
// box/token and reflects its current merge state in the toggle buttons.
function setFocus(gid) {
  focusedGroup = gid && groupById[gid] ? gid : null;
  if (cy) {
    cy.nodes().removeClass("focused-group");
    if (focusedGroup) cy.getElementById(focusedGroup).addClass("focused-group");
  }
  if (window.syncToggleButtons) window.syncToggleButtons();
}

// ---- zoom slider (log scale) ----
// Left end (0) = current "fit" zoom (whole graph); right end (100) = zoomed in.
function zoomToSlider(z) {
  const v = (100 * Math.log(z / zMin)) / Math.log(zMax / zMin);
  return Math.round(Math.min(100, Math.max(0, v)));
}
function sliderToZoom(v) {
  return zMin * Math.pow(zMax / zMin, v / 100);
}
function syncZoomSlider() {
  if (!cy) return;
  syncingZoom = true;
  document.getElementById("zoom").value = zoomToSlider(cy.zoom());
  syncingZoom = false;
}
// Anchor the slider's left end to the current (fitted) zoom, so it starts on
// the left and slides right to zoom in.
function anchorZoom() {
  // The slider uses a FIXED zoom range (zMin..zMax, see top) rather than
  // re-basing on each fit — so its feel is consistent across graphs and the
  // comfortable ~1.5-2× working zoom sits around mid-travel (not jammed into the
  // first fifth). Just reflect the current zoom in the slider position.
  if (!cy) return;
  syncZoomSlider();
}

function setMode(select) {
  // The Pan/Select toggle was removed (panning works from anywhere; nothing
  // consumed a box-selection). This is now only called once with select=false to
  // keep panning enabled; the old #mode button no longer exists.
  selectMode = select;
  if (cy) {
    cy.userPanningEnabled(!select);
    cy.boxSelectionEnabled(false);
  }
}

function esc(s) {
  return String(s).replace(/[&<>]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;" }[c]));
}

function showTip(n) {
  const tip = document.getElementById("tooltip");
  let html;
  if (n.data("isGroup")) {
    html =
      `<b>${esc(n.data("label"))}</b> <span class="muted">group</span>` +
      `<br><span class="muted">${n.data("count") || 0} gates · double-click to open</span>`;
  } else {
    const kind = n.data("kind");
    const consts = n.data("consts") || [];
    html =
      `<b>${esc(KIND_LABEL[kind] || kind || "node")}</b> <span class="muted">${esc(n.id())}</span>`;
    if (n.data("isOutput")) html += ` <span class="muted">· public output</span>`;
    if (n.data("detail")) html += `<br><code>${esc(n.data("detail"))}</code>`;
    // SIMD-ness line: merged op-blocks (`simd-…`), input/witness Simd vectors
    // (`simdval-…`), and broadcast consts (`kbc#…`) carry simd:true; show their
    // length when known. Everything else is a single scalar value.
    const id = String(n.id());
    const isBroadcastConst = id.startsWith("kbc#");
    const isSimd = !isBroadcastConst &&
      (!!n.data("simd") || id.startsWith("simd-") || id.startsWith("simdval-"));
    const len = n.data("len");
    html += isBroadcastConst
      ? `<br><span class="muted">broadcast constant · all lanes</span>`
      : isSimd
      ? `<br><span class="muted">SIMD · len ${len != null ? len : "?"}</span>`
      : `<br><span class="muted">QM31</span>`;
    if (consts.length) {
      html += `<br><span class="muted">constants:</span> ` +
        consts.map((c) => `${esc(c.name)} = [${c.var}]`).join(", ");
    }
  }
  tip.innerHTML = html;
  // Pin the info to a fixed bottom-left corner rather than next to the node, so
  // it never sits on top of the highlighted node or its (radiating) edges — the
  // thing you clicked to inspect stays fully visible.
  tip.style.display = "block";
  const th = tip.offsetHeight;
  tip.style.left = "12px";
  tip.style.top = `${Math.max(6, cy.height() - th - 12)}px`;
}
function hideTip() {
  const tip = document.getElementById("tooltip");
  if (tip) tip.style.display = "none";
}

// Accent a hovered gate and its wires (no dimming of the rest). Groups get only
// a tooltip — hovering them doesn't spotlight anything.
function highlight(n) {
  if (n.data("isGroup")) return;
  cy.batch(() => {
    n.addClass("hl-node");
    n.connectedEdges().addClass("hl-edge");
  });
}
function clearHighlight() {
  cy.batch(() => cy.elements().removeClass("faded hl-node hl-edge"));
}

// Legend lists only the gate kinds currently visible in the view (collapsed
// groups hide their gates, so those kinds drop out until expanded).
function updateLegend() {
  const present = new Set();
  const simdPresent = new Set(); // gate kinds appearing as MERGED Simd nodes
  const fam = new Map(); // group family (e.g. "round") -> occurrences in view
  let hasOutput = false, hasEq = false, hasSimd = false, hasGuess = false;
  if (cy) {
    cy.nodes().forEach((n) => {
      if (n.data("isGroup")) {
        const f = String(n.data("label")).split("#")[0]; // round#3 -> round
        fam.set(f, (fam.get(f) || 0) + 1);
        return;
      }
      const k = n.data("kind");
      if (k) { if (n.data("simd")) simdPresent.add(k); else present.add(k); }
      if (n.data("simd")) hasSimd = true;
      if (k === "witness") hasGuess = true;
      if (n.data("isOutput")) hasOutput = true;
    });
    hasEq = cy.edges('[rel = "eq"]').length > 0;
  }
  const KIND_ORDER = ["add", "sub", "mul", "pmul", "blakeg", "xor", "m2u", "perm", "out", "input"];
  const order = KIND_ORDER.filter((k) => present.has(k));
  const simdOrder = KIND_ORDER.filter((k) => simdPresent.has(k));
  const row = (k, name) =>
    `<div class="legend-row"><span class="swatch" style="background:${KIND_COLOR[k]}">${esc(KIND_SYMBOL[k])}</span>${name}</div>`;
  const kindRows = order.map((k) => row(k, KIND_LABEL[k])).join("");
  // Merged Simd nodes get their own "Simd::<op>" rows so a vector op is never
  // shown as a bare scalar gate (Simd::Sub vs Sub).
  const simdRows = simdOrder.map((k) => row(k, `Simd::${KIND_LABEL[k]}`)).join("");
  let html = `<h4>gates in view</h4>${(kindRows + simdRows) || '<div class="legend-row hint">— none —</div>'}`;
  if (hasGuess) html += `<div class="legend-row"><span class="swatch" style="background:#c5cae9;border:1.5px solid #9aa1c4"></span>guess (witness)</div>`;
  if (hasOutput) html += `<div class="legend-row"><span class="swatch" style="border:3px solid #f59e0b"></span>public output</div>`;
  if (hasSimd) html += `<div class="legend-row"><span class="swatch" style="border:3px solid #0ea5a4;background:transparent"></span>SIMD vector</div>`;
  if (hasEq) html += `<div class="legend-row"><span class="swatch swatch-eq"></span>= equality edge</div>`;
  if (fam.size) {
    // Cap the families list so a big circuit (e.g. an expanded `verify` with many
    // sub-group families) doesn't turn the legend into a tall wall over the graph.
    // Most-frequent first, then a "+N more" line. (The panel also scrolls.)
    const LEGEND_FAM_CAP = 14;
    const sorted = [...fam.keys()].sort((a, b) => (fam.get(b) - fam.get(a)) || a.localeCompare(b));
    const shown = sorted.slice(0, LEGEND_FAM_CAP);
    html += `<h4>groups in view</h4>` +
      shown
        .map((f) => {
          const n = fam.get(f);
          return `<div class="legend-row"><span class="swatch swatch-group" style="color:${familyColor(f)}"></span>${esc(f)}${n > 1 ? ` <span class="muted">×${n}</span>` : ""}</div>`;
        })
        .join("");
    if (sorted.length > LEGEND_FAM_CAP) {
      html += `<div class="legend-row hint">+${sorted.length - LEGEND_FAM_CAP} more…</div>`;
    }
  }
  document.getElementById("legend").innerHTML = html;
}

function load(graph) {
  if (cy) cy.destroy();
  rawGraph = graph;
  // Reset per-group merge state and focus for the freshly loaded circuit.
  groupMerge = {};
  rootMerge = { simd: false, eq: false };
  focusedGroup = null;
  document.getElementById("meta").textContent =
    `${graph.meta.n_gates} gates · ${graph.meta.n_vars} vars · ${graph.meta.n_groups} groups`;

  cy = cytoscape({
    container: document.getElementById("cy"),
    elements: [],
    style: style(),
    wheelSensitivity: 0.25,
    minZoom: ABS_MIN_ZOOM,
    maxZoom: ABS_MAX_ZOOM,
    boxSelectionEnabled: false,
    // Individual nodes/groups are NOT draggable — only the whole graph pans.
    autoungrabify: true,
    // No native selection at all — the app spotlights via the hl-node /
    // focused-group CLASSES, never cytoscape ":selected". Native selection only
    // caused crashes (selecting a node on tap, then a collapse removing it mid
    // double-click churned style hints → "reading 'index'").
    autounselectify: true,
  });

  // Edges are never selectable/markable (no pick, no box-select) — only nodes.
  // Registered before the first rebuild so initial edges are covered too.
  cy.on("add", "edge", (evt) => evt.target.unselectify());

  // Run the merge pipeline (SIMD + eq) to get the effective graph, then build the
  // plain-JS frontier model and render only the depth-0 frontier.
  buildModel(applyMerges());
  expanded = new Set();
  posCache = {};
  maxDepth = Math.max(1, graph.meta.max_depth);
  rebuild(true); // overview: only the depth-0 frontier is instantiated

  setMode(selectMode);
  try { cy.navigator({}); } catch (e) { /* minimap optional */ }

  cy.on("zoom", syncZoomSlider);
  cy.ready(() => { fitView(); anchorZoom(); });
  // Fit again once the container has settled its size.
  setTimeout(() => { if (cy) { fitView(); anchorZoom(); } }, 60);

  // Double-click semantics:
  //  - a collapsed group token  -> expand it
  //  - an expanded group box     -> collapse it
  //  - ANY node inside an expanded group (a child gate/const/witness, or a
  //    collapsed child token) -> collapse the NEAREST expanded ancestor group,
  //    so you can fold a region back up by double-clicking anywhere inside it.
  cy.on("dbltap", "node", (evt) => {
    const id = evt.target.id();
    // Decide the action, then run it DEFERRED (after this gesture's event dispatch
    // finishes). Rebuilding the graph synchronously inside the dbltap handler
    // mutates elements while cytoscape is still processing the real double-click,
    // crashing its style-hint pool ("reading 'index'"). A real double-click hit
    // this; emit('dbltap') didn't (no real event in flight).
    let action = null;
    if (groupById[id] && expanded.has(id)) {
      action = () => { expanded.delete(id); rebuild(true); };
    } else if (groupById[id] && !expanded.has(id)) {
      // A leaf group (no children) can't be opened further.
      if ((childGroups[id] || []).length === 0 && (childNodes[id] || []).length === 0) return;
      action = () => { expanded.add(id); rebuild(true); };
    } else {
      // Non-group node: collapse the nearest EXPANDED ancestor group.
      let p = parentOf[id], target = null;
      while (p) { if (expanded.has(p)) { target = p; break; } p = parentOf[p]; }
      if (target) action = () => { expanded.delete(target); rebuild(true); };
    }
    // Clear any pin/spotlight/tooltip the double-click's taps left on a node the
    // collapse is about to remove, so no stale info lingers.
    if (action) setTimeout(() => { pinnedId = null; clearHighlight(); hideTip(); action(); }, 0);
  });

  // Restore the pinned node's spotlight + info (a no-op if nothing is pinned or
  // the pinned node no longer exists after a rebuild).
  function restorePin() {
    if (!pinnedId) { hideTip(); return; }
    const n = cy.getElementById(pinnedId);
    if (n && n.length) { highlight(n); showTip(n); }
    else { pinnedId = null; hideTip(); }
  }

  // Hover: transient spotlight of the node + its wires, with details in the
  // corner. (No edge hover — far too many edges for per-edge tooltips.) Leaving
  // the node restores whatever is pinned. The info is pinned to a screen corner
  // by showTip so it never covers the node/edges you're inspecting.
  cy.on("mouseover", "node", (evt) => { clearHighlight(); highlight(evt.target); showTip(evt.target); });
  cy.on("mouseout", "node", () => { clearHighlight(); restorePin(); });
  cy.on("pan zoom", () => { if (!pinnedId) hideTip(); });

  // Tap behaviour:
  //  - on a group (box or collapsed token): make it the FOCUSED group (the
  //    target the global SIMD/eq toggle buttons act on), and drop any node pin.
  //  - on a non-group node: PIN its spotlight so its touching edges stay lit
  //    even after the cursor leaves — click a node to inspect what it connects to.
  //  - on empty background: clear both focus and the pin.
  cy.on("tap", (evt) => {
    if (evt.target === cy) { setFocus(null); pinnedId = null; clearHighlight(); hideTip(); return; }
    // Edges are never pickable — ignore taps on them (and never run showTip on
    // an edge, which would show an "undefined" header).
    if (!evt.target.isNode || !evt.target.isNode()) return;
    // A tap can be delivered to an element the dbltap collapse just removed (the
    // tap/dbltap of one double-click) — operating on it crashes cytoscape's style
    // hints. Bail if the target is gone.
    if (evt.target.removed && evt.target.removed()) return;
    let tgt = evt.target;
    // A tap landing in an expanded group's padding (not on a child) targets the
    // GROUP. Prefer a leaf node when the click is actually inside one's box, so
    // nodes hugging the group border are still clickable.
    if (tgt.isParent() && evt.position) {
      const p = evt.position; let near = null, bestD = Infinity;
      cy.nodes().forEach((n) => {
        if (n.isParent()) return;
        const bb = n.boundingBox();
        if (p.x >= bb.x1 - 4 && p.x <= bb.x2 + 4 && p.y >= bb.y1 - 4 && p.y <= bb.y2 + 4) {
          const dx = (bb.x1 + bb.x2) / 2 - p.x, dy = (bb.y1 + bb.y2) / 2 - p.y, d = dx * dx + dy * dy;
          if (d < bestD) { bestD = d; near = n; }
        }
      });
      if (near) tgt = near;
    }
    const id = tgt.id();
    if (groupById[id]) { pinnedId = null; clearHighlight(); hideTip(); setFocus(id); }
    else { setFocus(null); pinnedId = id; clearHighlight(); highlight(tgt); showTip(tgt); }
  });

  // Whole-graph panning from ANYWHERE — including over a node or inside a group
  // box. Cytoscape only pans on a BACKGROUND drag; a drag starting on an element
  // (autoungrabify) is otherwise swallowed, so you can't move the graph from
  // inside the extract_bits box. Fix: when a drag starts on an element, pan
  // manually by the tapdrag delta. Background drags pan natively (we skip those,
  // so no double-pan); a click without movement still fires "tap" for pin/focus
  // because tapdrag only fires once the pointer actually moves.
  let panFrom = null;
  cy.on("tapstart", (evt) => {
    panFrom = (!selectMode && evt.target !== cy && evt.renderedPosition)
      ? { x: evt.renderedPosition.x, y: evt.renderedPosition.y } : null;
  });
  cy.on("tapdrag", (evt) => {
    if (!panFrom || !evt.renderedPosition) return;
    const p = evt.renderedPosition;
    cy.panBy({ x: p.x - panFrom.x, y: p.y - panFrom.y });
    panFrom = { x: p.x, y: p.y };
  });
  cy.on("tapend", () => { panFrom = null; });

  // Sync the toggle buttons' pressed/disabled state for the freshly loaded graph
  // (otherwise a no-op toggle like `expand eq` on extract_bits wouldn't show as
  // disabled until the first focus/toggle).
  if (window.syncToggleButtons) window.syncToggleButtons();
}

function init() {
  const circuits = window.CIRCUITS || [];
  const sel = document.getElementById("circuit-select");
  circuits.forEach((c, i) => {
    const o = document.createElement("option");
    o.value = i; o.textContent = c.name;
    sel.appendChild(o);
  });
  sel.addEventListener("change", () => load(circuits[+sel.value].graph));
  const params = new URLSearchParams(location.search);
  const initIdx = Math.min(circuits.length - 1, Math.max(0, +(params.get("c") || 0)));
  const initLevel = +(params.get("level") || 0);
  document.getElementById("zoom").addEventListener("input", (e) => {
    if (syncingZoom || !cy) return;
    cy.zoom({ level: sliderToZoom(+e.target.value), renderedPosition: { x: cy.width() / 2, y: cy.height() / 2 } });
  });
  document.getElementById("collapse-all").addEventListener("click", () => setLevel(0));
  // Motif layout toggle (experiment): flip the canonical ladder ↔ the generalized
  // edge-length-min, re-laying out in place. Ladder is the default/fallback.
  document.getElementById("layout-mode").addEventListener("click", (e) => {
    window.__layoutMode = window.__layoutMode === "edgemin" ? "ladder" : "edgemin";
    e.target.textContent = `Layout: ${window.__layoutMode === "edgemin" ? "edge-min" : "ladder"}`;
    e.target.classList.toggle("active", window.__layoutMode === "edgemin");
    // runLayout (not rebuild) — the node set is unchanged, so rebuild would
    // no-op-skip the relayout; we need to actually re-run the motif placement.
    runLayout(false);
  });
  document.getElementById("fit").addEventListener("click", () => {
    if (cy) { fitView(); anchorZoom(); }
  });

  // SIMD / eq toggles (both default OFF = merged). Toggling re-runs the merge
  // pipeline from the RAW graph, rebuilds the frontier model, and re-renders +
  // fits. Group ids are stable across both toggles, so the open/expanded set is
  // preserved (filtered to ids that still exist after the rebuild).
  // A merged block/value id <-> the raw lane/member ids it stands for, read from
  // the RAW graph (so it's valid in either merge state). Lets us re-anchor across
  // a toggle even though node ids change (block <-> its lanes).
  const rawMembersOf = (blockId) => {
    for (const b of (rawGraph.simd_blocks || [])) if (b.id === blockId) return b.gate_ids || [];
    for (const sv of (rawGraph.simd_values || [])) if (sv.id === blockId) return sv.member_ids || [];
    return null;
  };
  // A stable representative RAW id for a visible node (a block's first member, or
  // the node's own id for a plain lane/node).
  const repMemberOf = (node) => { const m = rawMembersOf(node.id()); return m && m.length ? m[0] : node.id(); };
  // Find whatever node represents `rep` in the CURRENT view: the lane node itself
  // (un-merged), or the block/value node that now contains it (merged).
  const visibleNodeForRep = (rep) => {
    let n = cy.getElementById(rep); if (n && n.nonempty()) return n;
    for (const b of (rawGraph.simd_blocks || [])) if ((b.gate_ids || []).includes(rep)) { n = cy.getElementById(b.id); if (n.nonempty()) return n; }
    for (const sv of (rawGraph.simd_values || [])) if ((sv.member_ids || []).includes(rep)) { n = cy.getElementById(sv.id); if (n.nonempty()) return n; }
    return null;
  };
  const reapplyMerges = () => {
    if (!cy || !rawGraph) return;
    // Anchor: the visible leaf node nearest the viewport center, and its current
    // SCREEN position. After the relayout we pan so this node's successor lands
    // back on the same screen spot — so toggling SIMD/eq keeps the region you're
    // inspecting in view (preserving zoom/pan alone isn't enough: the un-merge
    // relayout balloons the graph and slides your nodes off-screen).
    const vcx = cy.width() / 2, vcy = cy.height() / 2;
    let anchorRep = null, anchorScreen = null, best = Infinity;
    cy.nodes().forEach((n) => {
      if (n.isParent()) return;
      const r = n.renderedPosition();
      const d = (r.x - vcx) ** 2 + (r.y - vcy) ** 2;
      if (d < best) { best = d; anchorRep = repMemberOf(n); anchorScreen = { x: r.x, y: r.y }; }
    });
    const prevOpen = expanded;
    buildModel(applyMerges());
    posCache = {}; // node ids may have changed (lane gates ↔ block nodes)
    expanded = new Set([...prevOpen].filter((gid) => groupById[gid]));
    // fit=false: a SIMD/eq toggle must PRESERVE the current zoom+pan, so you can
    // inspect a region un-merged without redoing all your navigation. (Group
    // expand/collapse still fits — see rebuild(true) callers.) After the relayout
    // settles, re-pan so the anchored node stays under the same screen point.
    rebuild(false, () => {
      if (!anchorRep || !anchorScreen) return;
      const t = visibleNodeForRep(anchorRep);
      if (t && t.nonempty()) {
        const r = t.renderedPosition();
        cy.panBy({ x: anchorScreen.x - r.x, y: anchorScreen.y - r.y });
      }
    });
  };
  const simdBtn = document.getElementById("toggle-simd");
  const eqBtn = document.getElementById("toggle-eq");
  // Reflect the FOCUSED group's current effective state (or the root default when
  // nothing is focused) in the two toggle buttons' pressed style.
  window.syncToggleButtons = () => {
    simdBtn.classList.toggle("active", mergeStateFor(focusedGroup, "simd"));
    eqBtn.classList.toggle("active", mergeStateFor(focusedGroup, "eq"));
    // #2: disable a toggle when it has nothing to merge, so an inert button
    // (e.g. `expand eq` on extract_bits, where every eq is a constraint) doesn't
    // read as broken. SIMD: any block/value exists. eq: any non-constraint eq
    // whose endpoints share a group (those are the only ones eq-merge touches).
    const simdOk = (rawGraph.simd_blocks || []).length > 0 || (rawGraph.simd_values || []).length > 0;
    const grpOf = {}; (rawGraph.nodes || []).forEach((n) => { grpOf[n.id] = n.group || null; });
    const eqOk = (rawGraph.edges || []).some((e) =>
      e.rel === "eq" && !e.constraint && grpOf[e.source] != null && grpOf[e.source] === grpOf[e.target]);
    const setEnabled = (btn, ok, what) => {
      btn.disabled = !ok;
      btn.title = ok ? "" : `nothing to merge — no ${what} in view`;
    };
    setEnabled(simdBtn, simdOk, "SIMD vectors");
    setEnabled(eqBtn, eqOk, "mergeable equalities");
  };
  // Toggle the `kind` merge state of the focused group (or rootMerge when none is
  // focused), then re-run merges and refit.
  const toggleMerge = (kind) => {
    const next = !mergeStateFor(focusedGroup, kind);
    if (focusedGroup == null) {
      rootMerge[kind] = next;
    } else {
      (groupMerge[focusedGroup] || (groupMerge[focusedGroup] = {}))[kind] = next;
    }
    window.syncToggleButtons();
    // On big graphs the merge-pipeline recompute (over the whole raw graph) is a
    // multi-second synchronous block BEFORE the layout, so show the busy overlay
    // and yield a frame to paint it first. reapplyMerges hides it (in runLayout's
    // layoutstop, or the no-op skip path).
    if (rawGraph.meta && rawGraph.meta.n_gates > 2000) {
      showBusy(true);
      requestAnimationFrame(() => requestAnimationFrame(() => reapplyMerges()));
    } else {
      reapplyMerges();
    }
  };
  simdBtn.addEventListener("click", () => toggleMerge("simd"));
  eqBtn.addEventListener("click", () => toggleMerge("eq"));

  if (circuits.length) {
    sel.value = initIdx;
    load(circuits[initIdx].graph);
    if (initLevel > 0) setTimeout(() => setLevel(initLevel), 80);
  }
}

document.addEventListener("DOMContentLoaded", init);
