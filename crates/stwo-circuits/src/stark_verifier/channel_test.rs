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

#[test]
fn test_draw_qm31_regression() {
    let mut context = TraceContext::default();

    let init_digest = [
        qm31_from_u32s(800533588, 1994201536, 2099095392, 678020158),
        qm31_from_u32s(1950435309, 1607451911, 2421030, 565867237),
    ];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let res = channel.draw_qm31(&mut context);
    assert_eq!(context.get(res), qm31_from_u32s(1511219767, 1680262446, 557532573, 1741612347));

    let res2 = channel.draw_qm31(&mut context);
    assert_eq!(context.get(res2), qm31_from_u32s(1010544646, 1898030754, 53928552, 587440252));

    context.circuit.check(context.values()).unwrap();
}

#[test]
fn test_draw_two_qm31s_regression() {
    let mut context = TraceContext::default();

    let init_digest = [
        qm31_from_u32s(800533588, 1994201536, 2099095392, 678020158),
        qm31_from_u32s(1950435309, 1607451911, 2421030, 565867237),
    ];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let res = channel.draw_two_qm31s(&mut context);
    assert_eq!(context.get(res[0]), qm31_from_u32s(1511219767, 1680262446, 557532573, 1741612347));
    assert_eq!(context.get(res[1]), qm31_from_u32s(1790671546, 1908058358, 2021264888, 1820912939));

    let res2 = channel.draw_two_qm31s(&mut context);
    assert_eq!(context.get(res2[0]), qm31_from_u32s(1010544646, 1898030754, 53928552, 587440252));
    assert_eq!(context.get(res2[1]), qm31_from_u32s(868459281, 1035649663, 299576823, 539722878));

    context.circuit.check(context.values()).unwrap();
}

#[test]
fn test_draw_point_regression() {
    let mut context = TraceContext::default();

    let init_digest = [
        qm31_from_u32s(2072130922, 1322677507, 1508142866, 1010842681),
        qm31_from_u32s(967226388, 1861793490, 1980108433, 243066861),
    ];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let pt = channel.draw_point(&mut context);

    assert_eq!(context.get(pt.x), qm31_from_u32s(1343313724, 1951183646, 1685075959, 888698585));
    assert_eq!(context.get(pt.y), qm31_from_u32s(674655034, 1516640953, 569857337, 1549701521));

    context.circuit.check(context.values()).unwrap();
}
