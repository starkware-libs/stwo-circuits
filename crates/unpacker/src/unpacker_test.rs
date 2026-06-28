use circuits::blake::HashValue;
use circuits::circuit::Circuit;
use circuits::context::{Context, TraceContext, Var};
use circuits::ivalue::{IValue, NoValue, qm31_from_u32s};
use circuits::ops::{Guess, eq};
use circuits::wrappers::U32Wrapper;
use num_traits::One;
use rstest::rstest;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasher};

use crate::tree::BinaryTree;
use crate::unpacker::verify_merkle_commitment;

/// Reference (out-of-circuit) single-`QM31` leaf hash, matching `hash_leaf_qm31`.
fn hash_leaf_ref(value: QM31) -> Blake2sHash {
    let bytes: Vec<u8> = value.to_m31_array().iter().flat_map(|m| m.0.to_le_bytes()).collect();
    Blake2sHasher::hash(&bytes)
}

/// Builds the canonical balanced Merkle tree of leaf *values* over `leaf_values` together with its
/// root, in a single shared reduction (pair adjacent siblings; carry a lone odd node up unchanged).
/// The tree leaves carry the raw `QM31` values; the root is the `Blake2sHash` reduction over their
/// leaf hashes. This is the convention the unpacker circuit must reproduce.
fn build_balanced_tree(leaf_values: &[QM31]) -> (BinaryTree<QM31>, Blake2sHash) {
    let mut layer: Vec<(BinaryTree<QM31>, Blake2sHash)> =
        leaf_values.iter().map(|&v| (BinaryTree::Leaf(v), hash_leaf_ref(v))).collect();
    while layer.len() > 1 {
        let mut next: Vec<(BinaryTree<QM31>, Blake2sHash)> = layer
            .chunks_exact(2)
            .map(|p| {
                let node = BinaryTree::Node(Box::new([p[0].0.clone(), p[1].0.clone()]));
                (node, Blake2sHasher::concat_and_hash(&p[0].1, &p[1].1))
            })
            .collect();
        if layer.len() % 2 == 1 {
            next.push(layer.last().unwrap().clone());
        }
        layer = next;
    }
    layer.into_iter().next().expect("at least one leaf")
}

fn example_leaves(n_leaves: u32) -> Vec<QM31> {
    (0..n_leaves).map(|i| qm31_from_u32s(i + 1, i + 2, i + 3, i + 4)).collect()
}

/// Binds two hash values equal word-by-word. Models the caller's obligation to constrain the root
/// returned by [`verify_merkle_commitment`] to the claimed commitment.
fn bind_hash_eq<Value: IValue>(
    context: &mut Context<Value>,
    a: &HashValue<Var>,
    b: &HashValue<Var>,
) {
    for (x, y) in a.iter().zip(b.iter()) {
        eq(context, *x.get(), *y.get());
    }
}

/// Builds a fresh context, hashes the caller-shaped leaf `tree` out of circuit into a
/// [`HashValue<QM31>`] tree (`verify_merkle_commitment` guesses every value internally), runs the
/// commitment circuit, binds its resolved root to the claimed `root`, and returns whether the
/// resulting circuit is satisfied.
fn verify_commitment(capacity: usize, root: Blake2sHash, tree: BinaryTree<QM31>) -> bool {
    let mut context = TraceContext::default();
    let tree = tree.map(&mut |v: QM31| HashValue::from(hash_leaf_ref(v)));
    let root_var = HashValue::from(root).guess(&mut context);
    let computed = verify_merkle_commitment(&mut context, &tree, capacity);
    bind_hash_eq(&mut context, &computed.root, &root_var);
    context.is_circuit_valid()
}

#[rstest]
#[case::one_leaf(1)]
#[case::two_leaves(2)]
#[case::three_leaves(3)]
#[case::four_leaves(4)]
#[case::five_leaves(5)]
#[case::seven_leaves(7)]
#[case::eight_leaves(8)]
fn verify_qm31_commitment_succeeds_balanced_capacity(#[case] n_leaves: u32) {
    let leaf_values = example_leaves(n_leaves);
    let (tree, root) = build_balanced_tree(&leaf_values);
    let capacity = (n_leaves as usize).next_power_of_two();
    assert!(verify_commitment(capacity, root, tree));
}

/// `n < L`: many padding zeros must be dropped without changing the root.
#[rstest]
#[case::one_pad4(1, 4)]
#[case::one_pad8(1, 8)]
#[case::two_pad8(2, 8)]
#[case::three_pad8(3, 8)]
#[case::five_pad16(5, 16)]
#[case::seven_pad16(7, 16)]
#[case::six_pad8(6, 8)]
fn verify_qm31_commitment_succeeds_with_slack_capacity(
    #[case] n_leaves: u32,
    #[case] capacity: usize,
) {
    let leaf_values = example_leaves(n_leaves);
    let (tree, root) = build_balanced_tree(&leaf_values);
    assert!(verify_commitment(capacity, root, tree));
}

