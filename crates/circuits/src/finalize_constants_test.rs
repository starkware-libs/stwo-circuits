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
fn test_plus_one_chain_topology() {
    let mut context = TraceContext::default();
    context.constant(M31::from(2u32).into());
    context.constant(M31::from(4u32).into());
    // Constants 2 and 4 are both below `min_base = 6`, so they're built by the chain regardless;
    // nothing continues the run past 6 (no limb at 7), so the base is exactly `min_base`.
    let m31_values = HashSet::from([0, 1, 2, 4]);
    assert_eq!(find_max_consecutive(&m31_values, 6), 6);
    // With base 6, the chain runs 2..=6.
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
    // With no limb above `min_base`, the base is exactly `min_base`.
    assert_eq!(find_max_consecutive(&HashSet::from([0]), 2), 2);
    // Limbs consecutive above `min_base` extend the base; the run stops at the first gap.
    assert_eq!(find_max_consecutive(&HashSet::from([3, 4, 5, 100]), 2), 5);
    // A gap immediately above `min_base` leaves the base at `min_base`.
    assert_eq!(find_max_consecutive(&HashSet::from([4, 5]), 2), 2);
    // Limbs at or below `min_base` are irrelevant — they're built by the chain regardless.
    assert_eq!(find_max_consecutive(&HashSet::from([0, 1, 5, 6]), 3), 3);
    // A gap *below* `min_base` no longer stops the count: the run above `min_base` still extends
    // it.
    assert_eq!(find_max_consecutive(&HashSet::from([0, 1, 3, 4, 5, 6]), 3), 6);
}

#[test]
fn test_qm31_limbs_extend_plus_one_chain() {
    let mut context = TraceContext::default();
    // A single QM31 constant whose limbs (2, 3, 4, 5) continue the consecutive run above
    // `min_base = 2`. No *pure* M31 constant requires 3, 4 or 5, but because the limbs are fed into
    // `find_max_consecutive` — which walks up from `min_base` — the limbs 3, 4, 5 grow the base to
    // 5, so the `+1` chain runs 2..=5 and each limb is produced by the chain rather than by base
    // decomposition.
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

    // The chain populated vars [4]..=[7] for values 2..=5, all built by `+1` Add gates above.
    assert_eq!(context.get(Var { idx: 4 }), M31::from(2u32).into());
    assert_eq!(context.get(Var { idx: 5 }), M31::from(3u32).into());
    assert_eq!(context.get(Var { idx: 6 }), M31::from(4u32).into());
    assert_eq!(context.get(Var { idx: 7 }), M31::from(5u32).into());
    // The QM31 constant itself is assembled into its reserved var [3].
    assert_eq!(context.get(Var { idx: 3 }), qm31_from_u32s(2, 3, 4, 5));

    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_limbs_extend_chain_across_min_base_gap() {
    let mut context = TraceContext::default();
    // A single QM31 constant with limbs {2, 5, 6, 9} and `min_base = 4`. Together with the
    // always-present `zero`, `one` and `u` constants (which contribute limbs 0 and 1), the full
    // limb set fed into `find_max_consecutive` is {0, 1, 2, 5, 6, 9}, with a gap on each side of
    // `min_base`:
    //   * the gap at 3 *below* `min_base` is ignored — `find_max_consecutive` walks up from
    //     `min_base` (not from 0), so the missing 3 doesn't cut the run short;
    //   * the gap at 7 *above* `min_base` stops the run, so the base is 6 (not 9).
    // Hence limbs 5 and 6 are produced by the `+1` chain (which runs 2..=6), while limb 9 lies past
    // the gap and is built by base-6 decomposition (9 = 1*6 + 3) instead.
    let m31_limbs = HashSet::from([0, 1, 2, 5, 6, 9]);
    assert_eq!(find_max_consecutive(&m31_limbs, 4), 6);
    context.constant(qm31_from_u32s(2, 5, 6, 9));
    finalize_constants_with_min_base(&mut context, 4);

    // The `+1` chain runs 2..=6 (vars [4]..=[8]); limb 9 is instead rebuilt from the base as
    // `1*6 + 3` in var [16], reusing chain value 6 (var [8]).
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

    // The `+1` chain populated vars [4]..=[8] with values 2..=6, all built by `+1` Add gates.
    assert_eq!(context.get(Var { idx: 4 }), M31::from(2u32).into());
    assert_eq!(context.get(Var { idx: 5 }), M31::from(3u32).into());
    assert_eq!(context.get(Var { idx: 6 }), M31::from(4u32).into());
    assert_eq!(context.get(Var { idx: 7 }), M31::from(5u32).into());
    assert_eq!(context.get(Var { idx: 8 }), M31::from(6u32).into());
    // Limb 9 lies past the gap, so it's not in the chain; it's rebuilt via base decomposition.
    assert_eq!(context.get(Var { idx: 16 }), M31::from(9u32).into());
    // The QM31 constant itself is assembled into its reserved var [3].
    assert_eq!(context.get(Var { idx: 3 }), qm31_from_u32s(2, 5, 6, 9));

    context.circuit.check_yields();
    context.validate_circuit();
}
