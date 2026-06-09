//! Converts a built [`Context`] into the serializable [`Graph`] for the viewer.
//!
//! Phase-1 design (the "exporter foundation"): emit the **un-merged raw graph**
//! plus a **node taxonomy** and **annotations** that a later viewer will use to
//! offer merge toggles. Concretely:
//!
//! * **Node taxonomy** — every variable has a producer NODE:
//!   * `gate` nodes (`add`/`sub`/`mul`/`pmul`/`blakeg`/`xor`/`m2u`/`perm`), one
//!     per gate, in its scope group;
//!   * `const` nodes — one **per use** (a fresh node per consuming use), labelled
//!     with the constant's friendly name (`0`/`1`/`u`/relation-id name/value);
//!   * `witness` nodes — every `guess()` var, laid near its consumers;
//!   * `input` nodes — genuine boundary inputs; a guess is promoted witness→input
//!     ONLY when it is a recognized motif's input port (see [`crate::catalog`]).
//! * **eq** — restored as a dashed `rel:"eq"` edge between the producer NODES of
//!   the two operands. Because consts and guesses now have nodes, every operand
//!   always has a node, so the dashed edge is emitted in all cases (no merging).
//! * **SIMD blocks** — detected lane-parallel runs are emitted as `SimdBlock`
//!   annotations over the raw gate node ids; the gates stay individual (no
//!   `simd::` groups, no merge).
//! * **extract_bits** — recognized as a flat GROUP over its raw chain gates.
//! * Scope groups (verify phases, blake block/round, finalize_*) are kept.

use std::collections::{BTreeMap, HashMap, HashSet};

use circuits::circuit::Gate;
use circuits::context::{Context, U_VAR_IDX};
use circuits::ivalue::NoValue;
use circuits::scopes::ScopeSpan;
use stwo::core::fields::qm31::QM31;

