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

/// A node in the graph: either a gate or an input pseudo-node.
#[derive(Serialize)]
pub struct Node {
    pub id: String,
    /// Gate abbreviation (`"add"`, `"mul"`, `"blakeg"`, ...) or `"input"`.
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
    pub meta: Meta,
}
