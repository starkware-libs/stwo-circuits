use itertools::Itertools;

use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::test_utils::packed_values;
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::select_queries::{get_select_queries_input, select_queries};

#[test]
fn test_full_select_queries_regression() {
    let mut context = TraceContext::default();

    let init_digest = [
        qm31_from_u32s(271333035, 1833401714, 819175623, 1270120203),
        qm31_from_u32s(1921341900, 364315769, 339695133, 365135865),
    ];
    let mut channel = Channel::from_digest(&mut context, init_digest);

    const N_QUERIES: usize = 3;
    const LOG_DOMAIN_SIZE: usize = 5;

    let select_queries_input = get_select_queries_input(&mut context, &mut channel, N_QUERIES);

    assert_eq!(select_queries_input.len(), N_QUERIES);
    assert_eq!(
        packed_values(&context, &select_queries_input),
        [qm31_from_u32s(577837367, 1394565488, 1540262994, 293692251)]
    );

    let queries = select_queries(&mut context, &select_queries_input, LOG_DOMAIN_SIZE);

    assert_eq!(queries.points.x.len(), N_QUERIES);
    assert_eq!(queries.points.y.len(), N_QUERIES);
    assert_eq!(
        packed_values(&context, &queries.points.x),
        vec![qm31_from_u32s(567259857, 1952787376, 194696271, 1133522282)]
    );
    assert_eq!(
        packed_values(&context, &queries.points.y),
        vec![qm31_from_u32s(194696271, 1580223790, 567259857, 280947147)]
    );
    assert_eq!(
        queries.bits.iter().map(|bits| packed_values(&context, bits)).collect_vec(),
        vec![
            vec![qm31_from_u32s(1, 0, 0, 1)],
            vec![qm31_from_u32s(1, 0, 1, 1)],
            vec![qm31_from_u32s(1, 0, 0, 0)],
            vec![qm31_from_u32s(0, 0, 0, 1)],
            vec![qm31_from_u32s(1, 1, 1, 1)],
        ]
    );

    context.circuit.check(context.values()).unwrap();
}
