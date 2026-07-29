//! Out-of-circuit extraction of the *touched skeleton* of a Patricia trie.
//!
//! The skeleton over a batch of `keys` is the union of their root-to-leaf walks: every binary and
//! edge node they pass through, the leaves they land on, and one opaque **sibling** unit per
//! untouched child of a touched node. It is what the (unimplemented) skeleton circuit consumes,
//! and this module is its witness generator — plain Rust types, no circuit types; the circuit-side
//! mirror comes later. [`witness_invariants`] is the witness analogue of
//! [`PatriciaTree::is_canonical`]: everything a satisfying assignment will have to imply.
//!
//! # Position convention
//!
//! Every unit carries the *position prefix* of its node: the key bits above its height, i.e.
//! `path = key >> height` read as a [`Word256`] integer (word 0 least significant, bit 0 the least
//! significant bit — the layout [`word256_bit`] indexes). The root at `height = H` has `path = 0`,
//! and a node at height `h` on key `k`'s walk has `path = k >> h`, so a **leaf's path is its key**
//! — that is what binds a leaf unit to the key it claims. A partial path at height `h` occupies
//! bits `0..H - h`; bits at or above `H - h` are zero, which is the range check a unit's path owes.
//! The two derived relations (design doc §4.2) are
//!
//! ```text
//! binary: child.path  = 2 · parent.path + bit        (bit = key bit h - 1, 0 = left)
//! edge:   bottom.path = parent.path · 2^ℓ + edge.path
//! ```
//!
//! # Absent keys
//!
//! A key is absent iff its walk reaches an edge whose compressed path disagrees with the key's
//! bits. The walk ends there: the diverging edge is still a touched unit, but its `bottom` is
//! untouched and becomes a sibling. That is the non-membership evidence an insert needs, so the
//! extractor supports absent keys from the start (design doc §4.3). Absent keys produce no leaf
//! unit; `keys` may also repeat, and a repeated key still yields one leaf unit.
//!
//! # Inert padding
//!
//! A fixed-topology circuit pads every class up to a [`SkeletonCapacity`] with [`INERT_UNIT`] =
//! `{ height: 0, path: 0, kind: Padding, hash: 0 }`. Two independent properties make it
//! unmistakable for a live unit, so padding cannot be laundered into the live flow:
//!
//! * `kind = Padding` (tag `0`) is never emitted for a real node, and no live slot rule accepts it;
//! * `hash = 0` is the empty-subtree hash — a live leaf's hash is its non-zero value and a live
//!   binary/edge hash is a blake2s output, so the live and inert unit sets stay disjoint even if
//!   the tag were ignored.
//!
//! The inert flow balances on its own: a padded binary slot consumes two inert units and produces
//! one, a padded edge slot consumes and produces one, and padded leaf and sibling slots each
//! produce one. That nets to zero *exactly* because sibling slots are derived as
//! `n_binary - n_leaves + 1`, so padding never disturbs the multiset identity of design doc §4.1.

use std::collections::BTreeMap;
use std::fmt;

use super::reference::{EMPTY_HASH, PatriciaTree, Word256, hash_binary, hash_edge, word256_bit};

#[cfg(test)]
#[path = "skeleton_test.rs"]
mod test;

#[cfg(test)]
#[path = "skeleton_mutation.rs"]
pub mod mutation;

/// A unit's class tag. It is part of the hashed tuple, so it is the domain separator that stops a
/// unit migrating between classes. Tag `0` is reserved for padding: no live node ever carries it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkeletonKind {
    /// The inert tag of a padded slot. See the module docs.
    Padding = 0,
    Leaf = 1,
    Binary = 2,
    Edge = 3,
}

impl SkeletonKind {
    /// The numeric tag that goes into the unit's hashed tuple.
    pub fn tag(self) -> u32 {
        self as u32
    }
}

/// One atom of the multiset identity: the node at `path` / `height`, its class, and its subtree
/// hash. See the module docs for the `path` convention.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SkeletonUnit {
    pub height: u32,
    pub path: Word256,
    pub kind: SkeletonKind,
    pub hash: Word256,
}