/// Repeated leaf values: the multiset soundness argument is multiplicity-aware.
#[test]
fn verify_qm31_commitment_succeeds_with_duplicate_leaves() {
    let v = qm31_from_u32s(7, 7, 7, 7);
    let w = qm31_from_u32s(9, 8, 7, 6);
    // A real leaf adjacent to its duplicate, and a value repeated non-adjacently.
    let leaf_values = vec![v, v, w, v, w];
    let (tree, root) = build_balanced_tree(&leaf_values);
    assert!(verify_commitment(8, root, tree));
}

/// A *caterpillar* is a maximally unbalanced tree: a single left-leaning spine where every internal
/// node has one leaf child and one subtree child, e.g. `(((a,b),c),d)` for four leaves (depth 3 vs.
/// a balanced tree's depth 2). It's the worst case for the depth-independence claim and a shape a
/// flat leaf list cannot express — the Merkle tree input drives it directly.
///
/// Returns the caterpillar tree of leaf *values* together with its out-of-circuit root, mirroring
/// [`build_balanced_tree`].
fn build_caterpillar_tree(leaf_values: &[QM31]) -> (BinaryTree<QM31>, Blake2sHash) {
    let mut tree = BinaryTree::Leaf(leaf_values[0]);
    let mut root = hash_leaf_ref(leaf_values[0]);
    for &v in &leaf_values[1..] {
        tree = BinaryTree::Node(Box::new([tree, BinaryTree::Leaf(v)]));
        root = Blake2sHasher::concat_and_hash(&root, &hash_leaf_ref(v));
    }
    (tree, root)
}

#[test]
fn verify_qm31_commitment_succeeds_for_caterpillar() {
    let leaf_values = example_leaves(4);
    let (tree, root) = build_caterpillar_tree(&leaf_values);
    assert!(verify_commitment(4, root, tree));
}

/// Shape is bound through `root`: a caterpillar witness must not verify against the *balanced* root
/// of the same leaves. The caterpillar reconstructs a different root, so the binding equality
/// against `balanced_root` is unsatisfiable and the circuit is invalid.
#[test]
fn verify_qm31_commitment_fails_on_wrong_shape() {
    let leaf_values = example_leaves(4);
    let (_, balanced_root) = build_balanced_tree(&leaf_values);
    // A caterpillar witness cannot reproduce the balanced root of the same leaves.
    let (caterpillar, _) = build_caterpillar_tree(&leaf_values);
    assert!(!verify_commitment(4, balanced_root, caterpillar));
}

#[test]
fn verify_qm31_commitment_fails_on_wrong_root() {
    let leaf_values = example_leaves(4);
    let (tree, root) = build_balanced_tree(&leaf_values);
    let mut root_bytes = root.0;
    root_bytes[0] ^= 1;
    assert!(!verify_commitment(4, Blake2sHash(root_bytes), tree));
}

#[test]
fn verify_qm31_commitment_fails_on_tampered_leaf() {
    let leaf_values = example_leaves(4);
    let (_, root) = build_balanced_tree(&leaf_values);

    let mut tampered = leaf_values;
    tampered[2] += QM31::one();
    let (tampered_tree, _) = build_balanced_tree(&tampered);

    // Root committed to the originals; tampered leaves reconstruct a different root, so the binding
    // equality is unsatisfiable.
    assert!(!verify_commitment(4, root, tampered_tree));
}

/// A padding zero placed where a real leaf belongs (dropping a real leaf) must not verify against
/// the genuine root.
#[test]
fn verify_qm31_commitment_fails_when_real_leaf_dropped() {
    let leaf_values = example_leaves(4);
    let (_, root) = build_balanced_tree(&leaf_values);
    let (dropped_tree, _) = build_balanced_tree(&leaf_values[..3]);
    // Only three real leaves supplied at capacity 4: the fourth slot becomes padding `Z`, so the
    // circuit reconstructs the root of [l0, l1, l2], which differs from the four-leaf root and
    // leaves the binding equality unsatisfiable.
    assert!(!verify_commitment(4, root, dropped_tree));
}

