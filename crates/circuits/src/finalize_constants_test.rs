use expect_test::expect;
use stwo::core::fields::m31::M31;

use super::*;
use crate::context::TraceContext;

#[test]
fn test_no_constants_beyond_defaults() {
    let mut context = TraceContext::default();
    // Add `u`.
    // TODO(Leo): remove this once `u` is added to the default constants.
    context.constant(qm31_from_u32s(0, 0, 1, 0));
    finalize_constants(&mut context);
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_plus_one_chain_topology() {
    let mut context = TraceContext::default();
    // Add `u`.
    // TODO(Leo): remove this once `u` is added to the default constants.
    context.constant(qm31_from_u32s(0, 0, 1, 0));
    context.constant(M31::from(2u32).into());
    context.constant(M31::from(4u32).into());
    let m31_constants = IndexMap::from([
        (0.into(), Var { idx: 0 }),
        (1.into(), Var { idx: 1 }),
        (2.into(), Var { idx: 3 }),
        (4.into(), Var { idx: 4 }),
    ]);
    assert_eq!(find_max_consecutive(&m31_constants), 2);
    // `min_base = 6` and `find_max_consecutive` returns 2 (gap at 3), so the chain runs 2..=6.
    finalize_constants_with_min_base(&mut context, 6);

    expect![[r#"
        [0] = [0] + [0]
        [1] = [1] + [0]
        [3] = [1] + [1]
        [5] = [3] + [1]
        [4] = [5] + [1]
        [6] = [4] + [1]
        [7] = [6] + [1]
        [2] = [2] * [1]
        output [2]
    "#]]
    .assert_eq(&format!("{:?}", context.circuit));

    // The chain populated fresh vars [6], [7], [8] with values 3, 5, 6 respectively.
    assert_eq!(context.get(Var { idx: 5 }), M31::from(3u32).into());
    assert_eq!(context.get(Var { idx: 6 }), M31::from(5u32).into());
    assert_eq!(context.get(Var { idx: 7 }), M31::from(6u32).into());

    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_large_m31_decomposition() {
    let mut context = TraceContext::default();
    // Add `u`.
    // TODO(Leo): remove this once `u` is added to the default constants.
    context.constant(qm31_from_u32s(0, 0, 1, 0));
    // 37 = (1*5 + 2)*5 + 2 in base `min_base = 5`, so it needs full base decomposition.
    context.constant(M31::from(37u32).into());
    finalize_constants_with_min_base(&mut context, 5);

    // The plus-one chain is [4],...,[7] (= 2, ..., 5).
    expect![[r#"
        [0] = [0] + [0]
        [1] = [1] + [0]
        [4] = [1] + [1]
        [5] = [4] + [1]
        [6] = [5] + [1]
        [7] = [6] + [1]
        [8] = [7] + [4]
        [3] = [9] + [4]
        [2] = [2] * [1]
        [9] = [8] * [7]
        output [2]
    "#]]
    .assert_eq(&format!("{:?}", context.circuit));

    // Decomposition intermediates carry the values they represent.
    assert_eq!(context.get(Var { idx: 8 }), M31::from(7u32).into());
    assert_eq!(context.get(Var { idx: 9 }), M31::from(35u32).into());
    assert_eq!(context.get(Var { idx: 3 }), M31::from(37u32).into());
    context.circuit.check_yields();
    context.validate_circuit();
}