/// The unit filling every padded slot. See the module docs for why it cannot pass for a live unit.
pub const INERT_UNIT: SkeletonUnit =
    SkeletonUnit { height: 0, path: [0; 8], kind: SkeletonKind::Padding, hash: EMPTY_HASH };

impl SkeletonUnit {
    pub fn is_inert(&self) -> bool {
        *self == INERT_UNIT
    }
}

/// A binary slot: consumes two child units, produces `out`. Children are tied to their producers
/// only through the multiset, never topologically — hence the duplicated unit values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BinarySlot {
    pub out: SkeletonUnit,
    pub left: SkeletonUnit,
    pub right: SkeletonUnit,
}

/// An edge slot: consumes `bottom`, produces `out`. `length` and `edge_path` are slot-local — they
/// are the compressed run, not a node position, and `edge_path` holds the `length` key bits
/// LSB-aligned exactly as [`PatriciaTree::Edge`] does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeSlot {
    pub out: SkeletonUnit,
    pub bottom: SkeletonUnit,
    pub length: u32,
    pub edge_path: Word256,
}

/// The touched skeleton of one trie: the classified units plus the trie's `height` and claimed
/// `root`. The root *unit* is not stored separately — it is the single produced unit that nothing
/// consumes, and `root` is the hash a caller binds it to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkeletonWitness {
    pub height: u32,
    pub root: Word256,
    pub leaves: Vec<SkeletonUnit>,
    pub binaries: Vec<BinarySlot>,
    pub edges: Vec<EdgeSlot>,
    pub siblings: Vec<SkeletonUnit>,
}

/// Slot budget for a fixed-topology skeleton circuit. Sibling slots are *derived*, not free, and
/// `n_binary` must cover sibling-induced branch points, so it scales as `K·log2(N/K)` rather than
/// `K` (design doc §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkeletonCapacity {
    pub n_leaves: usize,
    pub n_binary: usize,
    pub n_edge: usize,
}

impl SkeletonCapacity {
    /// The derived sibling-slot count `n_binary - n_leaves + 1`. Panics on an infeasible budget.
    pub fn n_siblings(&self) -> usize {
        assert!(self.n_binary + 1 >= self.n_leaves, "infeasible capacity {self:?}");
        self.n_binary + 1 - self.n_leaves
    }

    /// The tight budget of `witness` — the smallest capacity it fits into.
    pub fn covering(witness: &SkeletonWitness) -> Self {
        Self {
            n_leaves: witness.leaves.len(),
            n_binary: witness.binaries.len(),
            n_edge: witness.edges.len(),
        }
    }
}

impl SkeletonWitness {
    /// Pads every class up to `capacity` with inert units. See the module docs for why the inert
    /// flow balances. Panics if the witness does not fit, including the derived sibling budget.
    pub fn padded(&self, capacity: &SkeletonCapacity) -> SkeletonWitness {
        let n_siblings = capacity.n_siblings();
        assert!(capacity.n_leaves >= self.leaves.len(), "leaf slots short of {capacity:?}");
        assert!(capacity.n_binary >= self.binaries.len(), "binary slots short of {capacity:?}");
        assert!(capacity.n_edge >= self.edges.len(), "edge slots short of {capacity:?}");
        assert!(
            n_siblings >= self.siblings.len(),
            "{capacity:?} derives {n_siblings} sibling slots but the witness needs {}",
            self.siblings.len()
        );
        let inert_binary = BinarySlot { out: INERT_UNIT, left: INERT_UNIT, right: INERT_UNIT };
        let inert_edge =
            EdgeSlot { out: INERT_UNIT, bottom: INERT_UNIT, length: 0, edge_path: [0; 8] };
        SkeletonWitness {
            height: self.height,
            root: self.root,
            leaves: pad(&self.leaves, capacity.n_leaves, INERT_UNIT),
            binaries: pad(&self.binaries, capacity.n_binary, inert_binary),
            edges: pad(&self.edges, capacity.n_edge, inert_edge),
            siblings: pad(&self.siblings, n_siblings, INERT_UNIT),
        }
    }
}

