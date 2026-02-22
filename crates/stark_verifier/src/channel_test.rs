use rstest::rstest;
use stwo::core::channel::Blake2sM31Channel as NativeBlake2sM31Channel;
use stwo::core::channel::Channel as NativeChannelTrait;

use circuits::blake::HashValue;
use circuits::context::TraceContext;
use circuits::ivalue::qm31_from_u32s;
use circuits::stats::Stats;

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

    context.validate_circuit();
}

#[test]
fn test_mix_qm31s_regression() {
    let mut context = TraceContext::default();

    let init_digest = [
        qm31_from_u32s(266526289, 1341429509, 1126614795, 1001621831),
        qm31_from_u32s(1024638884, 1857778419, 1763024470, 1859929979),
    ];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let felts = [
        context.new_var(qm31_from_u32s(1, 0, 0, 0)),
        context.new_var(qm31_from_u32s(485399786, 1255952693, 1939438763, 1561715227)),
        context.new_var(qm31_from_u32s(1757357815, 8864493, 674769946, 1715431414)),
        context.new_var(qm31_from_u32s(1148846901, 1519172202, 357767101, 2129853554)),
        context.new_var(qm31_from_u32s(0, 0, 0, 0)),
        context.new_var(qm31_from_u32s(0, 0, 0, 0)),
        context.new_var(qm31_from_u32s(0, 0, 0, 0)),
    ];
    channel.mix_qm31s(&mut context, felts);

    assert_eq!(
        context.get(channel.digest.0),
        qm31_from_u32s(1186703962, 1584594219, 633548839, 1510969779)
    );
    assert_eq!(
        context.get(channel.digest.1),
        qm31_from_u32s(1524867388, 1224019906, 1564199416, 388718964)
    );
    assert_eq!(channel.n_draws, 0);

    context.validate_circuit();
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

    context.validate_circuit();
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

    context.validate_circuit();
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

    context.validate_circuit();
}

#[test]
fn test_draw_qm31_matches_native_channel() {
    let mut context = TraceContext::default();
    let mut our = Channel::new(&mut context);
    let mut native = NativeBlake2sM31Channel::default();

    let salt = qm31_from_u32s(12345, 0, 0, 0);
    let salt_var = context.new_var(salt);
    our.mix_qm31s(&mut context, [salt_var]);
    native.mix_felts(&[salt]);

    let our_draw = our.draw_qm31(&mut context);
    let native_draw = native.draw_secure_felt();
    assert_eq!(context.get(our_draw), native_draw);

    context.validate_circuit();
}

#[test]
fn test_draw_qm31_matches_native_from_fixed_digest() {
    let mut context = TraceContext::default();
    let init_digest = [
        qm31_from_u32s(1965230632, 1839411032, 1501942319, 262181679),
        qm31_from_u32s(1779914487, 909827358, 1760026750, 1554912623),
    ];
    let mut our = Channel::from_digest(&mut context, init_digest);
    let our_draw = our.draw_qm31(&mut context);
    let mut input = Vec::new();
    for w in [
        init_digest[0].0.0.0,
        init_digest[0].0.1.0,
        init_digest[0].1.0.0,
        init_digest[0].1.1.0,
        init_digest[1].0.0.0,
        init_digest[1].0.1.0,
        init_digest[1].1.0.0,
        init_digest[1].1.1.0,
    ] {
        input.extend_from_slice(&w.to_le_bytes());
    }
    input.extend_from_slice(&0u32.to_le_bytes());
    input.push(0u8);
    let digest = stwo::core::vcs::blake2_hash::Blake2sHasherGeneric::<true>::hash(&input).0;
    let native_draw = qm31_from_u32s(
        u32::from_le_bytes(digest[0..4].try_into().unwrap()),
        u32::from_le_bytes(digest[4..8].try_into().unwrap()),
        u32::from_le_bytes(digest[8..12].try_into().unwrap()),
        u32::from_le_bytes(digest[12..16].try_into().unwrap()),
    );
    assert_eq!(context.get(our_draw), native_draw);

    context.validate_circuit();
}

#[rstest]
#[case::success(10, 1524, true)]
#[case::wrong_n_bits0(11, 1524, false)]
#[case::wrong_n_bits1(9, 1524, false)]
#[case::wrong_nonce0(10, 1523, false)]
#[case::wrong_nonce1(10, 1525, false)]
fn test_proof_of_work_regression(#[case] n_bits: u32, #[case] nonce: u32, #[case] success: bool) {
    let mut context = TraceContext::default();

    let init_digest = [
        qm31_from_u32s(968886948, 725376924, 836084817, 484428276),
        qm31_from_u32s(1805658819, 300032261, 172116750, 994058243),
    ];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let nonce = context.new_var(qm31_from_u32s(nonce, 0, 0, 0));
    channel.proof_of_work(&mut context, n_bits, nonce);

    assert_eq!(context.is_circuit_valid(), success);

    if success {
        assert_eq!(
            context.get(channel.digest.0),
            qm31_from_u32s(271333035, 1833401714, 819175623, 1270120203)
        );
        assert_eq!(
            context.get(channel.digest.1),
            qm31_from_u32s(1921341900, 364315769, 339695133, 365135865)
        );
    }
}
