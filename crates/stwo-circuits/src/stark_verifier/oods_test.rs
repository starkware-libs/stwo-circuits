use stwo::core::circle::CirclePoint;

use crate::circuits::context::TraceContext;
use crate::circuits::ops::Guess;
use crate::stark_verifier::oods::extract_expected_composition_eval;
// use crate::stark_verifier::circle::CirclePoint;
use crate::circuits::ivalue::qm31_from_u32s;

#[test]
fn extract_expected_composition_eval_regression() {
    let mut context = TraceContext::default();

    let composition_eval_at_oods = [
        qm31_from_u32s(1508389461, 2095170364, 1242839621, 121914987),
        qm31_from_u32s(2074471118, 525791636, 1741315353, 560542608),
        qm31_from_u32s(1544603421, 1313779258, 1591174380, 2142352248),
        qm31_from_u32s(376285896, 1645064251, 1972412846, 145104793),
        qm31_from_u32s(425315367, 0, 0, 0),
        qm31_from_u32s(1670393541, 0, 0, 0),
        qm31_from_u32s(833801100, 0, 0, 0),
        qm31_from_u32s(374213131, 0, 0, 0),
    ]
    .guess(&mut context);

    let oods_point = CirclePoint {
        x: qm31_from_u32s(1343313724, 1951183646, 1685075959, 888698585).guess(&mut context),
        y: qm31_from_u32s(674655034, 1516640953, 569857337, 1549701521).guess(&mut context),
    };

    let log_evaluation_domain_size = 5;
    let expected_composition_eval = extract_expected_composition_eval(
        &mut context,
        &composition_eval_at_oods,
        oods_point,
        log_evaluation_domain_size,
    );
    assert_eq!(
        context.get(expected_composition_eval),
        qm31_from_u32s(443798542, 633915785, 595028408, 165661052)
    );

    context.circuit.check(context.values()).unwrap();
}