/// The invariants [`witness_invariants`] applies, in the order it applies them. **The order is part
/// of the contract**: per-unit and per-slot checks run before the global ones, so a local tampering
/// is attributed to the constraint family it broke rather than to the multiset it also unbalanced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Check {
    /// A padded slot is not fully inert, or an inert unit sits inside a live slot.
    Padding,
    /// A unit's class tag disagrees with its class or its height.
    KindTag,
    /// Canonical form: `ℓ ∈ [1, height]`, aligned edge paths, no edge over edge, no empty node.
    Canonicity,
    /// Height and `path` relations, and a unit's `path` range check.
    Position,
    /// A slot's `out.hash` is not `hash_binary` / `hash_edge` of its inputs.
    Hashing,
    /// The derived sibling count `siblings = binary - leaves + 1`.
    SiblingCount,
    /// Produced units, less the root, do not equal consumed units as multisets.
    Multiset,
    /// The held-out unit is not the claimed root at the trie's height and position `0`.
    Root,
}

/// A rejected witness, tagged with the [`Check`] that rejected it so a negative test can attribute
/// the rejection to the constraint family it aimed at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    pub check: Check,
    pub detail: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.check, self.detail)
    }
}

/// The position prefix of `key` at `height`: `key >> height`. At height 0 it is the key itself.
pub fn key_prefix(key: &Word256, height: u32) -> Word256 {
    assert!(height <= 256, "height {height} exceeds 256");
    let mut out = [0u32; 8];
    for i in height..256 {
        set_bit(&mut out, i - height, word256_bit(key, i));
    }
    out
}

/// Extracts the touched skeleton of `tree` (a canonical trie of `height`) over `keys`. Keys may be
/// absent from the trie and may repeat. `keys` empty means the whole trie is one opaque sibling.
///
/// Panics if `tree` is not canonical at `height` or if a key has bits set at or above `height`.
/// The *empty* trie has no skeleton at all: [`build_trie`](super::reference::build_trie) returns
/// `None` for it, and with no node there is no root unit for the multiset to hold out.
pub fn extract_skeleton(tree: &PatriciaTree, height: u32, keys: &[Word256]) -> SkeletonWitness {
    assert!(tree.is_canonical(height), "tree is not canonical at height {height}");
    for key in keys {
        assert!(
            (height..256).all(|i| word256_bit(key, i) == 0),
            "key {key:?} does not fit height {height}"
        );
    }
    let mut units = Units::default();
    let root = walk(tree, height, [0; 8], keys, &mut units);
    SkeletonWitness {
        height,
        root: root.hash,
        leaves: units.leaves,
        binaries: units.binaries,
        edges: units.edges,
        siblings: units.siblings,
    }
}

/// Everything a satisfying assignment of the skeleton circuit will have to imply, checked out of
/// circuit: class tags, canonicity, positions, node hashing, the derived sibling count, and the
/// production/consumption multiset with the root held out. Applied in [`Check`]'s declared order.
pub fn witness_invariants(witness: &SkeletonWitness) -> Result<(), Violation> {
    check_units(witness)?;
    for slot in &witness.binaries {
        check_binary_slot(slot)?;
    }
    for slot in &witness.edges {
        check_edge_slot(slot)?;
    }
    check_sibling_count(witness)?;
    check_multiset_and_root(witness)
}

/// Accumulator for [`walk`].
#[derive(Default)]
struct Units {
    leaves: Vec<SkeletonUnit>,
    binaries: Vec<BinarySlot>,
    edges: Vec<EdgeSlot>,
    siblings: Vec<SkeletonUnit>,
}

/// Emits the units of `tree` — a subtree at `height` whose position prefix is `path` — touched by
/// `keys`, returning the unit that represents the subtree. An untouched subtree collapses into one
/// opaque sibling unit. Slots are pushed post-order, so the root's slot is always the last.
fn walk(
    tree: &PatriciaTree,
    height: u32,
    path: Word256,
    keys: &[Word256],
    units: &mut Units,
) -> SkeletonUnit {
    if keys.is_empty() {
        let unit = SkeletonUnit { height, path, kind: kind_of(tree), hash: tree.hash() };
        units.siblings.push(unit);
        return unit;
    }
    match tree {
        PatriciaTree::Leaf { value } => {
            assert!(height == 0, "leaf at height {height}");
            let unit = SkeletonUnit { height, path, kind: SkeletonKind::Leaf, hash: *value };
            units.leaves.push(unit);
            unit
        }
        PatriciaTree::Binary { .. } => walk_binary(tree, height, path, keys, units),
        PatriciaTree::Edge { .. } => walk_edge(tree, height, path, keys, units),
    }
}

