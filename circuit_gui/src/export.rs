//! Converts a built [`Context`] into the serializable [`Graph`] for the viewer.
//!
//! Modelling choices (decided with the user):
//! * Gate-centric DAG: nodes are gates; a wire is drawn directly from the gate that yields a
//!   variable to each gate that uses it, merging the use/yield lookup terms into a single edge.
//!   Parallel wires between the same pair are bundled.
//! * Constants (`0`/`1`/`u`/relation ids/...) are inlined as badges on consumers, not nodes. Known
//!   constants get their source name (see [`crate::const_registry`]).
//! * `Eq(a, b)` collapses into an undirected dashed edge between the producers of `a` and `b`;
//!   `Output(v)` collapses into a badge on the producer of `v`.
//! * Groups come from the recorded scope spans (`push_scope`/`pop_scope`), which form a hierarchy.
//!   Blake is recorded as a single `blake` scope and then subdivided arithmetically into `block` /
//!   `round` / `G` here (no scope cost).

use std::collections::{HashMap, HashSet};

use circuits::circuit::Gate;
use circuits::context::{Context, U_VAR_IDX};
use circuits::ivalue::NoValue;
use circuits::scopes::ScopeSpan;

use crate::const_registry::named_constants;
use crate::model::{ConstTag, Edge, Graph, Group, Meta, Node};

/// Gate kinds in the order they appear in `Circuit`, as `(abbrev, short-label)`.
/// The index into this array is the "kind index" used against gate-count
/// snapshots; the abbrev is the stable id prefix.
const KINDS: [(&str, &str); 10] = [
    ("add", "+"),
    ("sub", "−"),
    ("mul", "×"),
    ("pmul", "⊙"),
    ("eq", "="),
    ("blakeg", "G"),
    ("xor", "⊕"),
    ("m2u", "m→u"),
    ("perm", "⇄"),
    ("out", "out"),
];

/// G-functions per Blake round and rounds per Blake block (Blake2s).
const G_PER_ROUND: usize = 8;
const ROUNDS_PER_BLOCK: usize = 10;
const BLAKEG_KIND: usize = 5;

struct GateRecord {
    id: String,
    kind: &'static str,
    label: &'static str,
    detail: String,
    uses: Vec<usize>,
    yields: Vec<usize>,
    kind_idx: usize,
    within_idx: usize,
}

fn collect<G: Gate>(recs: &mut Vec<GateRecord>, kind_idx: usize, gates: &[G]) {
    let (abbrev, label) = KINDS[kind_idx];
    for (i, g) in gates.iter().enumerate() {
        recs.push(GateRecord {
            id: format!("{abbrev}#{i}"),
            kind: abbrev,
            label,
            detail: format!("{g:?}"),
            uses: g.uses(),
            yields: g.yields(),
            kind_idx,
            within_idx: i,
        });
    }
}

