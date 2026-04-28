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
        [2] = [2] + [0]
        [1] = [1] + [0]
        [3] = [1] + [1]
        [6] = [3] + [1]
        [4] = [6] + [1]
        [7] = [4] + [1]
        [8] = [7] + [1]
        [12] = [1] + [10]
        [13] = [2] + [11]
        [14] = [12] + [13]
        [10] = [9] - [3]
        [5] = [2] * [1]
        [9] = [2] * [2]
        [11] = [10] * [2]
        [5] = [2]
        output [2]
    "#]]
    .assert_eq(&format!("{:?}", context.circuit));

    // The chain populated fresh vars [6], [7], [8] with values 3, 5, 6 respectively.
    assert_eq!(context.get(Var { idx: 6 }), M31::from(3u32).into());
    assert_eq!(context.get(Var { idx: 7 }), M31::from(5u32).into());
    assert_eq!(context.get(Var { idx: 8 }), M31::from(6u32).into());

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

    // The plus-one chain is [5],...,[8] (= 2, ..., 5).
    expect![[r#"
        [0] = [0] + [0]
        [2] = [2] + [0]
        [1] = [1] + [0]
        [5] = [1] + [1]
        [6] = [5] + [1]
        [7] = [6] + [1]
        [8] = [7] + [1]
        [9] = [8] + [5]
        [3] = [10] + [5]
        [14] = [1] + [12]
        [15] = [2] + [13]
        [16] = [14] + [15]
        [12] = [11] - [5]
        [4] = [2] * [1]
        [10] = [9] * [8]
        [11] = [2] * [2]
        [13] = [12] * [2]
        [4] = [2]
        output [2]
    "#]]
    .assert_eq(&format!("{:?}", context.circuit));

    // Decomposition intermediates carry the values they represent.
    assert_eq!(context.get(Var { idx: 9 }), M31::from(7u32).into());
    assert_eq!(context.get(Var { idx: 10 }), M31::from(35u32).into());
    assert_eq!(context.get(Var { idx: 3 }), M31::from(37u32).into());
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_broadcast_decomposition() {
    let mut context = TraceContext::default();
    // Add `u`.
    // TODO(Leo): remove this once `u` is added to the default constants.
    context.constant(qm31_from_u32s(0, 0, 1, 0));
    // Broadcast constant (11, 11, 11, 11) — should be yielded as 11 * (1, 1, 1, 1). Since 11 is
    // outside the chain (`min_base = 5`), the M31 factor 11 is itself built via base
    // decomposition: 11 = 2 * 5 + 1.
    context.constant(qm31_from_u32s(11, 11, 11, 11));
    finalize_constants_with_min_base(&mut context, 5);

    // The plus-one chain populates [5]..=[8] for values 2..=5. The QM31 basis allocates [9] = u*u,
    // [10] = u² - 2 = i, [11] = i*u = iu. The ones vector is built as ([1] + [10]) + ([2] + [11])
    // yielding wires [12], [13], [14]. Then the M31 factor 11 is decomposed in base 5:
    // [15] = [5] * [8] (= 2 * 5 = 10) and [16] = [15] + [1] (= 11). Finally the broadcast is
    // yielded by [3] = [16] * [14] (11 * ones).
    expect![[r#"
        [0] = [0] + [0]
        [2] = [2] + [0]
        [1] = [1] + [0]
        [5] = [1] + [1]
        [6] = [5] + [1]
        [7] = [6] + [1]
        [8] = [7] + [1]
        [12] = [1] + [10]
        [13] = [2] + [11]
        [14] = [12] + [13]
        [16] = [15] + [1]
        [10] = [9] - [5]
        [4] = [2] * [1]
        [9] = [2] * [2]
        [11] = [10] * [2]
        [15] = [5] * [8]
        [3] = [16] * [14]
        [4] = [2]
        output [2]
    "#]]
    .assert_eq(&format!("{:?}", context.circuit));

    assert_eq!(context.get(Var { idx: 14 }), qm31_from_u32s(1, 1, 1, 1));
    assert_eq!(context.get(Var { idx: 16 }), M31::from(11u32).into());
    assert_eq!(context.get(Var { idx: 3 }), qm31_from_u32s(11, 11, 11, 11));
    context.circuit.check_yields();
    context.validate_circuit();
}
