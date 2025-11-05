use crate::circuits::blake::HashValue;
use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::stats::Stats;

use super::Channel;

#[test]
fn test_mix_commitment_regression() {
    let mut context = TraceContext::default();

    let mut channel = Channel::new(&mut context);
    let root0 = HashValue(
        context.new_var(qm31_from_u32s(637418335, 1672023491, 980858689, 607764934)),
        context.new_var(qm31_from_u32s(386900718, 430556311, 1187803054, 669301442)),
    );
    let root1 = HashValue(
        context.new_var(qm31_from_u32s(1477561267, 1244239078, 1979857528, 1316512771)),
        context.new_var(qm31_from_u32s(490980261, 2016799283, 79573118, 1350641448)),
    );
    channel.mix_commitment(&mut context, root0);
    let digest0 = channel.digest;
    channel.mix_commitment(&mut context, root1);
    let digest1 = channel.digest;

    assert_eq!(
        context.get(digest0.0),
        qm31_from_u32s(1668664816, 1251290000, 263177925, 722663798)
    );
    assert_eq!(context.get(digest0.1), qm31_from_u32s(484105836, 140598027, 679686738, 1985395078));
    assert_eq!(
        context.get(digest1.0),
        qm31_from_u32s(800533588, 1994201536, 2099095392, 678020158)
    );
    assert_eq!(context.get(digest1.1), qm31_from_u32s(1950435309, 1607451911, 2421030, 565867237));

    assert_eq!(context.stats, Stats { blake_updates: 2, guess: 2, ..Stats::default() });

    context.circuit.check(context.values()).unwrap();
}
