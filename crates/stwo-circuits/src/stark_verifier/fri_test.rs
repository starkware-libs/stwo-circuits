use crate::circuits::blake::HashValue;
use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::ops::Guess;
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::fri_proof::FriCommitProof;

use super::fri_commit;

#[test]
fn test_fri_commit_regression() {
    let mut context = TraceContext::default();

    let init_digest = [
        qm31_from_u32s(375224163, 1270824854, 44060607, 991529112),
        qm31_from_u32s(1068130924, 1630210318, 1632828025, 1983481471),
    ];

    let mut channel = Channel::from_digest(&mut context, init_digest);

    let proof = FriCommitProof {
        layer_commitments: vec![
            HashValue(
                qm31_from_u32s(370372302, 356302922, 2040089875, 232934191),
                qm31_from_u32s(1279830905, 1240360672, 1788604172, 465814885),
            ),
            HashValue(
                qm31_from_u32s(1558212721, 609186473, 1554074721, 1956195301),
                qm31_from_u32s(1243917617, 135256448, 1193318416, 1792104990),
            ),
            HashValue(
                qm31_from_u32s(1017503040, 1411053946, 1805475392, 1906875756),
                qm31_from_u32s(2035075097, 617472393, 571220918, 1577790110),
            ),
            HashValue(
                qm31_from_u32s(1290083578, 670256590, 203247471, 492011214),
                qm31_from_u32s(353269841, 1619070080, 770215254, 1663098736),
            ),
        ],
        last_layer_coefs: vec![qm31_from_u32s(1802004671, 1018373769, 131996621, 1575090881)],
    };
    let proof_vars = proof.guess(&mut context);

    let alphas = fri_commit(&mut context, &mut channel, &proof_vars);
    assert_eq!(alphas.len(), 4);
    assert_eq!(context.get(alphas[0]), qm31_from_u32s(2047550788, 23895068, 1676134944, 263598239));
    assert_eq!(
        context.get(alphas[1]),
        qm31_from_u32s(1988032363, 1739489633, 826507892, 1797301629)
    );
    assert_eq!(
        context.get(alphas[2]),
        qm31_from_u32s(1957504342, 848565442, 1129943791, 1937962621)
    );
    assert_eq!(context.get(alphas[3]), qm31_from_u32s(1748651123, 2133979933, 232524784, 85583628));

    assert_eq!(
        context.get(channel.digest().0),
        qm31_from_u32s(968886948, 725376924, 836084817, 484428276)
    );
    assert_eq!(
        context.get(channel.digest().1),
        qm31_from_u32s(1805658819, 300032261, 172116750, 994058243)
    );

    context.circuit.check(context.values()).unwrap();
}