use crate::catalog::{Catalog, ExtractBitsSig};
use crate::const_registry::named_constants;
use crate::model::{Edge, Graph, Group, Meta, Node, SimdBlock, SimdValue};

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
    let catalog = crate::catalog::build();

    // --- Friendly names for constants. ---
    // `const_names`: a meaningful, symbolic LABEL per constant var (registry name,
    // `0`/`1`/`u`, a broadcast int / `2^k` / `k⁻¹`, or a compact `c[..]`).
    // `const_values`: the explicit raw QM31 value string (shown in the hover
    // detail), i.e. what the label used to be for non-registry constants.
    let registry = named_constants();
    let mut const_names: HashMap<usize, String> = HashMap::new();
    let mut const_values: HashMap<usize, String> = HashMap::new();
    for (value, var) in ctx.constants() {
        let name = match var.idx {
            0 => "0".to_string(),
            1 => "1".to_string(),
            U_VAR_IDX => "u".to_string(),
            _ => registry
                .get(value)
                .map(|n| n.to_string())
                .unwrap_or_else(|| symbolic_const_name(value)),
        };
        const_names.insert(var.idx, name);
        const_values.insert(var.idx, format!("{value}"));
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

    // --- Identify guessed variables. ---
    // Primary source: the live `guessed_vars` list (present before
    // `finalize_guessed_vars()` runs). Fallback: the trivial `x + 0 = x` finalize
    // gates (an Add with in0 == out) created by `finalize_guessed_vars()`. Those
    // finalize gates do NOT render as gate nodes — a `witness` node stands in for
    // the guessed var instead.
    let mut guessed: HashSet<usize> = HashSet::new();
    let mut finalize_gates: HashSet<String> = HashSet::new();
    if let Some(list) = ctx.guessed_vars.as_ref() {
        guessed.extend(list.iter().copied());
    }
    for r in &recs {
        if r.kind == "add" && r.uses.len() == 2 && r.uses[0] == r.yields[0] {
            // `x + 0 = x` finalize gate: in0 == out.
            guessed.insert(r.uses[0]);
            finalize_gates.insert(r.id.clone());
        }
    }
    // Constants are never witnesses (e.g. the `0` in a finalize gate).
    guessed.retain(|v| !const_names.contains_key(v));

    // --- Build the group hierarchy from scope spans. ---
    let (mut groups, mut group_of, blake_layout) = build_groups(spans, &recs);

    // --- Recognize SIMD blocks (annotation) + extract_bits (group). ---
    let simd_blocks = recognize_simd(&recs, &finalize_gates);
    // `value` (chain-entry) input ports promoted from witness -> input.
    let mut input_ports: HashSet<usize> = HashSet::new();
    // Source-level names for recognized motif guesses (var -> name, e.g. "lsb"),
    // populated by motif recognition and used to label witness nodes/vectors.
    let mut guess_names: HashMap<usize, String> = HashMap::new();
    recognize_extract_bits(
        &recs,
        &simd_blocks,
        &catalog,
        &mut groups,
        &mut group_of,
        &mut input_ports,
        &mut guess_names,
    );

    // --- Producer map: variable -> producer NODE id. ---
    // Gate yields produce their gate node; guessed vars produce a witness/input
    // node; constants are produced per-use (see below), not here.
    let mut producer: HashMap<usize, String> = HashMap::new();
    for r in &recs {
        if finalize_gates.contains(&r.id) {
            continue; // its yield is represented by the witness node
        }
        for &y in &r.yields {
            producer.insert(y, r.id.clone());
        }
    }
    for &g in &guessed {
        // Promoted input ports are `input` nodes; default guesses are `witness`.
        let id = if input_ports.contains(&g) {
            format!("in#{g}")
        } else {
            format!("w#{g}")
        };
        producer.insert(g, id);
    }

    // --- Broadcast-constant SIMD inputs. ---
    // A constant fed identically into every lane of a recognized SIMD block (e.g.
    // extract_bits' `inv_two`, the `in1` of every reduction `simd::mul`) is a
    // BROADCAST constant. Per-use emission explodes it into N identical const
    // chips; instead we collapse it to ONE shared const node per distinct
    // constant value (keyed by the const var — `constant()` dedups by value, so
    // a value maps 1:1 to a var). Each lane gate of the block wires to that one
    // shared node. Scalar (non-block) constant uses stay per-use.
    let rec_by_id: HashMap<&str, &GateRecord> = recs.iter().map(|r| (r.id.as_str(), r)).collect();
    // Map each block-member gate id -> (block id, slot -> broadcast const var).
    // We key broadcast detection per (block, slot): all lane gates of the block
    // use the SAME constant var in that slot.
    let mut gate_block: HashMap<&str, &str> = HashMap::new();
    // (block id, slot) -> the broadcast constant var, when broadcast.
    let mut block_slot_const: HashMap<(&str, usize), usize> = HashMap::new();
    // const var -> (group id -> consumer-lane count), to pick its shared group.
    let mut broadcast_group_votes: HashMap<usize, HashMap<String, usize>> = HashMap::new();
    for b in &simd_blocks {
        let lanes: Vec<&GateRecord> = b
            .gate_ids
            .iter()
            .filter_map(|gid| rec_by_id.get(gid.as_str()).copied())
            .collect();
        if lanes.len() != b.gate_ids.len() || lanes.is_empty() {
            continue;
        }
        for gid in &b.gate_ids {
            gate_block.insert(gid.as_str(), b.id.as_str());
        }
        let n_slots = lanes[0].uses.len();
        for slot in 0..n_slots {
            let v0 = lanes[0].uses.get(slot).copied();
            let Some(v0) = v0 else { continue };
            // Broadcast iff this slot is a constant identical across all lanes.
            if const_names.contains_key(&v0)
                && lanes.iter().all(|r| r.uses.get(slot).copied() == Some(v0))
            {
                block_slot_const.insert((b.id.as_str(), slot), v0);
                // Tally the groups of this block's lane gates for `v0`, so the
                // shared node can land in its consumers' shared group.
                for r in &lanes {
                    if let Some(g) = group_of.get(&r.id) {
                        *broadcast_group_votes.entry(v0).or_default().entry(g.clone()).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    // Shared broadcast-const nodes, created lazily, keyed by const var.
    let mut broadcast_nodes: HashMap<usize, String> = HashMap::new();
    // The group each shared broadcast-const node lands in: the group most of its
    // consuming block gates share (so `inv_two` renders INSIDE extract_bits).
    let broadcast_group: HashMap<usize, Option<String>> = broadcast_group_votes
        .into_iter()
        .map(|(v, votes)| (v, votes.into_iter().max_by_key(|(_, c)| *c).map(|(g, _)| g)))
        .collect();

    // --- Wire dependency DAG (for eq constraint detection). ---
    // For each variable, its producing gate's *input* (use) vars — the wire-DAG
    // predecessors. Var indices form a topo order (a gate's output idx exceeds
    // its input idxs), so dependency search walks strictly DOWN by idx. We use
    // this to decide whether the two sides of an `eq(a, b)` are dependency-
    // related (one computed from the other) — those are genuine constraints and
    // must never be eq-merged (merging would close a cycle / self-loop).
    let mut var_inputs: HashMap<usize, Vec<usize>> = HashMap::new();
    for r in &recs {
        if r.kind == "eq" || r.kind == "out" {
            continue; // these yield nothing
        }
        for &y in &r.yields {
            // Non-const wire inputs only (consts/inputs are DAG leaves anyway).
            let preds: Vec<usize> = r
                .uses
                .iter()
                .copied()
                .filter(|u| !const_names.contains_key(u))
                .collect();
            var_inputs.entry(y).or_default().extend(preds);
        }
    }

    // --- Collapse eq -> dashed edge, output -> badge. ---
    // Every operand now has a producer node (gate / witness / input / per-use
    // const), so eq always becomes a dashed edge. Const operands get a fresh
    // per-use const node here.
    let mut collapsed: HashSet<String> = HashSet::new();
    let mut is_output: HashSet<String> = HashSet::new();
    // (source node, target node, vars, constraint).
    let mut eq_edges: Vec<(String, String, Vec<usize>, bool)> = Vec::new();
    // Per-use const nodes accumulated across eq operands and gate uses.
    let mut const_nodes: Vec<Node> = Vec::new();
    // Shared broadcast-const nodes (one per distinct broadcast constant value).
    let mut broadcast_const_nodes: Vec<Node> = Vec::new();
    let mut const_node_id = |var: usize, owner: &str, slot: usize| -> String {
        let id = format!("k#{owner}#{slot}");
        let name = const_names.get(&var).cloned().unwrap_or_else(|| format!("{var}"));
        // Detail = the explicit raw QM31 value string (the symbolic name's
        // expansion); falls back to the var index if the value is unknown.
        let raw = const_values
            .get(&var)
            .cloned()
            .unwrap_or_else(|| format!("constant variable [{var}]"));
        const_nodes.push(Node {
            id: id.clone(),
            kind: "const".to_string(),
            label: name,
            detail: raw,
            // Per-use const lives in the same group as the gate that consumes it,
            // so motif-internal constants (e.g. extract_bits' inv_two) render
            // INSIDE the motif box instead of floating outside it.
            group: group_of.get(owner).cloned(),
            is_output: false,
            consts: Vec::new(),
            bk: None,
            bcol: None,
            brow: None,
        });
        id
    };

    for r in &recs {
        match r.kind {
            "eq" => {
                let (a, b) = (r.uses[0], r.uses[1]);
                let pa = node_for_operand(a, &producer, &const_names, &r.id, 0, &mut const_node_id);
                let pb = node_for_operand(b, &producer, &const_names, &r.id, 1, &mut const_node_id);
                // A constraint eq has one side computed from the other (the
                // higher-idx side reachable down to the lower-idx side via wire
                // preds). Search only downward from the larger idx toward the
                // smaller, bounded by a visit cap; if the cap is hit we treat it
                // as a constraint (keep the dashed edge — the safe default).
                let constraint = eq_dependency_related(a, b, &var_inputs);
                eq_edges.push((pa, pb, vec![a, b], constraint));
                collapsed.insert(r.id.clone());
            }
            "out" => {
                let v = r.uses[0];
                if let Some(p) = producer.get(&v) {
                    // The output var is produced by a gate: mark that gate.
                    is_output.insert(p.clone());
                    collapsed.insert(r.id.clone());
                } else {
                    // No producer gate: the output is a guess (witness) or a
                    // genuine input. Mark its witness/input NODE instead, so
                    // outputs on bare guesses/inputs are not dropped. (The
                    // witness/input node loop runs after this one and reads the
                    // populated `is_output` set.)
                    let id = if input_ports.contains(&v) {
                        format!("in#{v}")
                    } else {
                        format!("w#{v}")
                    };
                    is_output.insert(id);
                    collapsed.insert(r.id.clone());
                }
            }
            _ => {}
        }
    }

    // --- Gate nodes (non-collapsed, non-finalize gates). ---
    let mut nodes: Vec<Node> = Vec::new();
    for r in &recs {
        if collapsed.contains(&r.id) || finalize_gates.contains(&r.id) {
            continue;
        }
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
            consts: Vec::new(),
            bk,
            bcol,
            brow,
        });
    }

    // --- Witness / input nodes (one per guessed var). ---
    let mut witness_group: HashMap<usize, String> = HashMap::new();
    for &g in &guessed {
        if let Some(grp) = guess_group(g, &recs, &group_of, &finalize_gates) {
            witness_group.insert(g, grp);
        }
    }
    for &g in &guessed {
        let is_input = input_ports.contains(&g);
        let node_id = if is_input { format!("in#{g}") } else { format!("w#{g}") };
        nodes.push(Node {
            id: node_id.clone(),
            kind: if is_input { "input".to_string() } else { "witness".to_string() },
            label: if is_input {
                format!("in[{g}]")
            } else {
                guess_names.get(&g).cloned().unwrap_or_else(|| format!("w[{g}]"))
            },
            detail: if is_input {
                format!("input port (var [{g}])")
            } else {
                format!("guessed witness (var [{g}])")
            },
            // Inputs sit in the top row (no group); witnesses lay near consumers.
            group: if is_input { None } else { witness_group.get(&g).cloned() },
            is_output: is_output.contains(&node_id),
            consts: Vec::new(),
            bk: None,
            bcol: None,
            brow: None,
        });
    }

    // --- Wire edges: producer -> each non-collapsed consumer. ---
    // For constant uses, a fresh per-use const node is the source. Each entry is
    // `(source, target, var, slot)`, where `slot` is the operand position on the
    // target gate (so `op(x, x)` keeps two distinct slot-0/slot-1 edges).
    let mut raw: Vec<(String, String, usize, usize)> = Vec::new();
    for r in &recs {
        if collapsed.contains(&r.id) || finalize_gates.contains(&r.id) {
            continue;
        }
        for (slot, &u) in r.uses.iter().enumerate() {
            if const_names.contains_key(&u) {
                // Broadcast constant fed into every lane of a SIMD block: route
                // all lane gates to ONE shared const node (created lazily),
                // instead of a per-lane-per-use chip.
                let bc = gate_block
                    .get(r.id.as_str())
                    .and_then(|bid| block_slot_const.get(&(*bid, slot)))
                    .filter(|&&cv| cv == u);
                if let Some(&cv) = bc {
                    let cnode = broadcast_nodes.entry(cv).or_insert_with(|| {
                        let id = format!("kbc#{cv}");
                        let name =
                            const_names.get(&cv).cloned().unwrap_or_else(|| format!("{cv}"));
                        // Detail = the explicit raw QM31 value string.
                        let raw = const_values
                            .get(&cv)
                            .cloned()
                            .unwrap_or_else(|| format!("broadcast constant variable [{cv}]"));
                        broadcast_const_nodes.push(Node {
                            id: id.clone(),
                            kind: "const".to_string(),
                            label: name,
                            detail: raw,
                            group: broadcast_group.get(&cv).cloned().flatten(),
                            is_output: false,
                            consts: Vec::new(),
                            bk: None,
                            bcol: None,
                            brow: None,
                        });
                        id
                    });
                    raw.push((cnode.clone(), r.id.clone(), u, slot));
                } else {
                    let cnode = const_node_id(u, &r.id, slot);
                    raw.push((cnode, r.id.clone(), u, slot));
                }
            } else if let Some(p) = producer.get(&u) {
                raw.push((p.clone(), r.id.clone(), u, slot));
            }
        }
    }
    nodes.extend(const_nodes);
    nodes.extend(broadcast_const_nodes);

    // --- Bundle parallel wires. ---
    // Keyed by `(source, target, slot)` so two wires from the same source into
    // different operand slots (e.g. `pmul(x, x)`) stay TWO distinct edges; the
    // target node thus shows its true operand arity. Same-slot parallels (the
    // lane wires of a merged SIMD block feeding one operand slot) still bundle.
    let mut bundles: BTreeMap<(String, String, usize), Vec<usize>> = BTreeMap::new();
    for (s, t, v, slot) in raw {
        bundles.entry((s, t, slot)).or_default().push(v);
    }
    let mut edges: Vec<Edge> = Vec::new();
    for (i, ((s, t, slot), mut vars)) in bundles.into_iter().enumerate() {
        vars.sort_unstable();
        let count = vars.len();
        edges.push(Edge {
            id: format!("w{i}"),
            source: s,
            target: t,
            rel: "wire".into(),
            vars,
            count,
            slot,
            constraint: false,
        });
    }
    for (i, (a, b, vars, constraint)) in eq_edges.into_iter().enumerate() {
        edges.push(Edge {
            id: format!("e{i}"),
            source: a,
            target: b,
            rel: "eq".into(),
            count: 1,
            vars,
            slot: usize::MAX,
            constraint,
        });
    }

    // --- Recognize SIMD VALUE vectors (input / witness lane-vectors). ---
    let simd_values = recognize_simd_values(
        &simd_blocks,
        &rec_by_id,
        &input_ports,
        &guessed,
        &const_names,
        catalog.extract_bits.input_len,
        &guess_names,
    );

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
        simd_blocks,
        simd_values,
    }
}

/// Recognizes SIMD *value* vectors: for each recognized SIMD block, each input
/// slot carries one lane-aligned operand vector (lane `i`'s `uses[slot]`). When
/// every lane var of that slot is a genuine **input port** the vector is an
/// **input** Simd; when every lane var is a (non-input) **witness** guess it is a
/// **witness** Simd. Gate-output and constant (broadcast) slots are skipped — the
/// producing block and the shared broadcast-const node already stand for those.
///
/// Each distinct lane-vector (by its ordered var list) yields exactly ONE
/// [`SimdValue`], so a vector appearing in multiple slots/blocks is not double
/// emitted. `len` is the catalog's known length for the recognized extract_bits
/// input port (`input_len`), else `member_ids.len() * 4` (an upper bound: each
/// packed QM31 holds up to 4 M31 lanes).
fn recognize_simd_values(
    simd_blocks: &[SimdBlock],
    rec_by_id: &HashMap<&str, &GateRecord>,
    input_ports: &HashSet<usize>,
    guessed: &HashSet<usize>,
    const_names: &HashMap<usize, String>,
    extract_bits_input_len: usize,
    guess_names: &HashMap<usize, String>,
) -> Vec<SimdValue> {
    let mut values: Vec<SimdValue> = Vec::new();
    let mut seen: HashSet<Vec<usize>> = HashSet::new();

    for b in simd_blocks {
        let lanes: Vec<&GateRecord> = b
            .gate_ids
            .iter()
            .filter_map(|gid| rec_by_id.get(gid.as_str()).copied())
            .collect();
        if lanes.len() != b.gate_ids.len() || lanes.is_empty() {
            continue;
        }
        let n_slots = lanes[0].uses.len();
        for slot in 0..n_slots {
            // The per-lane operand vector for this slot (lane order).
            if lanes.iter().any(|r| r.uses.len() <= slot) {
                continue;
            }
            let vars: Vec<usize> = lanes.iter().map(|r| r.uses[slot]).collect();

            // Skip constant (broadcast) slots — handled as shared const nodes.
            if vars.iter().any(|v| const_names.contains_key(v)) {
                continue;
            }
            // Classify: all input ports -> input Simd; all witnesses -> witness.
            let all_input = vars.iter().all(|v| input_ports.contains(v));
            let all_witness =
                vars.iter().all(|v| guessed.contains(v) && !input_ports.contains(v));
            let kind = if all_input {
                "input"
            } else if all_witness {
                "witness"
            } else {
                // Mixed, or gate-output vector (produced by another block) — skip.
                continue;
            };

            // De-dup: one SimdValue per distinct lane-vector.
            if !seen.insert(vars.clone()) {
                continue;
            }

            let member_ids: Vec<String> = vars
                .iter()
                .map(|&v| if kind == "input" { format!("in#{v}") } else { format!("w#{v}") })
                .collect();
            // Input port length is definitionally known from the catalog; witness
            // (and any other) vectors fall back to the ≤bound of lanes*4.
            let len = if kind == "input" {
                extract_bits_input_len
            } else {
                member_ids.len() * 4
            };
            // Source-level name (e.g. "lsb"): only when every lane var shares the
            // same recognized motif name.
            let first_name = guess_names.get(&vars[0]);
            let name = first_name
                .filter(|n| vars.iter().all(|v| guess_names.get(v) == Some(*n)))
                .cloned();
            values.push(SimdValue {
                id: format!("simdval-{kind}-{}", vars[0]),
                kind: kind.to_string(),
                member_ids,
                len,
                name,
            });
        }
    }
    values
}

/// Mersenne31 prime, `2^31 - 1`.
const M31_P: u64 = (1 << 31) - 1;

/// A meaningful, symbolic LABEL for a constant QM31 value that is NOT in the
/// named-constant registry (and is not `0`/`1`/`u`, handled by var index).
///
/// Most circuit constants are **broadcast** scalars: the same M31 in all four
/// QM31 coordinates. For such a value `v` we name it by its scalar meaning:
/// * a small int (`v <= SMALL_INT_BOUND`) -> the int itself (`"7"`);
/// * the inverse of a small int (`v*k == 1 mod p`, `k` in `2..=64`) -> `"k⁻¹"`
///   (so the `inv_two` reduction constant renders as `"2⁻¹"`); checked BEFORE the
///   power-of-two case, since the M31 inverse of 2 is itself `2^30`;
/// * a power of two -> `"2^k"`;
/// * otherwise a compact form `"c[<v>]"`.
///
/// Non-broadcast values (coordinates differ) get a compact `"c[..]"` form over
/// the four coordinates so the chip stays small; the raw value lives in the hover.
fn symbolic_const_name(value: &QM31) -> String {
    // The four M31 coordinates as u32: (a + b i) + (c + d i) u.
    let coords = [value.0.0.0, value.0.1.0, value.1.0.0, value.1.1.0];
    let v = coords[0] as u64;
    let broadcast = coords.iter().all(|&c| c as u64 == v);
    if !broadcast {
        return format!("c[{},{},{},{}]", coords[0], coords[1], coords[2], coords[3]);
    }
    const SMALL_INT_BOUND: u64 = 1024;
    if v <= SMALL_INT_BOUND {
        return v.to_string();
    }
    // Inverse of a small int k (k in 2..=64): v*k ≡ 1 (mod p). Checked before the
    // power-of-two case because the M31 inverse of 2 is 2^30 (a power of two too),
    // and `2⁻¹` is the meaningful name there.
    for k in 2u64..=64 {
        if (v * k) % M31_P == 1 {
            return format!("{k}⁻¹");
        }
    }
    if v.is_power_of_two() {
        return format!("2^{}", v.trailing_zeros());
    }
    format!("c[{v}]")
}

/// Maximum number of distinct vars visited while testing whether the two sides
/// of an `eq` are dependency-related. Keeps the bounded BFS fast on large
/// circuits (e.g. c=2's ~44k gates). If the cap is hit, we conservatively report
/// "related" (treat the eq as a constraint that keeps its dashed edge).
const EQ_REACH_CAP: usize = 4096;

/// `true` if the two sides of `eq(a, b)` are DEPENDENCY-RELATED in the wire DAG,
/// i.e. one operand is computed from the other (a genuine constraint such as
/// `x² == x` or `q·b == a`). Such eqs must never be merged — merging closes a
/// cycle and drops the producing gate to an edgeless node.
///
/// `var_inputs[v]` lists the (non-const) wire predecessors of `v`. Var indices
/// are a topo order (a gate output exceeds its inputs), so reachability from `a`
/// to `b` is only possible when `a > b`; we BFS DOWN from the larger idx looking
/// for the smaller, pruning any predecessor with idx below the target. The
/// search is bounded by [`EQ_REACH_CAP`] visited vars; hitting the cap is
/// treated as "related" (the safe default — keep the dashed constraint edge).
fn eq_dependency_related(a: usize, b: usize, var_inputs: &HashMap<usize, Vec<usize>>) -> bool {
    if a == b {
        return true; // degenerate; a self-eq is trivially a constraint.
    }
    let (hi, lo) = if a > b { (a, b) } else { (b, a) };
    let mut stack: Vec<usize> = vec![hi];
    let mut seen: HashSet<usize> = HashSet::new();
    seen.insert(hi);
    while let Some(v) = stack.pop() {
        if seen.len() > EQ_REACH_CAP {
            return true; // cap hit: conservatively keep as a constraint.
        }
        let Some(preds) = var_inputs.get(&v) else { continue };
        for &p in preds {
            if p == lo {
                return true; // `hi` is computed from `lo`.
            }
            // Prune: vars below `lo` cannot reach `lo` (topo order).
            if p > lo && seen.insert(p) {
                stack.push(p);
            }
        }
    }
    false
}

/// Returns the producer node id for an eq operand, creating a per-use const node
/// if the operand is a constant.
fn node_for_operand(
    var: usize,
    producer: &HashMap<usize, String>,
    const_names: &HashMap<usize, String>,
    owner: &str,
    slot: usize,
    const_node_id: &mut impl FnMut(usize, &str, usize) -> String,
) -> String {
    if const_names.contains_key(&var) {
        const_node_id(var, owner, slot)
    } else if let Some(p) = producer.get(&var) {
        p.clone()
    } else {
        // Should not happen now (every non-const var has a node), but keep a
        // stable fallback rather than panicking on an unexpected circuit.
        format!("w#{var}")
    }
}

/// Picks a group for a guessed var's witness node: the deepest group shared by
/// its (non-finalize) consumer gates, so the witness lays near where it is used.
fn guess_group(
    var: usize,
    recs: &[GateRecord],
    group_of: &HashMap<String, String>,
    finalize_gates: &HashSet<String>,
) -> Option<String> {
    // Consumers of `var` and their groups.
    let mut counts: HashMap<String, usize> = HashMap::new();
    for r in recs {
        if finalize_gates.contains(&r.id) || !r.uses.contains(&var) {
            continue;
        }
        if let Some(g) = group_of.get(&r.id) {
            *counts.entry(g.clone()).or_insert(0) += 1;
        }
    }
    // The group most of its consumers share.
    counts.into_iter().max_by_key(|(_, c)| *c).map(|(g, _)| g)
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
    let span_gid: Vec<String> = (0..spans.len()).map(|i| format!("grp{i}")).collect();

    for (i, span) in spans.iter().enumerate() {
        let depth = span.path.len() - 1;
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

    subdivide_blake(spans, &befores, &afters, &span_gid, &mut groups, &mut group_of);

    let blake_layout = compute_blake_layout(spans, &befores, &afters, &span_gid);

    (groups, group_of, blake_layout)
}

/// Computes mirrored grid coordinates for each Blake's reduction/finalizer gates
/// (see the original doc — unchanged Phase-1).
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
        if add1 - add0 != 14 || mul1 - mul0 != 14 || m2u_count < 16 || (m2u_count - 16) % 4 != 0 {
            continue;
        }
        let bid = span_gid[i].clone();
        let msg_end = m0 + (m2u_count - 16);
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
    let parent: HashMap<String, Option<String>> =
        groups.iter().map(|g| (g.id.clone(), g.parent.clone())).collect();
    let mut keep: HashSet<String> = HashSet::new();
    for gid in group_of.values() {
        let mut cur = Some(gid.clone());
        while let Some(id) = cur {
            if !keep.insert(id.clone()) {
                break;
            }
            cur = parent.get(&id).cloned().flatten();
        }
    }
    groups.retain(|g| keep.contains(&g.id));
}

/// For each `blake` span, split its `blakeg` gate range into block -> round
/// sub-groups (unchanged Phase-1).
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
            continue;
        }
        let blake_gid = &span_gid[i];
        let n_blocks = n_g / per_block;
        let blake_depth = span.path.len() - 1;
        for b in 0..n_blocks {
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

/// Recognizes lane-parallel SIMD blocks and emits each as a [`SimdBlock`]
/// annotation over the raw gate node ids (NO grouping, NO merge).
///
/// A single `Simd::add`/`sub`/`mul`/`scalar_mul` call emits N consecutive
/// same-kind gates (one per lane), lane-aligned to its input vectors, with no
/// inter-lane wiring. We scan consecutive same-kind gates (creation order) and
/// greedily extend a run while it stays a valid lane-parallel block. Each run of
/// length >= 2 becomes one `SimdBlock`.
///
/// Op->kind: `Simd::add`->`add`, `Simd::sub`->`sub`, `Simd::mul`->`pmul`,
/// `Simd::scalar_mul`->`mul`.
fn recognize_simd(recs: &[GateRecord], finalize_gates: &HashSet<String>) -> Vec<SimdBlock> {
    const SIMD_OPS: [(&str, &str); 4] =
        [("add", "simd::add"), ("sub", "simd::sub"), ("pmul", "simd::mul"), ("mul", "simd::scalar_mul")];

    let mut blocks: Vec<SimdBlock> = Vec::new();
    for (abbrev, op_label) in SIMD_OPS {
        // Gates of this kind in creation order, excluding the `x + 0 = x`
        // finalize gates (self-referential), which otherwise masquerade as a
        // lane block.
        let kind_recs: Vec<&GateRecord> = recs
            .iter()
            .filter(|r| {
                r.kind == abbrev
                    && !finalize_gates.contains(&r.id)
                    && !r.yields.iter().any(|y| r.uses.contains(y))
            })
            .collect();
        if kind_recs.is_empty() {
            continue;
        }

        let n = kind_recs.len();
        let mut start = 0;
        while start < n {
            let mut end = start + 1;
            while end < n && extends_run(&kind_recs[start..end], kind_recs[end]) {
                end += 1;
            }
            let run = &kind_recs[start..end];
            if run.len() >= 2 {
                blocks.push(SimdBlock {
                    id: format!("simd-{abbrev}-{}", run[0].within_idx),
                    label: op_label.to_string(),
                    lanes: run.len(),
                    gate_ids: run.iter().map(|r| r.id.clone()).collect(),
                });
            }
            start = end;
        }
    }
    blocks
}

/// Returns `true` if appending `next` to the same-kind `run` keeps it a valid
/// lane-parallel SIMD block: mutual independence (no inter-lane wiring) AND, for
/// every input slot, the values across the run are all-equal (a broadcast lane)
/// OR a contiguous run (a fresh lane vector).
fn extends_run(run: &[&GateRecord], next: &GateRecord) -> bool {
    let first = run[0];
    if next.uses.len() != first.uses.len() {
        return false;
    }
    for r in run {
        if r.yields.iter().any(|y| next.uses.contains(y)) {
            return false;
        }
        if next.yields.iter().any(|y| r.uses.contains(y)) {
            return false;
        }
    }
    let len = run.len();
    for s in 0..first.uses.len() {
        let base = first.uses[s];
        let all_equal = run.iter().all(|r| r.uses[s] == base);
        let contiguous = run.iter().enumerate().all(|(i, r)| r.uses[s] == base + i);
        let next_equal = all_equal && next.uses[s] == base;
        let next_contig = contiguous && next.uses[s] == base + len;
        if !next_equal && !next_contig {
            return false;
        }
    }
    true
}

/// Recognizes `extract_bits` invocations as flat GROUPS, recognized at the
/// **SIMD-block level** so each lane-parallel block is wholly inside one motif
/// group (never split across per-lane groups). Promotes each instance's
/// chain-entry input var (the `value` vector fed into the first `sub`-block)
/// from witness -> input, per the catalog's port definition; the per-bit lsb
/// guesses are left as witnesses.
///
/// Structure of one `extract_bits(value, n_bits)` (per the catalog signature) is
/// a chain of SIMD BLOCKS: for each of the first `n_bits-1` bits a `simd::sub`
/// block (`value - lsb`) feeds a `simd::mul` block (`· inv_two`) producing the
/// next `value`; each bit's `assert_bits` adds a `simd::mul` block (`lsb²`) + an
/// `eq` per lane; the final MSB `assert_bits` adds a `simd::mul` block + `eq`s.
/// We anchor on the reduction `sub`-block/`mul`-block chain: a chain is
/// `subs_per_packed` long with each `sub`-block's yields feeding a `mul`-block
/// whose yields feed the next `sub`-block. The chain's first `sub`-block's
/// external in0 vars (across all lanes) are the input ports.
///
/// One `extract_bits` group is emitted per call: its members are the UNION of
/// all gate ids of all blocks in the chain (all lanes) plus the per-bit
/// `assert_bits` `eq` gates. Because membership is over whole blocks, every SIMD
/// block in the chain lies entirely within the one group.
fn recognize_extract_bits(
    recs: &[GateRecord],
    simd_blocks: &[SimdBlock],
    catalog: &Catalog,
    groups: &mut Vec<Group>,
    group_of: &mut HashMap<String, String>,
    input_ports: &mut HashSet<usize>,
    guess_names: &mut HashMap<usize, String>,
) {
    let sig: &ExtractBitsSig = &catalog.extract_bits;
    // The chain has one reduction sub per non-MSB bit.
    debug_assert_eq!(sig.subs_per_packed + 1, sig.n_bits);
    if sig.subs_per_packed == 0 {
        // Nothing to anchor on (would happen only for n_bits == 1).
        assign_counts_and_prune(groups, group_of);
        return;
    }

    let by_id: HashMap<&str, &GateRecord> = recs.iter().map(|r| (r.id.as_str(), r)).collect();
    let producer_gate: HashMap<usize, &str> = recs
        .iter()
        .flat_map(|r| r.yields.iter().map(move |&y| (y, r.id.as_str())))
        .collect();

    // --- A block-level view of each detected SIMD block. ---
    // For each block we precompute the union of its gates' input vars per slot
    // and the union of its yield vars, so we can chain block -> block.
    struct BlockView<'a> {
        block: &'a SimdBlock,
        kind: &'static str,            // gate kind of the block (`sub` / `pmul`).
        in0: HashSet<usize>,           // all in0 vars across lanes.
        yields: HashSet<usize>,        // all yield vars across lanes.
    }
    // The label tells the simd op; map back to gate kind.
    let kind_of = |label: &str| -> Option<&'static str> {
        match label {
            "simd::add" => Some("add"),
            "simd::sub" => Some("sub"),
            "simd::mul" => Some("pmul"),
            "simd::scalar_mul" => Some("mul"),
            _ => None,
        }
    };
    let views: Vec<BlockView> = simd_blocks
        .iter()
        .filter_map(|b| {
            let kind = kind_of(&b.label)?;
            let mut in0 = HashSet::new();
            let mut yields = HashSet::new();
            for gid in &b.gate_ids {
                let r = by_id.get(gid.as_str())?;
                if let Some(&v) = r.uses.first() {
                    in0.insert(v);
                }
                for &y in &r.yields {
                    yields.insert(y);
                }
            }
            Some(BlockView { block: b, kind, in0, yields })
        })
        .collect();

    // `true` if `mul`-view consumes `sub`-view's yields (the `· inv_two`
    // reduction step): the mul block's in0 set equals the sub block's yield set.
    let consumes = |consumer: &BlockView, producer: &BlockView| -> bool {
        !producer.yields.is_empty() && consumer.in0 == producer.yields
    };

    let mut used_blocks: HashSet<&str> = HashSet::new();
    let mut block_id = 0usize;

    // Anchor on `sub`-blocks: a chain entry's in0 vars are NOT the yields of a
    // reduction `mul`-block of an earlier step. We grow greedily and require
    // exactly `subs_per_packed` sub/mul block pairs.
    for sv in views.iter().filter(|v| v.kind == "sub") {
        if used_blocks.contains(sv.block.id.as_str()) {
            continue;
        }
        let mut chain_subs: Vec<&BlockView> = Vec::new();
        let mut chain_muls: Vec<&BlockView> = Vec::new();
        let mut cur: &BlockView = sv;
        let mut ok = true;
        for step in 0..sig.subs_per_packed {
            chain_subs.push(cur);
            // The sub block's yields feed a reduction mul block (`· inv_two`).
            let Some(mul) = views
                .iter()
                .find(|m| m.kind == "pmul" && consumes(m, cur))
            else {
                ok = false;
                break;
            };
            chain_muls.push(mul);
            if step + 1 == sig.subs_per_packed {
                break;
            }
            // The mul block's yields feed the next sub block's in0.
            let Some(next_sub) = views
                .iter()
                .find(|s| s.kind == "sub" && consumes(s, mul))
            else {
                ok = false;
                break;
            };
            cur = next_sub;
        }
        if !ok || chain_subs.len() != sig.subs_per_packed {
            continue;
        }

        // The lsb vars are each sub block's in1 across lanes; the MSB is the
        // chain's final value (the last reduction mul block's yields). Each
        // `assert_bits` adds a `simd::mul` (lsb²) block + a per-lane `eq`. The
        // square blocks are SIMD blocks; collect them via the lsb operands, and
        // the `eq` gates (not SIMD blocks) referencing each lsb and its square.
        let mut lsbs: Vec<usize> = Vec::new();
        for s in &chain_subs {
            for gid in &s.block.gate_ids {
                if let Some(r) = by_id.get(gid.as_str()) {
                    if let Some(&v) = r.uses.get(1) {
                        lsbs.push(v);
                    }
                }
            }
        }
        // Name the per-bit guesses after their source variable (`lsb`), taken
        // from the function body in advance (the MSB, added below, is a computed
        // value — a gate — not a guess, so it is not named here).
        for &v in &lsbs {
            guess_names.insert(v, sig.guess_name.to_string());
        }
        // MSB lane vars = the last reduction mul block's yields.
        let msbs: Vec<usize> = chain_muls.last().map(|m| m.yields.iter().copied().collect()).unwrap_or_default();
        lsbs.extend(&msbs);

        // The square `simd::mul` blocks: a mul block whose every gate is
        // `pmul(x, x)` with `x` an lsb (or MSB) of this chain.
        let lsb_set: HashSet<usize> = lsbs.iter().copied().collect();
        let mut square_blocks: Vec<&BlockView> = Vec::new();
        for v in views.iter().filter(|v| v.kind == "pmul") {
            if used_blocks.contains(v.block.id.as_str()) || chain_muls.iter().any(|m| std::ptr::eq(*m, v)) {
                continue;
            }
            let all_squares = v.block.gate_ids.iter().all(|gid| {
                by_id.get(gid.as_str()).is_some_and(|r| {
                    r.uses.len() == 2 && r.uses[0] == r.uses[1] && lsb_set.contains(&r.uses[0])
                })
            });
            if all_squares {
                square_blocks.push(v);
            }
        }

        // Build the member gate-id set: every gate of every block in the chain
        // (sub blocks + reduction mul blocks + square mul blocks), plus each
        // `assert_bits` `eq` (lsb == lsb²) gate.
        let mut members: Vec<String> = Vec::new();
        let mut chain_block_ids: Vec<&str> = Vec::new();
        for b in chain_subs.iter().chain(&chain_muls).chain(&square_blocks) {
            chain_block_ids.push(b.block.id.as_str());
            members.extend(b.block.gate_ids.iter().cloned());
        }
        // `eq(lsb, lsb²)` gates: for each square gate find the eq over both vars.
        for sq in &square_blocks {
            for gid in &sq.block.gate_ids {
                let Some(r) = by_id.get(gid.as_str()) else { continue };
                let lsb = r.uses[0];
                let sq_out = r.yields[0];
                if let Some(eq) = recs.iter().find(|e| {
                    e.kind == "eq" && e.uses.contains(&lsb) && e.uses.contains(&sq_out)
                }) {
                    members.push(eq.id.clone());
                }
            }
        }

        // Sanity: require all members to exist and not be double-claimed.
        if members.iter().any(|m| !by_id.contains_key(m.as_str())) {
            continue;
        }
        for bid in &chain_block_ids {
            used_blocks.insert(bid);
        }

        // Input ports = the chain's first sub block's in0 vars (the `value`
        // vector entries) — across all lanes. Promote producer-less guesses.
        for &port in &chain_subs[0].in0 {
            if !producer_gate.contains_key(&port) {
                input_ports.insert(port);
            }
        }

        // Emit one flat group for this whole call.
        let gid = format!("extract_bits#{block_id}");
        block_id += 1;
        let depth_of: HashMap<&String, usize> =
            groups.iter().map(|g| (&g.id, g.depth)).collect();
        let parent = members
            .iter()
            .filter_map(|m| group_of.get(m))
            .max_by_key(|gid| depth_of.get(gid).copied().unwrap_or(0))
            .cloned();
        let depth = parent
            .as_ref()
            .and_then(|p| groups.iter().find(|g| &g.id == p))
            .map(|g| g.depth + 1)
            .unwrap_or(0);
        groups.push(Group { id: gid.clone(), label: "extract_bits".to_string(), parent, depth, count: 0 });
        for m in &members {
            group_of.insert(m.clone(), gid.clone());
        }
    }

    assign_counts_and_prune(groups, group_of);
}

/// Prunes empty groups then recomputes subtree counts (run after all grouping).
fn assign_counts_and_prune(groups: &mut Vec<Group>, group_of: &HashMap<String, String>) {
    prune_empty(groups, group_of);
    assign_counts(groups, group_of);
}