pub fn export(ctx: &Context<NoValue>, spans: &[ScopeSpan]) -> Graph {
    let circuit = &ctx.circuit;

    // --- Friendly names for inlined constants. ---
    let registry = named_constants();
    let mut const_names: HashMap<usize, String> = HashMap::new();
    for (value, var) in ctx.constants() {
        let name = match var.idx {
            0 => "0".to_string(),
            1 => "1".to_string(),
            U_VAR_IDX => "u".to_string(),
            _ => registry.get(value).map(|n| n.to_string()).unwrap_or_else(|| format!("{value}")),
        };
        const_names.insert(var.idx, name);
    }

    // --- Collect gates. ---
    let mut recs: Vec<GateRecord> = Vec::new();
    collect(&mut recs, 0, &circuit.add);
    collect(&mut recs, 1, &circuit.sub);
    collect(&mut recs, 2, &circuit.mul);
    collect(&mut recs, 3, &circuit.pointwise_mul);
    collect(&mut recs, 4, &circuit.eq);
    collect(&mut recs, 5, &circuit.blake_g_gate);
    collect(&mut recs, 6, &circuit.triple_xor);
    collect(&mut recs, 7, &circuit.m31_to_u32);
    collect(&mut recs, 8, &circuit.permutation);
    collect(&mut recs, 9, &circuit.output);

    // --- Build the group hierarchy from scope spans. ---
    let (groups, group_of, blake_layout) = build_groups(spans, &recs);

    // --- Producer map: variable -> gate id that yields it. ---
    let mut producer: HashMap<usize, String> = HashMap::new();
    for r in &recs {
        for &y in &r.yields {
            producer.insert(y, r.id.clone());
        }
    }

    // --- Collapse eq -> edge, output -> badge (when the side has a producer). ---
    let mut collapsed: HashSet<String> = HashSet::new();
    let mut is_output: HashSet<String> = HashSet::new();
    let mut eq_edges: Vec<(String, String, Vec<usize>)> = Vec::new();
    for r in &recs {
        match r.kind {
            "eq" => {
                let (a, b) = (r.uses[0], r.uses[1]);
                if let (Some(pa), Some(pb)) = (producer.get(&a), producer.get(&b)) {
                    eq_edges.push((pa.clone(), pb.clone(), vec![a, b]));
                    collapsed.insert(r.id.clone());
                }
            }
            "out" => {
                let v = r.uses[0];
                if let Some(p) = producer.get(&v) {
                    is_output.insert(p.clone());
                    collapsed.insert(r.id.clone());
                }
            }
            _ => {}
        }
    }

    // --- Nodes (non-collapsed gates) + inlined constant badges. ---
    let mut nodes: Vec<Node> = Vec::new();
    for r in &recs {
        if collapsed.contains(&r.id) {
            continue;
        }
        let consts: Vec<ConstTag> = r
            .uses
            .iter()
            .filter_map(|v| const_names.get(v).map(|name| ConstTag { name: name.clone(), var: *v }))
            .collect();
        let (bk, bcol, brow) = match blake_layout.get(&r.id) {
            Some((id, c, row)) => (Some(id.clone()), Some(*c), Some(*row)),
            None => (None, None, None),
        };
        nodes.push(Node {
            id: r.id.clone(),
            kind: r.kind.to_string(),
            label: r.label.to_string(),
            detail: r.detail.clone(),
            group: group_of.get(&r.id).cloned(),
            is_output: is_output.contains(&r.id),
            consts,
            bk,
            bcol,
            brow,
        });
    }

    // --- Wire edges: producer -> each non-collapsed consumer. ---
    let mut consumers: HashMap<usize, Vec<&str>> = HashMap::new();
    for r in &recs {
        if collapsed.contains(&r.id) {
            continue;
        }
        for &u in &r.uses {
            consumers.entry(u).or_default().push(r.id.as_str());
        }
    }

    let mut raw: Vec<(String, String, usize)> = Vec::new();
    let mut input_vars: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
    for v in 0..circuit.n_vars {
        if const_names.contains_key(&v) {
            continue;
        }
        let Some(cons) = consumers.get(&v) else { continue };
        let src = match producer.get(&v) {
            Some(p) => p.clone(),
            None => {
                input_vars.insert(v);
                format!("in#{v}")
            }
        };
        for c in cons {
            raw.push((src.clone(), (*c).to_string(), v));
        }
    }

    for v in input_vars {
        nodes.push(Node {
            id: format!("in#{v}"),
            kind: "input".to_string(),
            label: format!("in[{v}]"),
            detail: format!("input variable [{v}]"),
            group: None,
            is_output: false,
            consts: Vec::new(),
            bk: None,
            bcol: None,
            brow: None,
        });
    }

    // --- Bundle parallel wires. ---
    let mut bundles: HashMap<(String, String), Vec<usize>> = HashMap::new();
    for (s, t, v) in raw {
        bundles.entry((s, t)).or_default().push(v);
    }
    let mut edges: Vec<Edge> = Vec::new();
    for (i, ((s, t), mut vars)) in bundles.into_iter().enumerate() {
        vars.sort_unstable();
        let count = vars.len();
        edges.push(Edge {
            id: format!("w{i}"),
            source: s,
            target: t,
            rel: "wire".into(),
            vars,
            count,
        });
    }
    for (i, (a, b, vars)) in eq_edges.into_iter().enumerate() {
        edges.push(Edge {
            id: format!("e{i}"),
            source: a,
            target: b,
            rel: "eq".into(),
            count: 1,
            vars,
        });
    }

    let max_depth = groups.iter().map(|g| g.depth).max().map(|d| d + 1).unwrap_or(0);
    Graph {
        meta: Meta {
            n_vars: circuit.n_vars,
            n_gates: recs.len(),
            n_groups: groups.len(),
            max_depth,
        },
        nodes,
        edges,
        groups,
    }
}

