use crate::cairo_air::statement::{
    CasmState, PubMemoryM31Value, PubMemoryValue, PublicData, PublicMemory, SegmentRange,
    public_logup_sum,
};
use crate::circuits::context::{TraceContext, Var};
use crate::circuits::ivalue::qm31_from_u32s;

#[test]
fn test_public_logup_sum() {
    let mut context = TraceContext::default();

    // Create dummy lookup elements (interaction elements)
    // Matching LookupElementsDummyImpl::dummy() from Cairo1:
    // z = qm31_const<1, 2, 3, 4>(), alpha = One::one()
    let interaction_z = context.constant(qm31_from_u32s(1, 2, 3, 4));
    let interaction_alpha = context.one(); // One::one() = QM31::one() = qm31_from_u32s(1, 0, 0, 0)
    let interaction_elements = [interaction_z, interaction_alpha];

    // Create initial state: CasmState uses { pc, ap, fp } order
    // Cairo1 has: pc=1, ap=1336, fp=1336
    let initial_state = CasmState {
        pc: context.constant(qm31_from_u32s(1, 0, 0, 0)),
        ap: context.constant(qm31_from_u32s(1336, 0, 0, 0)),
        fp: context.constant(qm31_from_u32s(1336, 0, 0, 0)),
    };

    // Create final state: CasmState uses { pc, ap, fp } order
    // Cairo1 has: pc=5, ap=2520, fp=1336
    let final_state = CasmState {
        pc: context.constant(qm31_from_u32s(5, 0, 0, 0)),
        ap: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
        fp: context.constant(qm31_from_u32s(1336, 0, 0, 0)),
    };

    // Create segment ranges matching the Cairo1 test
    // The 11 segments are: output, pedersen, range_check_128, ecdsa, bitwise, ec_op, keccak,
    // poseidon, range_check_96, add_mod, mul_mod
    let segment_ranges = [
        // output: id=228, value=2520
        SegmentRange {
            start: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
            end: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
        },
        // pedersen: id=228, value=2520
        SegmentRange {
            start: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
            end: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
        },
        // range_check_128: id=228, value=2520
        SegmentRange {
            start: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
            end: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
        },
        // ecdsa: id=5, value=0
        SegmentRange {
            start: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(5, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(0, 0, 0, 0)),
            },
            end: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(5, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(0, 0, 0, 0)),
            },
        },
        // bitwise: id=228, value=2520
        SegmentRange {
            start: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
            end: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
        },
        // ec_op: id=5, value=0
        SegmentRange {
            start: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(5, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(0, 0, 0, 0)),
            },
            end: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(5, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(0, 0, 0, 0)),
            },
        },
        // keccak: id=5, value=0
        SegmentRange {
            start: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(5, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(0, 0, 0, 0)),
            },
            end: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(5, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(0, 0, 0, 0)),
            },
        },
        // poseidon: id=228, value=2520
        SegmentRange {
            start: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
            end: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
        },
        // range_check_96: id=228, value=2520
        SegmentRange {
            start: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
            end: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
        },
        // add_mod: id=228, value=2520
        SegmentRange {
            start: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
            end: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
        },
        // mul_mod: id=228, value=2520
        SegmentRange {
            start: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
            end: PubMemoryM31Value {
                id: context.constant(qm31_from_u32s(228, 0, 0, 0)),
                value: context.constant(qm31_from_u32s(2520, 0, 0, 0)),
            },
        },
    ];

    // Create safe_call_ids: [227, 5]
    let safe_call_id_0 = context.constant(qm31_from_u32s(227, 0, 0, 0));
    let safe_call_id_1 = context.constant(qm31_from_u32s(5, 0, 0, 0));
    let safe_call_ids = [safe_call_id_0, safe_call_id_1];

    // Create empty output vector
    let output: Vec<PubMemoryValue<Var>> = vec![];

    let public_memory = PublicMemory { segement_ranges: segment_ranges, safe_call_ids, output };

    let public_data = PublicData { initial_state, final_state, public_memory };

    // Call public_logup_sum
    let result = public_logup_sum(&mut context, &public_data, interaction_elements);

    // The result should be a valid variable
    let result_value = context.get(result);
    context.mark_as_unused(result);

    // Expected from Cairo1: qm31_const<1553510278, 1990190377, 918519607, 1802790922>
    let expected = qm31_from_u32s(1553510278, 1990190377, 918519607, 1802790922);

    assert_eq!(result_value, expected);

    // Validate the circuit
    context.check_vars_used();
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}
