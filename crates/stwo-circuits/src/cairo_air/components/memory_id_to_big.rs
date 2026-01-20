use crate::cairo_air::components::prelude::*;
use crate::cairo_air::preprocessed_columns::MAX_SEQUENCE_LOG_SIZE;
use crate::stark_verifier::verify::MAX_TRACE_SIZE_BITS;

const LARGE_MEMORY_VALUE_ID_BASE: u32 = 0x40000000; // 2^30.
const ID_TO_BIG_MAX_ROWS: u32 = 1 << MAX_SEQUENCE_LOG_SIZE;
pub const N_TRACE_COLUMNS: usize = 29;
pub const N_INTERACTION_COLUMNS: usize = 32;

pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
    index: u32,
) {
    let _ = component_data;
    let [
        memory_id_to_big_output_col0,
        memory_id_to_big_output_col1,
        memory_id_to_big_output_col2,
        memory_id_to_big_output_col3,
        memory_id_to_big_output_col4,
        memory_id_to_big_output_col5,
        memory_id_to_big_output_col6,
        memory_id_to_big_output_col7,
        memory_id_to_big_output_col8,
        memory_id_to_big_output_col9,
        memory_id_to_big_output_col10,
        memory_id_to_big_output_col11,
        memory_id_to_big_output_col12,
        memory_id_to_big_output_col13,
        memory_id_to_big_output_col14,
        memory_id_to_big_output_col15,
        memory_id_to_big_output_col16,
        memory_id_to_big_output_col17,
        memory_id_to_big_output_col18,
        memory_id_to_big_output_col19,
        memory_id_to_big_output_col20,
        memory_id_to_big_output_col21,
        memory_id_to_big_output_col22,
        memory_id_to_big_output_col23,
        memory_id_to_big_output_col24,
        memory_id_to_big_output_col25,
        memory_id_to_big_output_col26,
        memory_id_to_big_output_col27,
        multiplicity_0,
    ] = input.try_into().unwrap();
    let seq = seq_of_component_size(context, component_data, acc);

    range_check_mem_value_n_28::accumulate_constraints(
        &[
            eval!(context, memory_id_to_big_output_col0),
            eval!(context, memory_id_to_big_output_col1),
            eval!(context, memory_id_to_big_output_col2),
            eval!(context, memory_id_to_big_output_col3),
            eval!(context, memory_id_to_big_output_col4),
            eval!(context, memory_id_to_big_output_col5),
            eval!(context, memory_id_to_big_output_col6),
            eval!(context, memory_id_to_big_output_col7),
            eval!(context, memory_id_to_big_output_col8),
            eval!(context, memory_id_to_big_output_col9),
            eval!(context, memory_id_to_big_output_col10),
            eval!(context, memory_id_to_big_output_col11),
            eval!(context, memory_id_to_big_output_col12),
            eval!(context, memory_id_to_big_output_col13),
            eval!(context, memory_id_to_big_output_col14),
            eval!(context, memory_id_to_big_output_col15),
            eval!(context, memory_id_to_big_output_col16),
            eval!(context, memory_id_to_big_output_col17),
            eval!(context, memory_id_to_big_output_col18),
            eval!(context, memory_id_to_big_output_col19),
            eval!(context, memory_id_to_big_output_col20),
            eval!(context, memory_id_to_big_output_col21),
            eval!(context, memory_id_to_big_output_col22),
            eval!(context, memory_id_to_big_output_col23),
            eval!(context, memory_id_to_big_output_col24),
            eval!(context, memory_id_to_big_output_col25),
            eval!(context, memory_id_to_big_output_col26),
            eval!(context, memory_id_to_big_output_col27),
        ],
        context,
        component_data,
        acc,
    );

    // Yield MemoryIdToBig.
    let offset = LARGE_MEMORY_VALUE_ID_BASE + index * ID_TO_BIG_MAX_ROWS;
    let tuple_1 = &[
        eval!(context, 1662111297),
        eval!(context, (seq) + (context.constant(offset.into()))),
        eval!(context, memory_id_to_big_output_col0),
        eval!(context, memory_id_to_big_output_col1),
        eval!(context, memory_id_to_big_output_col2),
        eval!(context, memory_id_to_big_output_col3),
        eval!(context, memory_id_to_big_output_col4),
        eval!(context, memory_id_to_big_output_col5),
        eval!(context, memory_id_to_big_output_col6),
        eval!(context, memory_id_to_big_output_col7),
        eval!(context, memory_id_to_big_output_col8),
        eval!(context, memory_id_to_big_output_col9),
        eval!(context, memory_id_to_big_output_col10),
        eval!(context, memory_id_to_big_output_col11),
        eval!(context, memory_id_to_big_output_col12),
        eval!(context, memory_id_to_big_output_col13),
        eval!(context, memory_id_to_big_output_col14),
        eval!(context, memory_id_to_big_output_col15),
        eval!(context, memory_id_to_big_output_col16),
        eval!(context, memory_id_to_big_output_col17),
        eval!(context, memory_id_to_big_output_col18),
        eval!(context, memory_id_to_big_output_col19),
        eval!(context, memory_id_to_big_output_col20),
        eval!(context, memory_id_to_big_output_col21),
        eval!(context, memory_id_to_big_output_col22),
        eval!(context, memory_id_to_big_output_col23),
        eval!(context, memory_id_to_big_output_col24),
        eval!(context, memory_id_to_big_output_col25),
        eval!(context, memory_id_to_big_output_col26),
        eval!(context, memory_id_to_big_output_col27),
    ];
    let numerator_1 = eval!(context, -(multiplicity_0));
    acc.add_to_relation(context, numerator_1, tuple_1);
}

pub struct Component {
    // The trace can contain multiple memory_id_to_big components, each responsible
    // for a range of IDs. The index differentiates between them and sets the ID range
    // for each.
    pub index: u32,
}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &ComponentData<'_>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(
            component_data.trace_columns,
            context,
            component_data,
            acc,
            self.index,
        );

        // Verify size <= ID_TO_BIG_MAX_ROWS (otherwise it will overlap with the next component)
        for bit_pos in (MAX_SEQUENCE_LOG_SIZE + 1)..MAX_TRACE_SIZE_BITS {
            let bit = component_data.get_n_instances_bit(context, bit_pos as usize);
            eq(context, bit, context.zero());
        }
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }
}