/// `true` if span `a`'s gate range contains span `b`'s in every kind.
fn contains(
    a_before: &[usize; 10],
    a_after: &[usize; 10],
    b_before: &[usize; 10],
    b_after: &[usize; 10],
) -> bool {
    (0..10).all(|k| a_before[k] <= b_before[k] && b_after[k] <= a_after[k])
}

/// Builds the group hierarchy from `ctx.scope_spans()` and assigns each gate to
/// its deepest enclosing group. Also synthesizes Blake `block`/`round` sub-groups.
type BlakeLayout = HashMap<String, (String, f64, f64)>;

fn build_groups(
    spans: &[ScopeSpan],
    recs: &[GateRecord],
) -> (Vec<Group>, HashMap<String, String>, BlakeLayout) {
    let befores: Vec<[usize; 10]> = spans.iter().map(|s| s.before).collect();
    let afters: Vec<[usize; 10]> = spans.iter().map(|s| s.after).collect();

    let mut groups: Vec<Group> = Vec::new();
    // Map span index -> emitted group id.
    let span_gid: Vec<String> = (0..spans.len()).map(|i| format!("grp{i}")).collect();

    for (i, span) in spans.iter().enumerate() {
        let depth = span.path.len() - 1;
        // Parent: the span whose path is this path minus its last element and
        // whose range contains this one.
        let parent = if depth == 0 {
            None
        } else {
            let prefix = &span.path[..span.path.len() - 1];
            spans
                .iter()
                .enumerate()
                .find(|(j, s)| {
                    *j != i
                        && s.path.as_slice() == prefix
                        && contains(&befores[*j], &afters[*j], &befores[i], &afters[i])
                })
                .map(|(j, _)| span_gid[j].clone())
        };
        groups.push(Group {
            id: span_gid[i].clone(),
            label: span.path.last().cloned().unwrap_or_default(),
            parent,
            depth,
            count: 0,
        });
    }

    // Assign each gate to the deepest span containing it.
    let mut group_of: HashMap<String, String> = HashMap::new();
    for r in recs {
        let mut best: Option<usize> = None;
        for (i, _span) in spans.iter().enumerate() {
            let k = r.kind_idx;
            if befores[i][k] <= r.within_idx && r.within_idx < afters[i][k] {
                match best {
                    Some(b) if spans[b].path.len() >= spans[i].path.len() => {}
                    _ => best = Some(i),
                }
            }
        }
        if let Some(i) = best {
            group_of.insert(r.id.clone(), span_gid[i].clone());
        }
    }

    // Subdivide each Blake scope into block / round groups (G gates are leaves).
    // Any non-round gates (message unpack, reduction, ...) stay as direct gates
    // of the Blake group — no synthetic "other" wrapper.
    subdivide_blake(spans, &befores, &afters, &span_gid, &mut groups, &mut group_of);

    // Recognize lane-parallel SIMD blocks and wrap each in a `simd::<op>` group.
    // Runs AFTER subdivide_blake so simd groups nest under the finest scope.
    recognize_simd(recs, &mut groups, &mut group_of);

    // Drop groups that end up with no gates (directly or via descendants), so we
    // never render an empty box (e.g. an inputs/outputs scope of guesses/badges).
    prune_empty(&mut groups, &group_of);

    assign_counts(&mut groups, &group_of);

    let blake_layout = compute_blake_layout(spans, &befores, &afters, &span_gid);

    (groups, group_of, blake_layout)
}

