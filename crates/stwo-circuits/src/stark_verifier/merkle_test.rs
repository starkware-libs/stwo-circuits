use crate::circuits::blake::HashValue;
use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::stark_verifier::merkle::{hash_leaf_m31s, hash_node};

#[test]
fn hash_leaf_m31s_regression() {
    let mut context = TraceContext::default();

    let values = [context.new_var(1641251221.into())];

    let hash = hash_leaf_m31s(&mut context, &values);

    assert_eq!(context.get(hash.0), qm31_from_u32s(268251613, 660344597, 1395766214, 1277826589));
    assert_eq!(context.get(hash.1), qm31_from_u32s(1447949022, 1496147392, 1638488896, 1977465263));

    let values = [1, 1641251221, 1176667027, 568581975].map(|v: u32| context.new_var(v.into()));

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
