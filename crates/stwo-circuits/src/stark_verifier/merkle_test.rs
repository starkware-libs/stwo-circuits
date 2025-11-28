use num_traits::Zero;
use rstest::rstest;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::circuits::blake::{HashValue, blake_qm31};
use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::ops::Guess;
use crate::circuits::wrappers::M31Wrapper;
use crate::stark_verifier::merkle::{
    AuthPath, NODE_PREFIX, hash_leaf_m31s, hash_node, merkle_path,
};

#[test]
fn hash_leaf_m31s_regression() {
    let mut context = TraceContext::default();

    let values = [M31Wrapper::from(M31::from(1641251221)).guess(&mut context)];

    let hash = hash_leaf_m31s(&mut context, &values);

    assert_eq!(context.get(hash.0), qm31_from_u32s(268251613, 660344597, 1395766214, 1277826589));
    assert_eq!(context.get(hash.1), qm31_from_u32s(1447949022, 1496147392, 1638488896, 1977465263));

    let values = [1, 1641251221, 1176667027, 568581975]
        .map(|v: u32| M31Wrapper::from(M31::from(v)).guess(&mut context));

    let hash = hash_leaf_m31s(&mut context, &values);

    assert_eq!(context.get(hash.0), qm31_from_u32s(483650195, 1143215778, 1399105963, 121243225));
    assert_eq!(context.get(hash.1), qm31_from_u32s(1343116297, 264974384, 1201369425, 1524730384));

    context.validate_circuit();
}

#[test]
fn hash_node_regression() {
    let mut context = TraceContext::default();

    let left = HashValue(
        context.new_var(qm31_from_u32s(1206199574, 725559475, 484842011, 871283881)),
        context.new_var(qm31_from_u32s(1827188342, 1597668943, 763527182, 238830106)),
    );
    let right = HashValue(
        context.new_var(qm31_from_u32s(314780017, 161087059, 1415631711, 1712686715)),
        context.new_var(qm31_from_u32s(873946371, 993675704, 1750257287, 1496441219)),
    );

    let hash = hash_node(&mut context, left, right);

    assert_eq!(context.get(hash.0), qm31_from_u32s(1290083578, 670256590, 203247471, 492011214));
    assert_eq!(context.get(hash.1), qm31_from_u32s(353269841, 1619070080, 770215254, 1663098736));

    context.validate_circuit();
}

/// Similar to `hash_node`, but for `QM31` values rather than `Var`s.
fn hash_node_qm31(left: HashValue<QM31>, right: HashValue<QM31>) -> HashValue<QM31> {
    let node_prefix = QM31::from(NODE_PREFIX);
    let zero = QM31::zero();
    blake_qm31(&[node_prefix, zero, zero, zero, left.0, left.1, right.0, right.1], 128)
}

#[rstest]
#[case::success(false, false)]
#[case::wrong_bit(true, false)]
#[case::wrong_root(false, true)]
fn test_merkle_path(#[case] wrong_bit: bool, #[case] wrong_root: bool) {
    let mut context = TraceContext::default();

    let leaf_val = HashValue(qm31_from_u32s(1, 2, 3, 4), qm31_from_u32s(5, 6, 7, 8));
    let auth_path0 = HashValue(qm31_from_u32s(9, 10, 11, 12), qm31_from_u32s(13, 14, 15, 16));
    let auth_path1 = HashValue(qm31_from_u32s(17, 18, 19, 20), qm31_from_u32s(21, 22, 23, 24));
    let auth_path2 = HashValue(qm31_from_u32s(25, 26, 27, 28), qm31_from_u32s(29, 30, 31, 32));
    let auth_path3 = HashValue(qm31_from_u32s(33, 34, 35, 36), qm31_from_u32s(37, 38, 39, 40));
    let auth_path4 = HashValue(qm31_from_u32s(41, 42, 43, 44), qm31_from_u32s(45, 46, 47, 48));
    let auth_path = AuthPath(vec![auth_path0, auth_path1, auth_path2, auth_path3, auth_path4])
        .guess(&mut context);

    let leaf = HashValue(context.new_var(leaf_val.0), context.new_var(leaf_val.1));
    let bits = vec![
        context.new_var(qm31_from_u32s(if wrong_bit { 0 } else { 1 }, 0, 0, 0)),
        context.new_var(qm31_from_u32s(1, 0, 0, 0)),
        context.new_var(qm31_from_u32s(0, 0, 0, 0)),
        context.new_var(qm31_from_u32s(0, 0, 0, 0)),
        context.new_var(qm31_from_u32s(1, 0, 0, 0)),
    ];

    let mut node = leaf_val;
    node = hash_node_qm31(auth_path0, node);
    node = hash_node_qm31(auth_path1, node);
    node = hash_node_qm31(node, auth_path2);
    node = hash_node_qm31(node, auth_path3);
    node = hash_node_qm31(auth_path4, node);
    if wrong_root {
        node.0 += qm31_from_u32s(0, 0, 1, 0);
    }
    let root = HashValue(context.new_var(node.0), context.new_var(node.1));

    merkle_path(&mut context, leaf, &bits, root, &auth_path);
    let success = !wrong_bit && !wrong_root;
    assert_eq!(context.is_circuit_valid(), success);
}