fn walk_binary(
    node: &PatriciaTree,
    height: u32,
    path: Word256,
    keys: &[Word256],
    units: &mut Units,
) -> SkeletonUnit {
    let PatriciaTree::Binary { left, right } = node else { unreachable!() };
    let (zeros, ones): (Vec<Word256>, Vec<Word256>) =
        keys.iter().copied().partition(|key| word256_bit(key, height - 1) == 0);
    let left = walk(left, height - 1, child_path(&path, 0), &zeros, units);
    let right = walk(right, height - 1, child_path(&path, 1), &ones, units);
    let out = SkeletonUnit {
        height,
        path,
        kind: SkeletonKind::Binary,
        hash: hash_binary(&left.hash, &right.hash),
    };
    units.binaries.push(BinarySlot { out, left, right });
    out
}

fn walk_edge(
    node: &PatriciaTree,
    height: u32,
    path: Word256,
    keys: &[Word256],
    units: &mut Units,
) -> SkeletonUnit {
    let PatriciaTree::Edge { length, path: edge_path, bottom } = node else { unreachable!() };
    let (length, edge_path) = (*length, *edge_path);
    let below_height = height - length;
    let below_path = bottom_path(&path, length, &edge_path);
    // A key whose bits disagree with the compressed run is absent: its walk ends at this edge.
    let below: Vec<Word256> =
        keys.iter().copied().filter(|key| key_prefix(key, below_height) == below_path).collect();
    let bottom = walk(bottom, below_height, below_path, &below, units);
    let out = SkeletonUnit {
        height,
        path,
        kind: SkeletonKind::Edge,
        hash: hash_edge(&bottom.hash, &edge_path, length),
    };
    units.edges.push(EdgeSlot { out, bottom, length, edge_path });
    out
}

fn kind_of(tree: &PatriciaTree) -> SkeletonKind {
    match tree {
        PatriciaTree::Leaf { .. } => SkeletonKind::Leaf,
        PatriciaTree::Binary { .. } => SkeletonKind::Binary,
        PatriciaTree::Edge { .. } => SkeletonKind::Edge,
    }
}

fn set_bit(word: &mut Word256, i: u32, bit: u32) {
    word[(i / 32) as usize] |= bit << (i % 32);
}

/// `value · 2^shift`. Panics if a set bit would be pushed past bit 255.
fn shl(value: &Word256, shift: u32) -> Word256 {
    assert!(shift < 256, "shift {shift} out of range");
    assert!(
        (256 - shift..256).all(|i| word256_bit(value, i) == 0),
        "shifting {value:?} by {shift} overflows 256 bits"
    );
    let mut out = [0u32; 8];
    for i in 0..256 - shift {
        set_bit(&mut out, i + shift, word256_bit(value, i));
    }
    out
}

/// `path · 2 + bit` — a binary node's child position.
fn child_path(path: &Word256, bit: u32) -> Word256 {
    let mut out = shl(path, 1);
    out[0] |= bit;
    out
}

/// `path · 2^length + edge_path` — an edge's `bottom` position (design doc §4.2). The `or` is an
/// add: the shift clears bits below `length` and `edge_path` has none at or above it.
fn bottom_path(path: &Word256, length: u32, edge_path: &Word256) -> Word256 {
    let mut out = shl(path, length);
    for (word, extra) in out.iter_mut().zip(edge_path) {
        *word |= extra;
    }
    out
}

fn pad<T: Clone>(items: &[T], to: usize, filler: T) -> Vec<T> {
    let mut out = items.to_vec();
    out.resize(to, filler);
    out
}

fn violation(check: Check, detail: String) -> Violation {
    Violation { check, detail }
}

