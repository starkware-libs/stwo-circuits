use expect_test::expect;
use stwo::core::fields::m31::M31;

use super::*;
use crate::context::TraceContext;

#[test]
fn test_no_constants_beyond_defaults() {
    let mut context = TraceContext::default();
    finalize_constants(&mut context);
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_finalize_constants_passes_check_vars_used() {
    let finalized = TraceContext::default().finalize(true);
    finalized.circuit().check_yields();
    finalized.validate_circuit();
}

#[test]
fn test_plus_one_chain_topology() {
    let mut context = TraceContext::default();
    context.constant(M31::from(2u32).into());
    context.constant(M31::from(4u32).into());
    // No limb past `min_base = 6` (no 7), so the base stays at 6 and the chain runs 2..=6.
    let m31_values = HashSet::from([0, 1, 2, 4]);
    assert_eq!(find_max_consecutive(&m31_values, 6), 6);
    finalize_constants_with_min_base(&mut context, 6);

    expect![[r#"
        [0] = [0] + [0]
        [1] = [1] + [0]
        [3] = [1] + [1]
        [5] = [3] + [1]
        [4] = [5] + [1]
        [6] = [4] + [1]
        [7] = [6] + [1]
        [10] = [9] + [1]
        [12] = [10] + [11]
        [9] = [8] - [3]
        [2] = [2] * [1]
        [8] = [2] * [2]
        [11] = [10] * [2]
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
        [12] = [11] + [1]
        [14] = [12] + [13]
        [11] = [10] - [4]
        [2] = [2] * [1]
        [9] = [8] * [7]
        [10] = [2] * [2]
        [13] = [12] * [2]
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

#[test]
fn test_broadcast_decomposition() {
    let mut context = TraceContext::default();
    // Broadcast constant (11, 11, 11, 11) — should be yielded as 11 * (1, 1, 1, 1). Since 11 is
    // outside the chain (`min_base = 5`), the M31 factor 11 is itself built via base
    // decomposition: 11 = 2 * 5 + 1.
    context.constant(qm31_from_u32s(11, 11, 11, 11));
    finalize_constants_with_min_base(&mut context, 5);

    // The plus-one chain populates [4]..=[7] for values 2..=5. The QM31 basis allocates
    // [8] = u*u and [9] = u² - 2 = i. The ones vector (1, 1, 1, 1) lands in [12], built as
    // [10] + [11], where [10] = [9] + [1] = i + 1 and [11] = [10] * [2] = u + iu. Then the M31
    // factor 11 is decomposed in base 5 (11 = 2*5 + 1): [13] = [4] * [7] (= 2 * 5 = 10) and
    // [14] = [13] + [1] (= 11). Finally the broadcast is yielded by [3] = [14] * [12]
    // (11 * ones).
    expect![[r#"
        [0] = [0] + [0]
        [1] = [1] + [0]
        [4] = [1] + [1]
        [5] = [4] + [1]
        [6] = [5] + [1]
        [7] = [6] + [1]
        [10] = [9] + [1]
        [12] = [10] + [11]
        [14] = [13] + [1]
        [9] = [8] - [4]
        [2] = [2] * [1]
        [8] = [2] * [2]
        [11] = [10] * [2]
        [13] = [4] * [7]
        [3] = [14] * [12]
        output [2]
    "#]]
    .assert_eq(&format!("{:?}", context.circuit));

    assert_eq!(context.get(Var { idx: 12 }), qm31_from_u32s(1, 1, 1, 1));
    assert_eq!(context.get(Var { idx: 14 }), M31::from(11u32).into());
    assert_eq!(context.get(Var { idx: 3 }), qm31_from_u32s(11, 11, 11, 11));
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_mixed_m31_and_qm31_constants_small() {
    let mut context = TraceContext::default();
    // General (non-broadcast, non-base-field) QM31 constant. All limbs (1, 2, 3, 4) live in the
    // chain, so no base decomposition is triggered.
    context.constant(qm31_from_u32s(1, 2, 3, 4));
    finalize_constants_with_min_base(&mut context, 5);

    // The plus-one chain populates [4]..=[7] for values 2..=5. The QM31 basis allocates
    // [8] = u*u and [9] = u² - 2 = i. The ones vector (1, 1, 1, 1) is assembled as
    // [10] + [11], where [10] = [9] + [1] = i + 1 and [11] = [10] * [2] = u + iu, with the
    // result in [12] (unused here, since there are no broadcast qm31 constants). The general QM31
    // constant is then assembled as a + b*i + c*u + d*iu:
    //   [13] = [9] * [4]  (i * 2),  [14] = [1] + [13]  (1 + 2*i)
    //   [15] = [9] * [6]  (i * 4),  [16] = [5] + [15]  (3 + 4*i),  [17] = [16] * [2]  (3*u + 4*iu)
    //   [3]  = [14] + [17]           (finally constrain the constant).
    expect![[r#"
        [0] = [0] + [0]
        [1] = [1] + [0]
        [4] = [1] + [1]
        [5] = [4] + [1]
        [6] = [5] + [1]
        [7] = [6] + [1]
        [10] = [9] + [1]
        [12] = [10] + [11]
        [14] = [1] + [13]
        [16] = [5] + [15]
        [3] = [14] + [17]
        [9] = [8] - [4]
        [2] = [2] * [1]
        [8] = [2] * [2]
        [11] = [10] * [2]
        [13] = [9] * [4]
        [15] = [9] * [6]
        [17] = [16] * [2]
        output [2]
    "#]]
    .assert_eq(&format!("{:?}", context.circuit));

    // The reserved Var carries the assembled QM31 value; the partial sums hold the two halves.
    assert_eq!(context.get(Var { idx: 3 }), qm31_from_u32s(1, 2, 3, 4));
    assert_eq!(context.get(Var { idx: 14 }), qm31_from_u32s(1, 2, 0, 0));
    assert_eq!(context.get(Var { idx: 17 }), qm31_from_u32s(0, 0, 3, 4));
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_mixed_m31_and_qm31_constants_large() {
    let mut context = TraceContext::default();
    // Add constants of various types.
    context.constant(qm31_from_u32s(1000, 2000, 3000, 4000));
    context.constant(qm31_from_u32s(1, 1, 1, 1));
    context.constant(qm31_from_u32s(2, 2, 2, 2));
    context.constant(qm31_from_u32s(666, 666, 666, 666));
    context.constant(qm31_from_u32s(3456, 0, 0, 0));
    context.constant(qm31_from_u32s(7890, 0, 0, 0));
    context.constant(qm31_from_u32s(1234, 2, 3, 4));
    context.constant(qm31_from_u32s(0, 1234, 0, 0));
    context.constant(qm31_from_u32s(0, 0, 1234, 0));
    context.constant(qm31_from_u32s(0, 0, 0, 1234));
    finalize_constants(&mut context);
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_find_max_consecutive() {
    // No limb above `min_base`: base stays at `min_base`.
    assert_eq!(find_max_consecutive(&HashSet::from([0]), 2), 2);
    // Consecutive limbs above `min_base` extend the base until the first gap.
    assert_eq!(find_max_consecutive(&HashSet::from([3, 4, 5, 100]), 2), 5);
    // A gap right above `min_base` leaves the base at `min_base`.
    assert_eq!(find_max_consecutive(&HashSet::from([4, 5]), 2), 2);
    // Limbs at or below `min_base` are irrelevant.
    assert_eq!(find_max_consecutive(&HashSet::from([0, 1, 5, 6]), 3), 3);
    // A gap *below* `min_base` doesn't stop the count; the run above still extends the base.
    assert_eq!(find_max_consecutive(&HashSet::from([0, 1, 3, 4, 5, 6]), 3), 6);
}

#[test]
fn test_qm31_limbs_extend_plus_one_chain() {
    let mut context = TraceContext::default();
    // A QM31 constant whose limbs (2, 3, 4, 5) continue the run above `min_base = 2`. Feeding them
    // into `find_max_consecutive` grows the base to 5, so the `+1` chain builds 2..=5 and every
    // limb comes from the chain rather than base decomposition.
    context.constant(qm31_from_u32s(2, 3, 4, 5));
    finalize_constants_with_min_base(&mut context, 2);

    expect![[r#"
        [0] = [0] + [0]
        [1] = [1] + [0]
        [4] = [1] + [1]
        [5] = [4] + [1]
        [6] = [5] + [1]
        [7] = [6] + [1]
        [10] = [9] + [1]
        [12] = [10] + [11]
        [14] = [4] + [13]
        [16] = [6] + [15]
        [3] = [14] + [17]
        [9] = [8] - [4]
        [2] = [2] * [1]
        [8] = [2] * [2]
        [11] = [10] * [2]
        [13] = [9] * [5]
        [15] = [9] * [7]
        [17] = [16] * [2]
        output [2]
    "#]]
    .assert_eq(&format!("{:?}", context.circuit));

    // Chain vars [4]..=[7] hold values 2..=5, all built by `+1` gates.
    assert_eq!(context.get(Var { idx: 4 }), M31::from(2u32).into());
    assert_eq!(context.get(Var { idx: 5 }), M31::from(3u32).into());
    assert_eq!(context.get(Var { idx: 6 }), M31::from(4u32).into());
    assert_eq!(context.get(Var { idx: 7 }), M31::from(5u32).into());
    // The QM31 constant is assembled into its reserved var [3].
    assert_eq!(context.get(Var { idx: 3 }), qm31_from_u32s(2, 3, 4, 5));

    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_limbs_extend_chain_across_min_base_gap() {
    let mut context = TraceContext::default();
    // QM31 constant with limbs {2, 5, 6, 9}, `min_base = 4`. With the always-present 0 and 1 limbs
    // (from `zero`/`one`/`u`), the limb set is {0, 1, 2, 5, 6, 9}. The gap at 3 below `min_base` is
    // ignored (the walk starts at `min_base`); the gap at 7 above it stops the run, so base = 6.
    // Limbs 5 and 6 come from the `+1` chain (2..=6); limb 9 is past the gap and uses base-6
    // decomposition (9 = 1*6 + 3).
    let m31_limbs = HashSet::from([0, 1, 2, 5, 6, 9]);
    assert_eq!(find_max_consecutive(&m31_limbs, 4), 6);
    context.constant(qm31_from_u32s(2, 5, 6, 9));
    finalize_constants_with_min_base(&mut context, 4);

    // Chain runs 2..=6 (vars [4]..=[8]); limb 9 is rebuilt as `1*6 + 3` in var [16], reusing 6.
    expect![[r#"
        [0] = [0] + [0]
        [1] = [1] + [0]
        [4] = [1] + [1]
        [5] = [4] + [1]
        [6] = [5] + [1]
        [7] = [6] + [1]
        [8] = [7] + [1]
        [11] = [10] + [1]
        [13] = [11] + [12]
        [15] = [4] + [14]
        [16] = [8] + [5]
        [18] = [8] + [17]
        [3] = [15] + [19]
        [10] = [9] - [4]
        [2] = [2] * [1]
        [9] = [2] * [2]
        [12] = [11] * [2]
        [14] = [10] * [7]
        [17] = [10] * [16]
        [19] = [18] * [2]
        output [2]
    "#]]
    .assert_eq(&format!("{:?}", context.circuit));

    // Chain vars [4]..=[8] hold values 2..=6, all built by `+1` gates.
    assert_eq!(context.get(Var { idx: 4 }), M31::from(2u32).into());
    assert_eq!(context.get(Var { idx: 5 }), M31::from(3u32).into());
    assert_eq!(context.get(Var { idx: 6 }), M31::from(4u32).into());
    assert_eq!(context.get(Var { idx: 7 }), M31::from(5u32).into());
    assert_eq!(context.get(Var { idx: 8 }), M31::from(6u32).into());
    // Limb 9 is past the gap, rebuilt via base decomposition.
    assert_eq!(context.get(Var { idx: 16 }), M31::from(9u32).into());
    // The QM31 constant is assembled into its reserved var [3].
    assert_eq!(context.get(Var { idx: 3 }), qm31_from_u32s(2, 5, 6, 9));

    context.circuit.check_yields();
    context.validate_circuit();
}
