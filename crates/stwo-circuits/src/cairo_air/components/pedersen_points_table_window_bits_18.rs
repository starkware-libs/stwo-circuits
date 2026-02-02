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
    let seq_23 = acc.get_preprocessed_column(&PreProcessedColumnId { id: "seq_23".to_owned() });
    let pedersen_points_0 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_0".to_owned() });
    let pedersen_points_1 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_1".to_owned() });
    let pedersen_points_2 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_2".to_owned() });
    let pedersen_points_3 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_3".to_owned() });
    let pedersen_points_4 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_4".to_owned() });
    let pedersen_points_5 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_5".to_owned() });
    let pedersen_points_6 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_6".to_owned() });
    let pedersen_points_7 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_7".to_owned() });
    let pedersen_points_8 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_8".to_owned() });
    let pedersen_points_9 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_9".to_owned() });
    let pedersen_points_10 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_10".to_owned() });
    let pedersen_points_11 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_11".to_owned() });
    let pedersen_points_12 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_12".to_owned() });
    let pedersen_points_13 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_13".to_owned() });
    let pedersen_points_14 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_14".to_owned() });
    let pedersen_points_15 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_15".to_owned() });
    let pedersen_points_16 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_16".to_owned() });
    let pedersen_points_17 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_17".to_owned() });
    let pedersen_points_18 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_18".to_owned() });
    let pedersen_points_19 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_19".to_owned() });
    let pedersen_points_20 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_20".to_owned() });
    let pedersen_points_21 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_21".to_owned() });
    let pedersen_points_22 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_22".to_owned() });
    let pedersen_points_23 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_23".to_owned() });
    let pedersen_points_24 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_24".to_owned() });
    let pedersen_points_25 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_25".to_owned() });
    let pedersen_points_26 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_26".to_owned() });
    let pedersen_points_27 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_27".to_owned() });
    let pedersen_points_28 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_28".to_owned() });
    let pedersen_points_29 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_29".to_owned() });
    let pedersen_points_30 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_30".to_owned() });
    let pedersen_points_31 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_31".to_owned() });
    let pedersen_points_32 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_32".to_owned() });
    let pedersen_points_33 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_33".to_owned() });
    let pedersen_points_34 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_34".to_owned() });
    let pedersen_points_35 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_35".to_owned() });
    let pedersen_points_36 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_36".to_owned() });
    let pedersen_points_37 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_37".to_owned() });
    let pedersen_points_38 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_38".to_owned() });
    let pedersen_points_39 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_39".to_owned() });
    let pedersen_points_40 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_40".to_owned() });
    let pedersen_points_41 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_41".to_owned() });
    let pedersen_points_42 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_42".to_owned() });
    let pedersen_points_43 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_43".to_owned() });
    let pedersen_points_44 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_44".to_owned() });
    let pedersen_points_45 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_45".to_owned() });
    let pedersen_points_46 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_46".to_owned() });
    let pedersen_points_47 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_47".to_owned() });
    let pedersen_points_48 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_48".to_owned() });
    let pedersen_points_49 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_49".to_owned() });
    let pedersen_points_50 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_50".to_owned() });
    let pedersen_points_51 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_51".to_owned() });
    let pedersen_points_52 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_52".to_owned() });
    let pedersen_points_53 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_53".to_owned() });
    let pedersen_points_54 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_54".to_owned() });
    let pedersen_points_55 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pedersen_points_55".to_owned() });

    // Yield PedersenPointsTableWindowBits18.
    let tuple_0 = &[
        eval!(context, 1444721856),
        eval!(context, seq_23),
        eval!(context, pedersen_points_0),
        eval!(context, pedersen_points_1),
        eval!(context, pedersen_points_2),
        eval!(context, pedersen_points_3),
        eval!(context, pedersen_points_4),
        eval!(context, pedersen_points_5),
        eval!(context, pedersen_points_6),
        eval!(context, pedersen_points_7),
        eval!(context, pedersen_points_8),
        eval!(context, pedersen_points_9),
        eval!(context, pedersen_points_10),
        eval!(context, pedersen_points_11),
        eval!(context, pedersen_points_12),
        eval!(context, pedersen_points_13),
        eval!(context, pedersen_points_14),
        eval!(context, pedersen_points_15),
        eval!(context, pedersen_points_16),
        eval!(context, pedersen_points_17),
        eval!(context, pedersen_points_18),
        eval!(context, pedersen_points_19),
        eval!(context, pedersen_points_20),
        eval!(context, pedersen_points_21),
        eval!(context, pedersen_points_22),
        eval!(context, pedersen_points_23),
        eval!(context, pedersen_points_24),
        eval!(context, pedersen_points_25),
        eval!(context, pedersen_points_26),
        eval!(context, pedersen_points_27),
        eval!(context, pedersen_points_28),
        eval!(context, pedersen_points_29),
        eval!(context, pedersen_points_30),
        eval!(context, pedersen_points_31),
        eval!(context, pedersen_points_32),
        eval!(context, pedersen_points_33),
        eval!(context, pedersen_points_34),
        eval!(context, pedersen_points_35),
        eval!(context, pedersen_points_36),
        eval!(context, pedersen_points_37),
        eval!(context, pedersen_points_38),
        eval!(context, pedersen_points_39),
        eval!(context, pedersen_points_40),
        eval!(context, pedersen_points_41),
        eval!(context, pedersen_points_42),
        eval!(context, pedersen_points_43),
        eval!(context, pedersen_points_44),
        eval!(context, pedersen_points_45),
        eval!(context, pedersen_points_46),
        eval!(context, pedersen_points_47),
        eval!(context, pedersen_points_48),
        eval!(context, pedersen_points_49),
        eval!(context, pedersen_points_50),
        eval!(context, pedersen_points_51),
        eval!(context, pedersen_points_52),
        eval!(context, pedersen_points_53),
        eval!(context, pedersen_points_54),
        eval!(context, pedersen_points_55),
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
        // Verify this component has 2 ** 23 rows
        let size_bit = component_data.get_n_instances_bit(context, 23);
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