/// Computes mirrored grid coordinates for each Blake's reduction/finalizer gates
/// so the (otherwise identical) output sub-circuits are laid out symmetrically.
///
/// Per Blake (after rounds): 8 reduction units, each `low(m2u), high(m2u),
/// mul, add`, laid out as 8 columns × rows 0..3; then two `from_partial_evals`
/// finalizers (3 mul + 3 add each) for out0 (units 0–3) and out1 (units 4–7),
/// centered under their halves. All derived from the Blake gate-count ranges.
fn compute_blake_layout(
    spans: &[ScopeSpan],
    befores: &[[usize; 10]],
    afters: &[[usize; 10]],
    span_gid: &[String],
) -> BlakeLayout {
    let mut map: BlakeLayout = HashMap::new();
    for (i, span) in spans.iter().enumerate() {
        if span.path.last().map(|s| s.as_str()) != Some("blake") {
            continue;
        }
        let (add0, add1) = (befores[i][0], afters[i][0]);
        let (mul0, mul1) = (befores[i][2], afters[i][2]);
        let (m0, m1) = (befores[i][7], afters[i][7]);
        let m2u_count = m1 - m0;
        // Expect exactly: 14 adds, 14 muls, and message+16 m2u (reduction=16).
        if add1 - add0 != 14 || mul1 - mul0 != 14 || m2u_count < 16 || (m2u_count - 16) % 4 != 0 {
            continue; // unexpected shape — fall back to dagre for this Blake
        }
        let bid = span_gid[i].clone();
        let msg_end = m0 + (m2u_count - 16); // start of the 16 reduction m2u
        let mut put = |id: String, col: f64, row: f64| {
            map.insert(id, (bid.clone(), col, row));
        };
        for unit in 0..8 {
            let c = unit as f64;
            put(format!("m2u#{}", msg_end + 2 * unit), c, 0.0);
            put(format!("m2u#{}", msg_end + 2 * unit + 1), c, 1.0);
            put(format!("mul#{}", mul0 + unit), c, 2.0);
            put(format!("add#{}", add0 + unit), c, 3.0);
        }
        // Finalizers: out0 over units 0–3 (center col 1.5), out1 over 4–7 (5.5).
        for (half, center) in [(0usize, 1.5f64), (1usize, 5.5f64)] {
            let base = 8 + half * 3;
            let gates = [
                format!("mul#{}", mul0 + base),
                format!("add#{}", add0 + base),
                format!("mul#{}", mul0 + base + 1),
                format!("add#{}", add0 + base + 1),
                format!("mul#{}", mul0 + base + 2),
                format!("add#{}", add0 + base + 2),
            ];
            for (k, id) in gates.into_iter().enumerate() {
                let col = center + if k % 2 == 0 { -0.5 } else { 0.5 };
                put(id, col, 4.0 + (k / 2) as f64);
            }
        }
    }
    map
}

/// Sets each group's `count` to the number of gate nodes in its whole subtree.
fn assign_counts(groups: &mut [Group], group_of: &HashMap<String, String>) {
    let parent: HashMap<String, Option<String>> =
        groups.iter().map(|g| (g.id.clone(), g.parent.clone())).collect();
    let mut count: HashMap<String, usize> = HashMap::new();
    for gid in group_of.values() {
        let mut cur = Some(gid.clone());
        while let Some(id) = cur {
            *count.entry(id.clone()).or_insert(0) += 1;
            cur = parent.get(&id).cloned().flatten();
        }
    }
    for g in groups.iter_mut() {
        g.count = count.get(&g.id).copied().unwrap_or(0);
    }
}