fn check_units(witness: &SkeletonWitness) -> Result<(), Violation> {
    for unit in &witness.leaves {
        check_unit(witness, unit, "leaf slot")?;
        if !unit.is_inert() && unit.kind != SkeletonKind::Leaf {
            let detail = format!("leaf slot holds a {:?} unit: {unit:?}", unit.kind);
            return Err(violation(Check::KindTag, detail));
        }
    }
    for unit in &witness.siblings {
        check_unit(witness, unit, "sibling slot")?;
    }
    for slot in &witness.binaries {
        for unit in [&slot.out, &slot.left, &slot.right] {
            check_unit(witness, unit, "binary slot")?;
        }
    }
    for slot in &witness.edges {
        for unit in [&slot.out, &slot.bottom] {
            check_unit(witness, unit, "edge slot")?;
        }
    }
    Ok(())
}

/// Per-unit well-formedness, independent of the slot holding it: inert-or-live, the `path` range
/// check implied by `height`, the tag/height agreement, and non-emptiness.
fn check_unit(
    witness: &SkeletonWitness,
    unit: &SkeletonUnit,
    class: &str,
) -> Result<(), Violation> {
    if unit.is_inert() {
        return Ok(());
    }
    if unit.kind == SkeletonKind::Padding {
        let detail = format!("{class} unit is tagged padding but is not inert: {unit:?}");
        return Err(violation(Check::Padding, detail));
    }
    if unit.height > witness.height {
        let detail = format!("{class} unit above the trie height {}: {unit:?}", witness.height);
        return Err(violation(Check::Position, detail));
    }
    if !(witness.height - unit.height..256).all(|i| word256_bit(&unit.path, i) == 0) {
        let detail = format!("{class} unit path has bits above its height: {unit:?}");
        return Err(violation(Check::Position, detail));
    }
    if (unit.kind == SkeletonKind::Leaf) != (unit.height == 0) {
        let detail = format!("{class} unit tag disagrees with its height: {unit:?}");
        return Err(violation(Check::KindTag, detail));
    }
    if unit.hash == EMPTY_HASH {
        let detail = format!("{class} unit has the empty-subtree hash: {unit:?}");
        return Err(violation(Check::Canonicity, detail));
    }
    Ok(())
}

fn check_binary_slot(slot: &BinarySlot) -> Result<(), Violation> {
    if slot.out.is_inert() {
        if !slot.left.is_inert() || !slot.right.is_inert() {
            let detail = format!("padded binary slot has a live input: {slot:?}");
            return Err(violation(Check::Padding, detail));
        }
        return Ok(());
    }
    if slot.left.is_inert() || slot.right.is_inert() {
        let detail = format!("live binary slot has an inert input: {slot:?}");
        return Err(violation(Check::Padding, detail));
    }
    if slot.out.kind != SkeletonKind::Binary {
        let detail = format!("binary slot produces a {:?} unit: {slot:?}", slot.out.kind);
        return Err(violation(Check::KindTag, detail));
    }
    if slot.out.height != slot.left.height + 1 || slot.out.height != slot.right.height + 1 {
        let detail = format!("binary slot children are not one level down: {slot:?}");
        return Err(violation(Check::Position, detail));
    }
    if slot.left.path != child_path(&slot.out.path, 0)
        || slot.right.path != child_path(&slot.out.path, 1)
    {
        let detail = format!("binary slot children are not at 2·path (+1): {slot:?}");
        return Err(violation(Check::Position, detail));
    }
    if slot.out.hash != hash_binary(&slot.left.hash, &slot.right.hash) {
        let detail = format!("binary slot hash is not blake2s(left ‖ right): {slot:?}");
        return Err(violation(Check::Hashing, detail));
    }
    Ok(())
}

