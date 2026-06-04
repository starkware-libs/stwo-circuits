"use strict";

// Muted, roughly equal-luminance categorical palette per gate kind
// (Linear-style: color carries meaning, nothing shouts).
const KIND_COLOR = {
  add: "#5e6ad2", sub: "#7c8af0", mul: "#d9912f", pmul: "#c1568f",
  eq: "#8a6fd4", blakeg: "#16a34a", xor: "#d4a72c", m2u: "#0891b2",
  perm: "#b5546b", out: "#3a3f4d", input: "#ffffff",
};
const KIND_SYMBOL = {
  add: "+", sub: "−", mul: "×", pmul: "⊙", eq: "=", blakeg: "G",
  xor: "⊕", m2u: "m→u", perm: "⇄", out: "out", input: "in",
};
const KIND_LABEL = {
  add: "Add", sub: "Sub", mul: "Mul", pmul: "PointwiseMul", eq: "Eq",
  blakeg: "BlakeG", xor: "TripleXor", m2u: "M31ToU32", perm: "Permutation",
  out: "Output", input: "input",
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
let zMin = 0.04, zMax = 3;

let cy = null;
let ec = null; // expand-collapse api
let selectMode = false;
let syncingZoom = false;
let bulkOp = false; // true while setLevel drives many expands/collapses at once
let maxDepth = 1; // group nesting depth of the loaded circuit
// Cached node positions for stable expand/collapse (keyed by group id).
let preExpandPos = {};
let postExpandPos = {};

function snapshotPositions() {
  const m = {};
  cy.nodes().forEach((n) => {
    const p = n.position();
    m[n.id()] = { x: p.x, y: p.y };
  });
  return m;
}
function restorePositions(m) {
  cy.batch(() => {
    for (const id in m) {
      const n = cy.getElementById(id);
      if (n.nonempty()) n.position(m[id]);
    }
  });
}

function buildElements(graph) {
  const els = [];
  for (const g of graph.groups) {
    const d = {
      id: g.id, label: g.label, isGroup: true, depth: g.depth,
      count: g.count || 0, gcolor: familyColor(g.label),
    };
    if (g.parent) d.parent = g.parent;
    els.push({ data: d });
  }
  for (const n of graph.nodes) {
    const d = {
      id: n.id, label: n.label, kind: n.kind, detail: n.detail || "",
      isOutput: !!n.is_output, consts: n.consts || [],
    };
    if (n.group) d.parent = n.group;
    if (n.bk) { d.bk = n.bk; d.bcol = n.bcol; d.brow = n.brow; }
    els.push({ data: d });
  }
  for (const e of graph.edges) {
    els.push({
      data: {
        id: e.id, source: e.source, target: e.target, rel: e.rel,
        vars: e.vars, count: e.count,
        label: e.rel === "eq" ? "=" : e.count > 1 ? `×${e.count}` : "",
      },
    });
  }
  return els;
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
    { selector: "node[?isOutput]",
      style: { "border-width": 2.5, "border-color": "#d9912f" } },
    // Expanded group: a softly tinted container with a title chip on top.
    {
      selector: "node[?isGroup]",
      style: {
        "background-color": "data(gcolor)",
        "background-opacity": (n) => 0.05 + 0.05 * (n.data("depth") || 0),
        shape: "round-rectangle", "corner-radius": 14,
        "border-width": 1, "border-style": "solid",
        "border-color": "data(gcolor)", "border-opacity": 0.55,
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
      selector: "node.cy-expand-collapse-collapsed-node",
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
      style: { "border-width": 3, "border-color": "#5e6ad2" } },
    {
      selector: "edge",
      style: {
        "curve-style": "straight",
        width: (e) => Math.min(1.6, 0.6 + 0.22 * (e.data("count") || 1)),
        "line-color": "#d8dbe3", "target-arrow-color": "#c2c6d2",
        "target-arrow-shape": "triangle", "arrow-scale": 0.45,
        "line-cap": "round", opacity: 0.5,
        label: "data(label)", "font-size": 9, color: "#475569",
        "text-background-color": "#f8fafc", "text-background-opacity": 0.95,
        "text-background-padding": 2, "text-background-shape": "round-rectangle",
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
    { selector: "edge:selected", style: { "line-color": "#5e6ad2", "target-arrow-color": "#5e6ad2" } },
  ];
}

function runLayout(fit = true, onDone) {
  // Align inputs/outputs and fit only after the layout has committed positions
  // (the layout applies positions asynchronously).
  const layout = cy.layout({
    name: "dagre", rankDir: "TB", nodeSep: 22, rankSep: 52,
    edgeSep: 8, animate: false, fit: false, padding: 40,
  });
  layout.one("layoutstop", () => {
    alignInputsOutputs();
    arrangeBlakeBlocks();
    arrangeBlakeReduction();
    if (fit) fitView();
    updateLegend();
    if (onDone) onDone();
  });
  layout.run();
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
const MAX_FIT_ZOOM = 1.4;
const MIN_FIT_ZOOM = 0.3; // don't shrink nodes to specks; pan/minimap for the rest
function fitView() {
  if (!cy) return;
  cy.fit(null, 40);
  const z = cy.zoom();
  if (z > MAX_FIT_ZOOM) { cy.zoom(MAX_FIT_ZOOM); cy.center(); }
  else if (z < MIN_FIT_ZOOM) { cy.zoom(MIN_FIT_ZOOM); cy.center(); }
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

function setLevel(level) {
  if (!ec) return;
  preExpandPos = {};
  postExpandPos = {};
  bulkOp = true;
  ec.collapseAll();
  for (let d = 0; d < level; d++) {
    cy.nodes().forEach((n) => {
      if (n.data("isGroup") && n.data("depth") === d && ec.isExpandable(n)) {
        ec.expand(n, { layoutBy: null });
      }
    });
  }
  bulkOp = false;
  runLayout(true);
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
  if (!cy) return;
  zMin = cy.zoom();
  zMax = zMin * 8;
  syncZoomSlider();
}

function setMode(select) {
  selectMode = select;
  if (cy) {
    cy.userPanningEnabled(!select);
    cy.boxSelectionEnabled(true);
  }
  const btn = document.getElementById("mode");
  btn.textContent = select ? "Select" : "Pan";
  btn.classList.toggle("active", select);
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
      `<b>${esc(KIND_LABEL[kind] || kind)}</b> <span class="muted">${esc(n.id())}</span>`;
    if (n.data("isOutput")) html += ` <span class="muted">· public output</span>`;
    if (n.data("detail")) html += `<br><code>${esc(n.data("detail"))}</code>`;
    if (consts.length) {
      html += `<br><span class="muted">constants:</span> ` +
        consts.map((c) => `${esc(c.name)} = [${c.var}]`).join(", ");
    }
  }
  tip.innerHTML = html;
  // Position near the node, clamped inside the canvas.
  const bb = n.renderedBoundingBox();
  const w = cy.width(), h = cy.height();
  tip.style.display = "block";
  const tw = tip.offsetWidth, th = tip.offsetHeight;
  let left = bb.x1;
  let top = bb.y2 + 8;
  if (left + tw > w - 6) left = Math.max(6, w - 6 - tw);
  if (top + th > h - 6) top = Math.max(6, bb.y1 - 8 - th);
  tip.style.left = `${left}px`;
  tip.style.top = `${top}px`;
}
function hideTip() {
  const tip = document.getElementById("tooltip");
  if (tip) tip.style.display = "none";
}

// Tooltip for a hovered edge (shows the wire's variables).
function showEdgeTip(e) {
  const tip = document.getElementById("tooltip");
  const vars = e.data("vars") || [];
  const rel = e.data("rel");
  tip.innerHTML =
    `<b>${rel === "eq" ? "equality" : "wire"}</b>` +
    (vars.length ? `<br><span class="muted">vars:</span> ${vars.map((v) => `[${v}]`).join(", ")}` : "");
  const mid = e.midpoint();
  const z = cy.zoom(), p = cy.pan();
  tip.style.display = "block";
  tip.style.left = `${mid.x * z + p.x + 8}px`;
  tip.style.top = `${mid.y * z + p.y + 8}px`;
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
  const fam = new Map(); // group family (e.g. "round") -> occurrences in view
  let hasOutput = false, hasEq = false;
  if (cy) {
    cy.nodes().forEach((n) => {
      if (n.data("isGroup")) {
        const f = String(n.data("label")).split("#")[0]; // round#3 -> round
        fam.set(f, (fam.get(f) || 0) + 1);
        return;
      }
      const k = n.data("kind");
      if (k) present.add(k);
      if (n.data("isOutput")) hasOutput = true;
    });
    hasEq = cy.edges('[rel = "eq"]').length > 0;
  }
  const order = ["add", "sub", "mul", "pmul", "blakeg", "xor", "m2u", "perm", "out", "input"]
    .filter((k) => present.has(k));
  const kindRows = order
    .map(
      (k) =>
        `<div class="legend-row"><span class="swatch" style="background:${KIND_COLOR[k]}">${esc(KIND_SYMBOL[k])}</span>${KIND_LABEL[k]}</div>`,
    )
    .join("");
  let html = `<h4>gates in view</h4>${kindRows || '<div class="legend-row hint">— none —</div>'}`;
  if (hasOutput) html += `<div class="legend-row"><span class="swatch" style="border:3px solid #f59e0b"></span>public output</div>`;
  if (hasEq) html += `<div class="legend-row"><span class="swatch swatch-eq"></span>= equality edge</div>`;
  if (fam.size) {
    html += `<h4>groups in view</h4>` +
      [...fam.keys()].sort()
        .map((f) => {
          const n = fam.get(f);
          return `<div class="legend-row"><span class="swatch swatch-group" style="color:${familyColor(f)}"></span>${esc(f)}${n > 1 ? ` <span class="muted">×${n}</span>` : ""}</div>`;
        })
        .join("");
  }
  document.getElementById("legend").innerHTML = html;
}

function load(graph) {
  if (cy) cy.destroy();
  document.getElementById("meta").textContent =
    `${graph.meta.n_gates} gates · ${graph.meta.n_vars} vars · ${graph.meta.n_groups} groups`;

  cy = cytoscape({
    container: document.getElementById("cy"),
    elements: buildElements(graph),
    style: style(),
    wheelSensitivity: 0.25,
    minZoom: ABS_MIN_ZOOM,
    maxZoom: ABS_MAX_ZOOM,
    boxSelectionEnabled: true,
  });

  ec = cy.expandCollapse({
    // Disable the plugin's own relayout; we always call runLayout() ourselves
    // (its async layout was overriding our input/output alignment).
    layoutBy: null,
    fisheye: false, animate: false, undoable: false,
    // No ⊕/⊖ cues — expand/collapse is via double-click.
    cueEnabled: false,
    // Merge the many parallel wires between collapsed groups into one meta-edge.
    groupEdgesOfSameTypeOnCollapse: true,
  });

  maxDepth = Math.max(1, graph.meta.max_depth);
  setLevel(0); // overview: everything collapsed

  setMode(selectMode);
  try { cy.navigator({}); } catch (e) { /* minimap optional */ }

  cy.on("zoom", syncZoomSlider);
  cy.ready(() => { fitView(); anchorZoom(); });
  // Fit again once the container has settled its size.
  setTimeout(() => { if (cy) { fitView(); anchorZoom(); } }, 60);

  // Double-click a group to expand/collapse it, with stable layouts:
  //  - on first expand: lay out once and cache the result;
  //  - on collapse: restore the positions from just before that expand, so the
  //    graph returns exactly to how it looked;
  //  - on re-expand: restore the cached expanded layout (no re-scatter).
  cy.on("dbltap", "node", (evt) => {
    const n = evt.target;
    if (!n.data("isGroup")) return;
    const id = n.id();
    if (ec.isExpandable(n)) {
      preExpandPos[id] = snapshotPositions();
      ec.expand(n);
      if (postExpandPos[id]) {
        restorePositions(postExpandPos[id]);
        fitView();
      } else {
        runLayout(false, () => { postExpandPos[id] = snapshotPositions(); fitView(); });
      }
    } else if (ec.isCollapsible(n)) {
      ec.collapse(n);
      if (preExpandPos[id]) {
        restorePositions(preExpandPos[id]);
        fitView();
      } else {
        runLayout(false, () => fitView());
      }
    }
  });

  // Hover: show full details, then spotlight the node + its wires.
  cy.on("mouseover", "node", (evt) => { showTip(evt.target); highlight(evt.target); });
  cy.on("mouseout", "node", () => { clearHighlight(); hideTip(); });
  cy.on("mouseover", "edge", (evt) => showEdgeTip(evt.target));
  cy.on("mouseout", "edge", hideTip);
  cy.on("pan zoom", hideTip);
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
  document.getElementById("mode").addEventListener("click", () => setMode(!selectMode));
  document.getElementById("collapse-all").addEventListener("click", () => setLevel(0));
  document.getElementById("expand-all").addEventListener("click", () => setLevel(maxDepth));
  document.getElementById("fit").addEventListener("click", () => {
    if (cy) { fitView(); anchorZoom(); }
  });

  if (circuits.length) {
    sel.value = initIdx;
    load(circuits[initIdx].graph);
    if (initLevel > 0) setTimeout(() => setLevel(initLevel), 80);
  }
}

document.addEventListener("DOMContentLoaded", init);