/// Removes groups with no gates directly or in any descendant.
fn prune_empty(groups: &mut Vec<Group>, group_of: &HashMap<String, String>) {
    use std::collections::HashSet;
    let parent: HashMap<String, Option<String>> =
        groups.iter().map(|g| (g.id.clone(), g.parent.clone())).collect();
    let mut keep: HashSet<String> = HashSet::new();
    for gid in group_of.values() {
        // Mark this group and all its ancestors as non-empty.
        let mut cur = Some(gid.clone());
        while let Some(id) = cur {
            if !keep.insert(id.clone()) {
                break; // ancestors already marked
            }
            cur = parent.get(&id).cloned().flatten();
        }
    }
    groups.retain(|g| keep.contains(&g.id));
}

/// For each `blake` span, split its `blakeg` gate range into block -> round
/// sub-groups (arithmetic, no scope cost): each block = 10 rounds, each round =
/// 8 G gates. Reassigns the G gates to their round group.
fn subdivide_blake(
    spans: &[ScopeSpan],
    befores: &[[usize; 10]],
    afters: &[[usize; 10]],
    span_gid: &[String],
    groups: &mut Vec<Group>,
    group_of: &mut HashMap<String, String>,
) {
    let per_block = ROUNDS_PER_BLOCK * G_PER_ROUND; // 80
    for (i, span) in spans.iter().enumerate() {
        if span.path.last().map(|s| s.as_str()) != Some("blake") {
            continue;
        }
        let bg0 = befores[i][BLAKEG_KIND];
        let bg1 = afters[i][BLAKEG_KIND];
        let n_g = bg1 - bg0;
        if n_g == 0 || n_g % per_block != 0 {
            continue; // irregular; leave as a single blake group
        }
        let blake_gid = &span_gid[i];
        let n_blocks = n_g / per_block;
        let blake_depth = span.path.len() - 1;
        for b in 0..n_blocks {
            // Always create the block level (even for a single block).
            let block_gid = format!("{blake_gid}-b{b}");
            groups.push(Group {
                id: block_gid.clone(),
                label: format!("blake block#{b}"),
                parent: Some(blake_gid.clone()),
                depth: blake_depth + 1,
                count: 0,
            });
            for r in 0..ROUNDS_PER_BLOCK {
                let round_gid = format!("{block_gid}-r{r}");
                groups.push(Group {
                    id: round_gid.clone(),
                    label: format!("blake round#{r}"),
                    parent: Some(block_gid.clone()),
                    depth: blake_depth + 2,
                    count: 0,
                });
                let base = bg0 + b * per_block + r * G_PER_ROUND;
                for g in 0..G_PER_ROUND {
                    let gate_id = format!("blakeg#{}", base + g);
                    group_of.insert(gate_id, round_gid.clone());
                }
            }
        }
    }
}

