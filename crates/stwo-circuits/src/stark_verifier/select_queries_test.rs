use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::test_utils::packed_values;
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::select_queries::get_select_queries_input;

#[test]
fn test_full_select_queries_regression() {
    let mut context = TraceContext::default();

    let init_digest = [
        qm31_from_u32s(271333035, 1833401714, 819175623, 1270120203),
        qm31_from_u32s(1921341900, 364315769, 339695133, 365135865),
    ];
    let mut channel = Channel::from_digest(&mut context, init_digest);

    const N_QUERIES: usize = 3;

    let select_queries_input = get_select_queries_input(&mut context, &mut channel, N_QUERIES);

    assert_eq!(select_queries_input.len(), N_QUERIES);
    assert_eq!(
        packed_values(&context, &select_queries_input),
        [qm31_from_u32s(577837367, 1394565488, 1540262994, 293692251),]
    );

    // TODO(lior): Once implemented, test `select_queries` on `select_queries_input`.

    context.circuit.check(context.values()).unwrap();
}
