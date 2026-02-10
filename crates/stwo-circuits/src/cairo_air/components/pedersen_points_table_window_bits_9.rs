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
    let pedersen_points_small_0 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_0".to_owned(),
    });
    let pedersen_points_small_1 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_1".to_owned(),
    });
    let pedersen_points_small_10 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_10".to_owned(),
    });
    let pedersen_points_small_11 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_11".to_owned(),
    });
    let pedersen_points_small_12 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_12".to_owned(),
    });
    let pedersen_points_small_13 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_13".to_owned(),
    });
    let pedersen_points_small_14 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_14".to_owned(),
    });
    let pedersen_points_small_15 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_15".to_owned(),
    });
    let pedersen_points_small_16 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_16".to_owned(),
    });
    let pedersen_points_small_17 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_17".to_owned(),
    });
    let pedersen_points_small_18 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_18".to_owned(),
    });
    let pedersen_points_small_19 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_19".to_owned(),
    });
    let pedersen_points_small_2 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_2".to_owned(),
    });
    let pedersen_points_small_20 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_20".to_owned(),
    });
    let pedersen_points_small_21 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_21".to_owned(),
    });
    let pedersen_points_small_22 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_22".to_owned(),
    });
    let pedersen_points_small_23 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_23".to_owned(),
    });
    let pedersen_points_small_24 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_24".to_owned(),
    });
    let pedersen_points_small_25 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_25".to_owned(),
    });
    let pedersen_points_small_26 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_26".to_owned(),
    });
    let pedersen_points_small_27 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_27".to_owned(),
    });
    let pedersen_points_small_28 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_28".to_owned(),
    });
    let pedersen_points_small_29 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_29".to_owned(),
    });
    let pedersen_points_small_3 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_3".to_owned(),
    });
    let pedersen_points_small_30 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_30".to_owned(),
    });
    let pedersen_points_small_31 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_31".to_owned(),
    });
    let pedersen_points_small_32 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_32".to_owned(),
    });
    let pedersen_points_small_33 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_33".to_owned(),
    });
    let pedersen_points_small_34 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_34".to_owned(),
    });
    let pedersen_points_small_35 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_35".to_owned(),
    });
    let pedersen_points_small_36 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_36".to_owned(),
    });
    let pedersen_points_small_37 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_37".to_owned(),
    });
    let pedersen_points_small_38 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_38".to_owned(),
    });
    let pedersen_points_small_39 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_39".to_owned(),
    });
    let pedersen_points_small_4 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_4".to_owned(),
    });
    let pedersen_points_small_40 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_40".to_owned(),
    });
    let pedersen_points_small_41 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_41".to_owned(),
    });
    let pedersen_points_small_42 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_42".to_owned(),
    });
    let pedersen_points_small_43 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_43".to_owned(),
    });
    let pedersen_points_small_44 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_44".to_owned(),
    });
    let pedersen_points_small_45 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_45".to_owned(),
    });
    let pedersen_points_small_46 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_46".to_owned(),
    });
    let pedersen_points_small_47 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_47".to_owned(),
    });
    let pedersen_points_small_48 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_48".to_owned(),
    });
    let pedersen_points_small_49 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_49".to_owned(),
    });
    let pedersen_points_small_5 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_5".to_owned(),
    });
    let pedersen_points_small_50 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_50".to_owned(),
    });
    let pedersen_points_small_51 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_51".to_owned(),
    });
    let pedersen_points_small_52 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_52".to_owned(),
    });
    let pedersen_points_small_53 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_53".to_owned(),
    });
    let pedersen_points_small_54 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_54".to_owned(),
    });
    let pedersen_points_small_55 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_55".to_owned(),
    });
    let pedersen_points_small_6 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_6".to_owned(),
    });
    let pedersen_points_small_7 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_7".to_owned(),
    });
    let pedersen_points_small_8 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_8".to_owned(),
    });
    let pedersen_points_small_9 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "pedersen_points_small_9".to_owned(),
    });
    let seq_15 = acc.get_preprocessed_column(&PreProcessedColumnId { id: "seq_15".to_owned() });

    // Yield PedersenPointsTableWindowBits9.
    let tuple_0 = &[
        eval!(context, 1791500038),
        eval!(context, seq_15),
        eval!(context, pedersen_points_small_0),
        eval!(context, pedersen_points_small_1),
        eval!(context, pedersen_points_small_2),
        eval!(context, pedersen_points_small_3),
        eval!(context, pedersen_points_small_4),
        eval!(context, pedersen_points_small_5),
        eval!(context, pedersen_points_small_6),
        eval!(context, pedersen_points_small_7),
        eval!(context, pedersen_points_small_8),
        eval!(context, pedersen_points_small_9),
        eval!(context, pedersen_points_small_10),
        eval!(context, pedersen_points_small_11),
        eval!(context, pedersen_points_small_12),
        eval!(context, pedersen_points_small_13),
        eval!(context, pedersen_points_small_14),
        eval!(context, pedersen_points_small_15),
        eval!(context, pedersen_points_small_16),
        eval!(context, pedersen_points_small_17),
        eval!(context, pedersen_points_small_18),
        eval!(context, pedersen_points_small_19),
        eval!(context, pedersen_points_small_20),
        eval!(context, pedersen_points_small_21),
        eval!(context, pedersen_points_small_22),
        eval!(context, pedersen_points_small_23),
        eval!(context, pedersen_points_small_24),
        eval!(context, pedersen_points_small_25),
        eval!(context, pedersen_points_small_26),
        eval!(context, pedersen_points_small_27),
        eval!(context, pedersen_points_small_28),
        eval!(context, pedersen_points_small_29),
        eval!(context, pedersen_points_small_30),
        eval!(context, pedersen_points_small_31),
        eval!(context, pedersen_points_small_32),
        eval!(context, pedersen_points_small_33),
        eval!(context, pedersen_points_small_34),
        eval!(context, pedersen_points_small_35),
        eval!(context, pedersen_points_small_36),
        eval!(context, pedersen_points_small_37),
        eval!(context, pedersen_points_small_38),
        eval!(context, pedersen_points_small_39),
        eval!(context, pedersen_points_small_40),
        eval!(context, pedersen_points_small_41),
        eval!(context, pedersen_points_small_42),
        eval!(context, pedersen_points_small_43),
        eval!(context, pedersen_points_small_44),
        eval!(context, pedersen_points_small_45),
        eval!(context, pedersen_points_small_46),
        eval!(context, pedersen_points_small_47),
        eval!(context, pedersen_points_small_48),
        eval!(context, pedersen_points_small_49),
        eval!(context, pedersen_points_small_50),
        eval!(context, pedersen_points_small_51),
        eval!(context, pedersen_points_small_52),
        eval!(context, pedersen_points_small_53),
        eval!(context, pedersen_points_small_54),
        eval!(context, pedersen_points_small_55),
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
        // Verify this component has 2 ** 15 rows
        let size_bit = component_data.get_n_instances_bit(context, 15);
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