fn check_edge_slot(slot: &EdgeSlot) -> Result<(), Violation> {
    if slot.out.is_inert() {
        if !slot.bottom.is_inert() || slot.length != 0 || slot.edge_path != EMPTY_HASH {
            let detail = format!("padded edge slot is not fully inert: {slot:?}");
            return Err(violation(Check::Padding, detail));
        }
        return Ok(());
    }
    if slot.bottom.is_inert() {
        let detail = format!("live edge slot has an inert bottom: {slot:?}");
        return Err(violation(Check::Padding, detail));
    }
    if slot.out.kind != SkeletonKind::Edge {
        let detail = format!("edge slot produces a {:?} unit: {slot:?}", slot.out.kind);
        return Err(violation(Check::KindTag, detail));
    }
    check_edge_canonicity(slot)?;
    if slot.bottom.height != slot.out.height - slot.length {
        let detail = format!("edge slot bottom is not ℓ levels down: {slot:?}");
        return Err(violation(Check::Position, detail));
    }
    if slot.bottom.path != bottom_path(&slot.out.path, slot.length, &slot.edge_path) {
        let detail = format!("edge slot bottom is not at path·2^ℓ + edge_path: {slot:?}");
        return Err(violation(Check::Position, detail));
    }
    if slot.out.hash != hash_edge(&slot.bottom.hash, &slot.edge_path, slot.length) {
        let detail = format!("edge slot hash is not blake2s(bottom ‖ path ‖ ℓ): {slot:?}");
        return Err(violation(Check::Hashing, detail));
    }
    Ok(())
}

/// The three canonicity rules an edge slot owns. Checked before the position and hash relations, so
/// an out-of-range `ℓ` is attributed here rather than to the relations it also breaks.
fn check_edge_canonicity(slot: &EdgeSlot) -> Result<(), Violation> {
    if slot.length == 0 || slot.length > slot.out.height {
        let detail = format!("edge length outside [1, {}]: {slot:?}", slot.out.height);
        return Err(violation(Check::Canonicity, detail));
    }
    if !(slot.length..256).all(|i| word256_bit(&slot.edge_path, i) == 0) {
        let detail = format!("edge path has bits at or above ℓ: {slot:?}");
        return Err(violation(Check::Canonicity, detail));
    }
    if slot.bottom.kind == SkeletonKind::Edge {
        let detail = format!("edge over edge — the run is not maximally merged: {slot:?}");
        return Err(violation(Check::Canonicity, detail));
    }
    Ok(())
}

/// The identity of design doc §4.1, which holds for the extracted witness *and* for any capacity it
/// is padded to. Checked before the multiset so a class-cardinality error is attributed here.
fn check_sibling_count(witness: &SkeletonWitness) -> Result<(), Violation> {
    let expected = witness.binaries.len() + 1;
    if witness.siblings.len() + witness.leaves.len() != expected {
        let detail = format!(
            "{} siblings + {} leaves != {} binaries + 1",
            witness.siblings.len(),
            witness.leaves.len(),
            witness.binaries.len()
        );
        return Err(violation(Check::SiblingCount, detail));
    }
    Ok(())
}

/// The bottom-up fold, expressed as the circuit expresses it: produced units (leaves, slot outputs,
/// siblings) less consumed units (slot inputs) must leave exactly the root unit.
fn check_multiset_and_root(witness: &SkeletonWitness) -> Result<(), Violation> {
    let mut counts: BTreeMap<SkeletonUnit, i64> = BTreeMap::new();
    for unit in witness.leaves.iter().chain(&witness.siblings) {
        *counts.entry(*unit).or_default() += 1;
    }
    for slot in &witness.binaries {
        *counts.entry(slot.out).or_default() += 1;
        *counts.entry(slot.left).or_default() -= 1;
        *counts.entry(slot.right).or_default() -= 1;
    }
    for slot in &witness.edges {
        *counts.entry(slot.out).or_default() += 1;
        *counts.entry(slot.bottom).or_default() -= 1;
    }
    counts.retain(|_, n| *n != 0);
    let leftover: Vec<(SkeletonUnit, i64)> = counts.into_iter().collect();
    let [(root, 1)] = leftover.as_slice() else {
        let detail = format!("produced minus consumed is not a single unit: {leftover:?}");
        return Err(violation(Check::Multiset, detail));
    };
    if root.hash != witness.root || root.height != witness.height || root.path != [0; 8] {
        let detail = format!(
            "held-out unit {root:?} is not root {:?} at height {}",
            witness.root, witness.height
        );
        return Err(violation(Check::Root, detail));
    }
    Ok(())
}