/// Builds the full circuit (commitment + root binding) for `n` balanced leaves at `capacity`,
/// returning the finalized [`Circuit`] — gates *and* their variable wiring.
/// `verify_merkle_commitment` guesses every value internally, so the `Value` type only drives
/// whether concrete witness values are tracked, never the emitted gates.
fn build_circuit<Value: IValue>(n: usize, capacity: usize) -> Circuit {
    let mut ctx = Context::<Value>::default();
    let leaf_values = example_leaves(n as u32);
    let (value_tree, root) = build_balanced_tree(&leaf_values);
    let tree = value_tree.map(&mut |v: QM31| HashValue::from(hash_leaf_ref(v)));
    // The value-carrying build must satisfy the binding equality, so bind the resolved root to the
    // real root. The topology is independent of the root's value, so the `NoValue` build (whose
    // hashes are absent) produces the same gates.
    let root_words: [u32; 8] =
        std::array::from_fn(|i| u32::from_le_bytes(root.0[i * 4..i * 4 + 4].try_into().unwrap()));
    let root_value: HashValue<Value> =
        HashValue(root_words.map(|w| U32Wrapper::new_unsafe(Value::pack_u32(w))));
    let root_var = root_value.guess(&mut ctx);
    let computed = verify_merkle_commitment(&mut ctx, &tree, capacity);
    bind_hash_eq(&mut ctx, &computed.root, &root_var);
    ctx.finalize(false).context.circuit
}

/// The circuit must not depend on the witness values: building with concrete `QM31` values must
/// yield exactly the same gates (and wiring) as building with `NoValue`.
#[rstest]
#[case(3, 8)]
#[case(5, 8)]
#[case(1, 4)]
fn structure_is_witness_independent(#[case] n: usize, #[case] capacity: usize) {
    let with_values = build_circuit::<QM31>(n, capacity);
    let without_values = build_circuit::<NoValue>(n, capacity);
    assert_eq!(with_values, without_values);
}

/// The circuit is *fixed*: because `verify_merkle_commitment` guesses every leaf and node value
/// internally (padding with guessed — not constant — zeros), the emitted circuit, **including its
/// variable wiring**, is byte-identical for every real-leaf count `n` at a given `capacity`.
#[rstest]
#[case(8)]
#[case(16)]
fn circuit_is_fixed_across_n(#[case] capacity: usize) {
    let reference = build_circuit::<QM31>(1, capacity);
    for n in 2..=capacity {
        assert_eq!(
            build_circuit::<QM31>(n, capacity),
            reference,
            "circuit for n={n} differs from n=1 at capacity={capacity}"
        );
    }
}

/// The circuit is also independent of tree *shape*: a balanced tree and a maximally unbalanced
/// caterpillar over the same leaves at the same `capacity` emit a byte-identical circuit — the
/// "shape is wiring, not topology" guarantee.
#[test]
fn circuit_is_fixed_across_shape() {
    let leaf_values = example_leaves(4);
    let balanced =
        build_balanced_tree(&leaf_values).0.map(&mut |v: QM31| HashValue::from(hash_leaf_ref(v)));
    let caterpillar =
        build_caterpillar_tree(&leaf_values).0.map(&mut |v: QM31| HashValue::from(hash_leaf_ref(v)));

    let circuit_of = |tree: &BinaryTree<HashValue<QM31>>| {
        let mut ctx = TraceContext::default();
        verify_merkle_commitment(&mut ctx, tree, 4);
        ctx.finalize(false).context.circuit
    };
    assert_eq!(circuit_of(&balanced), circuit_of(&caterpillar));
}

/// A valid circuit must satisfy its constraints *and* the soundness-critical lookup invariant that
/// every variable is yielded exactly once ([`Circuit::check_yields`]).
///
/// Note: the stronger `check_vars_used` invariant (every variable is also *consumed*) does not hold
/// here because [`permute_hash_values`](crate::permutation::permute_hash_values) leaves dead tag
/// variables behind; that is a property of the shared utility, not a soundness concern, so we only
/// assert `check_yields`.
#[test]
fn valid_circuit_passes_yield_and_constraint_checks() {
    let leaf_values = example_leaves(5);
    let (value_tree, root) = build_balanced_tree(&leaf_values);

    let mut context = TraceContext::default();
    let tree = value_tree.map(&mut |v: QM31| HashValue::from(hash_leaf_ref(v)));
    let root_var = HashValue::from(root).guess(&mut context);
    let computed = verify_merkle_commitment(&mut context, &tree, 8);
    bind_hash_eq(&mut context, &computed.root, &root_var);

    let finalized = context.finalize(false);
    finalized.validate_circuit();
    finalized.circuit().check_yields();
}

#[test]
#[should_panic(expected = "power of two")]
fn non_power_of_two_capacity_panics() {
    let mut context = TraceContext::default();
    let (value_tree, _) = build_balanced_tree(&example_leaves(3));
    let tree = value_tree.map(&mut |v: QM31| HashValue::from(hash_leaf_ref(v)));
    verify_merkle_commitment(&mut context, &tree, 3);
}

#[test]
#[should_panic(expected = "capacity")]
fn too_many_leaves_panics() {
    let mut context = TraceContext::default();
    let (value_tree, _) = build_balanced_tree(&example_leaves(5));
    let tree = value_tree.map(&mut |v: QM31| HashValue::from(hash_leaf_ref(v)));
    verify_merkle_commitment(&mut context, &tree, 4);
}
