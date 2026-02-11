// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 1;
pub const N_INTERACTION_COLUMNS: usize = 4;

pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [multiplicity_0] = input.try_into().unwrap();
    let poseidon_round_keys_0 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_0".to_owned() });
    let poseidon_round_keys_1 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_1".to_owned() });
    let poseidon_round_keys_10 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_10".to_owned() });
    let poseidon_round_keys_11 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_11".to_owned() });
    let poseidon_round_keys_12 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_12".to_owned() });
    let poseidon_round_keys_13 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_13".to_owned() });
    let poseidon_round_keys_14 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_14".to_owned() });
    let poseidon_round_keys_15 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_15".to_owned() });
    let poseidon_round_keys_16 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_16".to_owned() });
    let poseidon_round_keys_17 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_17".to_owned() });
    let poseidon_round_keys_18 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_18".to_owned() });
    let poseidon_round_keys_19 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_19".to_owned() });
    let poseidon_round_keys_2 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_2".to_owned() });
    let poseidon_round_keys_20 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_20".to_owned() });
    let poseidon_round_keys_21 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_21".to_owned() });
    let poseidon_round_keys_22 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_22".to_owned() });
    let poseidon_round_keys_23 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_23".to_owned() });
    let poseidon_round_keys_24 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_24".to_owned() });
    let poseidon_round_keys_25 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_25".to_owned() });
    let poseidon_round_keys_26 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_26".to_owned() });
    let poseidon_round_keys_27 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_27".to_owned() });
    let poseidon_round_keys_28 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_28".to_owned() });
    let poseidon_round_keys_29 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_29".to_owned() });
    let poseidon_round_keys_3 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_3".to_owned() });
    let poseidon_round_keys_4 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_4".to_owned() });
    let poseidon_round_keys_5 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_5".to_owned() });
    let poseidon_round_keys_6 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_6".to_owned() });
    let poseidon_round_keys_7 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_7".to_owned() });
    let poseidon_round_keys_8 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_8".to_owned() });
    let poseidon_round_keys_9 = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "poseidon_round_keys_9".to_owned() });
    let seq_6 = acc.get_preprocessed_column(&PreProcessedColumnId { id: "seq_6".to_owned() });

    // Yield PoseidonRoundKeys.
    let tuple_0 = &[
        eval!(context, 1024310512),
        eval!(context, seq_6),
        eval!(context, poseidon_round_keys_0),
        eval!(context, poseidon_round_keys_1),
        eval!(context, poseidon_round_keys_2),
        eval!(context, poseidon_round_keys_3),
        eval!(context, poseidon_round_keys_4),
        eval!(context, poseidon_round_keys_5),
        eval!(context, poseidon_round_keys_6),
        eval!(context, poseidon_round_keys_7),
        eval!(context, poseidon_round_keys_8),
        eval!(context, poseidon_round_keys_9),
        eval!(context, poseidon_round_keys_10),
        eval!(context, poseidon_round_keys_11),
        eval!(context, poseidon_round_keys_12),
        eval!(context, poseidon_round_keys_13),
        eval!(context, poseidon_round_keys_14),
        eval!(context, poseidon_round_keys_15),
        eval!(context, poseidon_round_keys_16),
        eval!(context, poseidon_round_keys_17),
        eval!(context, poseidon_round_keys_18),
        eval!(context, poseidon_round_keys_19),
        eval!(context, poseidon_round_keys_20),
        eval!(context, poseidon_round_keys_21),
        eval!(context, poseidon_round_keys_22),
        eval!(context, poseidon_round_keys_23),
        eval!(context, poseidon_round_keys_24),
        eval!(context, poseidon_round_keys_25),
        eval!(context, poseidon_round_keys_26),
        eval!(context, poseidon_round_keys_27),
        eval!(context, poseidon_round_keys_28),
        eval!(context, poseidon_round_keys_29),
    ];
    let numerator_0 = eval!(context, -(multiplicity_0));
    acc.add_to_relation(context, numerator_0, tuple_0);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &ComponentData<'_>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns, context, component_data, acc);
        // Verify this component has 2 ** 6 rows
        let size_bit = component_data.get_n_instances_bit(context, 6);
        eq(context, size_bit, context.one());
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &RELATION_USES_PER_ROW
    }
}
