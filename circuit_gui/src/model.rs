//! Serializable graph model emitted to the browser viewer.
//!
//! The model is *gate-centric*: nodes are gates (and a few input pseudo-nodes),
//! and edges are wires. Because every circuit variable is yielded by exactly one
//! gate and used by some gates, a wire is drawn directly producer-gate ->
//! consumer-gate, merging the use- and yield-lookup terms of a variable into a
//! single edge (parallel wires between the same pair are bundled).

use serde::Serialize;

/// A constant wire (`0`, `1`, `u`, ...) consumed by a gate, rendered inline as a
/// badge on the consuming node instead of as a shared high-degree hub node.
#[derive(Serialize)]
pub struct ConstTag {
    /// Friendly name: `"0"`, `"1"`, `"u"`, or the QM31 value for other constants.
    pub name: String,
    /// The variable index of the constant.
    pub var: usize,
}

/// A node in the graph.
///
/// `kind` is one of:
/// * a gate abbreviation (`"add"`, `"sub"`, `"mul"`, `"pmul"`, `"blakeg"`,
///   `"xor"`, `"m2u"`, `"perm"`) — one node per gate;
/// * `"const"` — a per-use constant node (a fresh node per consuming use);
/// * `"witness"` — a prover-guessed (`guess()`) variable that is not a
///   recognized motif's input port;
/// * `"input"` — a genuine boundary input (a guess promoted to a motif input
///   port, laid out in the top row).
#[derive(Serialize)]
pub struct Node {
    pub id: String,
    /// See the struct docs for the full taxonomy.
    pub kind: String,
    /// Short human label shown on the node.
    pub label: String,
    /// Full gate description shown in the side panel (the gate's `Debug`).
    pub detail: String,
    /// Id of the collapsible group (e.g. a Blake invocation) this node belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// True if an `Output` gate marks this node's yielded variable as public.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_output: bool,
    /// Declaration ordinal among outputs: the index of the `output()` call that
    /// made this node public (0-based, in circuit order). Lets the layout order
    /// outputs the way the circuit declares them rather than by node id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_index: Option<usize>,
    /// Inlined constant inputs (rendered as badges).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub consts: Vec<ConstTag>,
    /// Custom layout hint for Blake reduction/finalizer gates: the Blake group
    /// id this gate belongs to, plus grid coordinates `(col, row)` the viewer
    /// uses to place these (otherwise symmetric) sub-circuits deterministically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcol: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brow: Option<f64>,
}

/// An edge between two nodes.
#[derive(Serialize)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    /// `"wire"` for a data wire (producer -> consumer), `"eq"` for an equality
    /// constraint (an `Eq` gate collapsed into an undirected dashed edge).
    pub rel: String,
    /// Variable indices carried by this (possibly bundled) edge.
    pub vars: Vec<usize>,
    /// Number of wires bundled into this edge.
    pub count: usize,
    /// Operand position on the TARGET node for a `wire` edge (so two wires from
    /// the SAME source into different slots, e.g. `pmul(x, x)`, stay distinct
    /// edges and the node shows its true arity). `usize::MAX` for `eq` edges and
    /// group/meta aggregation where an operand slot is not applicable.
    pub slot: usize,
    /// For an `eq` edge: `true` if the two sides are DEPENDENCY-RELATED (one is
    /// computed from the other in the gate DAG), i.e. a genuine constraint
    /// (`x²==x`, `q·b==a`). Such an eq is never merged (always stays a dashed
    /// edge). `false` for independent eqs (mergeable) and for `wire` edges.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub constraint: bool,
}

/// A collapsible compound node in the hierarchy (a recorded scope, e.g. one
/// `blake()` invocation, or a synthetic Blake sub-level like a block/round).
#[derive(Serialize)]
pub struct Group {
    pub id: String,
    pub label: String,
    /// Parent group id, or `None` for a top-level group. Drives nested compound
    /// nodes in the viewer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Depth in the hierarchy (0 = top-level). Used by the viewer's level control.
    pub depth: usize,
    /// Number of gate nodes in this group's whole subtree (shown on the
    /// collapsed pill).
    pub count: usize,
}

/// A recognized lane-parallel SIMD block, emitted as an *annotation* over the
/// raw gate nodes (the gates stay individually in the graph; this does not merge
/// or group them). The viewer can later collapse a block's member gates into a
/// single SIMD node as a toggle.
#[derive(Serialize)]
pub struct SimdBlock {
    /// Stable id for this block.
    pub id: String,
    /// Human label, e.g. `"simd::add"` / `"simd::sub"` / `"simd::mul"`.
    pub label: String,
    /// Number of lanes (member gates) in this block.
    pub lanes: usize,
    /// Ids of the raw gate nodes that make up this block (each exists as a
    /// node in `nodes`).
    pub gate_ids: Vec<String>,
}

/// A recognized SIMD *value* vector: the per-lane operand vars feeding one input
/// slot of a recognized SIMD block, when those vars are all genuine input ports
/// (an **input** Simd) or all prover witnesses (a **witness** Simd). Emitted as
/// an annotation over the existing `in#`/`w#` lane nodes so the viewer can
/// collapse the whole vector into one node when SIMD is merged (mirroring how it
/// collapses gate op-blocks). Gate-output and broadcast-const vectors are NOT
/// emitted here (the producing block / shared broadcast-const node already
/// represents them).
#[derive(Serialize)]
pub struct SimdValue {
    /// Stable id for the merged node (e.g. `simdval-input-<first var>`).
    pub id: String,
    /// `"input"` (all lane vars are input ports) or `"witness"` (all guesses).
    pub kind: String,
    /// The `in#`/`w#` node ids of the member lanes, in lane order.
    pub member_ids: Vec<String>,
    /// Logical Simd length. For the `extract_bits` input port this is the
    /// catalog's known length (16); otherwise `member_ids.len() * 4`, an UPPER
    /// BOUND (each packed QM31 holds up to 4 M31 lanes).
    pub len: usize,
    /// Source-level name for this vector when it is a recognized motif role
    /// (e.g. `"lsb"` for `extract_bits`' per-bit guesses), transcribed from the
    /// motif's code via the catalog. `None` for unrecognized vectors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Summary counts shown in the viewer header.
#[derive(Serialize, Default)]
pub struct Meta {
    pub n_vars: usize,
    pub n_gates: usize,
    pub n_groups: usize,
    /// Deepest group nesting (0 if there are no groups). Bounds the level control.
    pub max_depth: usize,
}

/// The full exported graph for a single circuit.
#[derive(Serialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub groups: Vec<Group>,
    /// Recognized SIMD lane-blocks, as annotations over the raw gate nodes (no
    /// merge). See [`SimdBlock`].
    pub simd_blocks: Vec<SimdBlock>,
    /// Recognized SIMD value vectors (input / witness lane-vectors feeding SIMD
    /// blocks), as annotations over the `in#`/`w#` lane nodes. See [`SimdValue`].
    pub simd_values: Vec<SimdValue>,
    pub meta: Meta,
}