/// Motif catalog — layer 1: recognizes lane-parallel SIMD blocks.
///
/// A single `Simd::add`/`sub`/`mul`/`scalar_mul` call emits N consecutive
/// same-kind gates (one per lane), lane-aligned to its input vectors, with no
/// inter-lane wiring. We scan consecutive same-kind gates (in `within_idx`
/// creation order) and greedily extend a run while it stays a valid
/// lane-parallel block, then close it at the boundary. Each run of length ≥ 2
/// becomes a synthesized `simd::<op>` group nested under the deepest existing
/// group its gates share.
///
/// Op→kind: `Simd::add`→`add`, `Simd::sub`→`sub`, `Simd::mul`→`pmul`
/// (pointwise_mul), `Simd::scalar_mul`→`mul`. The compound SIMD ops (`inv`,
/// `assert_bits`, `guess_inv_or_zero`, `select`, `pow2`, `combine_bits`) emit
/// multi-gate per-lane patterns and are left for a later catalog layer.
fn recognize_simd(
    recs: &[GateRecord],
    groups: &mut Vec<Group>,
    group_of: &mut HashMap<String, String>,
) {
    // (gate-kind abbrev, simd op label).
    const SIMD_OPS: [(&str, &str); 4] =
        [("add", "simd::add"), ("sub", "simd::sub"), ("pmul", "simd::mul"), ("mul", "simd::scalar_mul")];

    let depth_of: HashMap<String, usize> =
        groups.iter().map(|g| (g.id.clone(), g.depth)).collect();

    // Trivial filler gates — `finalize_guessed_vars`' `x + 0 = x` (a yield that
    // is also a use) — are mutually independent with contiguous in0 and a
    // constant-0 in1, so they masquerade as a SIMD block. A real arithmetic gate
    // always yields a FRESH var, so excluding self-referential gates (a yield
    // that appears in uses) drops these false positives structurally, with no
    // scope-name special case.

    for (abbrev, op_label) in SIMD_OPS {
        // Gates of this kind, in creation (`within_idx`) order. `collect` appends
        // each kind's gates in order, so filtering preserves that ordering.
        let kind_recs: Vec<&GateRecord> = recs
            .iter()
            .filter(|r| r.kind == abbrev && !r.yields.iter().any(|y| r.uses.contains(y)))
            .collect();
        if kind_recs.is_empty() {
            continue;
        }

        let n = kind_recs.len();
        let mut start = 0;
        while start < n {
            // Greedily extend a run [start, end) that stays a valid lane block.
            let mut end = start + 1;
            while end < n && extends_run(&kind_recs[start..end], kind_recs[end]) {
                end += 1;
            }
            let run = &kind_recs[start..end];
            if run.len() >= 2 {
                // Parent: the deepest existing group the run's gates share. They
                // should all share one; pick the deepest if they happen to differ.
                let parent = run
                    .iter()
                    .filter_map(|r| group_of.get(&r.id))
                    .max_by_key(|gid| depth_of.get(*gid).copied().unwrap_or(0))
                    .cloned();
                let parent_depth =
                    parent.as_deref().and_then(|p| depth_of.get(p)).copied().unwrap_or(usize::MAX);
                let gid = format!("simd-{abbrev}-{}", run[0].within_idx);
                groups.push(Group {
                    id: gid.clone(),
                    label: op_label.to_string(),
                    parent,
                    depth: parent_depth.wrapping_add(1),
                    count: 0,
                });
                for r in run {
                    group_of.insert(r.id.clone(), gid.clone());
                }
            }
            start = end;
        }
    }
}

/// Returns `true` if appending `next` to the existing same-kind `run` keeps it a
/// valid lane-parallel SIMD block:
/// * mutual independence — `next`'s yields are not used by the run and the run's
///   yields are not used by `next` (no inter-lane wiring); and
/// * input-slot consistency — for every input slot, the values across the run +
///   `next` are EITHER all-equal (a broadcast/constant lane) OR a contiguous run
///   (`uses[s] of gate i == uses[s] of gate 0 + i`, a fresh lane vector).
///
/// The contiguous check is what splits e.g. `extract_bits` (one `Simd::sub` per
/// bit, each independent but with different input vectors) into separate groups:
/// the in0 progression jumps at each bit boundary, breaking contiguity.
fn extends_run(run: &[&GateRecord], next: &GateRecord) -> bool {
    let first = run[0];
    // Same arity is required to compare slots positionally.
    if next.uses.len() != first.uses.len() {
        return false;
    }

    // Independence: no wiring between `next` and any gate already in the run.
    for r in run {
        if r.yields.iter().any(|y| next.uses.contains(y)) {
            return false;
        }
        if next.yields.iter().any(|y| r.uses.contains(y)) {
            return false;
        }
    }

    let len = run.len(); // index `next` would occupy in the run.
    for s in 0..first.uses.len() {
        let base = first.uses[s];
        // Pattern established by the run so far: all-equal or contiguous?
        let all_equal = run.iter().all(|r| r.uses[s] == base);
        let contiguous = run.iter().enumerate().all(|(i, r)| r.uses[s] == base + i);
        // `next` must continue whichever pattern(s) the run still admits.
        let next_equal = all_equal && next.uses[s] == base;
        let next_contig = contiguous && next.uses[s] == base + len;
        if !next_equal && !next_contig {
            return false;
        }
    }
    true
}
