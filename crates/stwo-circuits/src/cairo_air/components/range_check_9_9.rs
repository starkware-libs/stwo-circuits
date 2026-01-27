// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 8;
pub const N_INTERACTION_COLUMNS: usize = 16;

pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let _ = component_data;
    let _ = acc;
    let [
        multiplicity_0,
        multiplicity_1,
        multiplicity_2,
        multiplicity_3,
        multiplicity_4,
        multiplicity_5,
        multiplicity_6,
        multiplicity_7,
    ] = input.try_into().unwrap();
    let range_check_9_9_column_0 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "range_check_9_9_column_0".to_owned(),
    });
    let range_check_9_9_column_1 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "range_check_9_9_column_1".to_owned(),
    });

    // Yield RangeCheck_9_9.
    let tuple_0 = &[
        eval!(context, 517791011),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_0 = eval!(context, -(multiplicity_0));
    acc.add_to_relation(context, numerator_0, tuple_0);

    // Yield RangeCheck_9_9_B.
    let tuple_1 = &[
        eval!(context, 1897792095),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_1 = eval!(context, -(multiplicity_1));
    acc.add_to_relation(context, numerator_1, tuple_1);

    // Yield RangeCheck_9_9_C.
    let tuple_2 = &[
        eval!(context, 1881014476),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_2 = eval!(context, -(multiplicity_2));
    acc.add_to_relation(context, numerator_2, tuple_2);

    // Yield RangeCheck_9_9_D.
    let tuple_3 = &[
        eval!(context, 1864236857),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_3 = eval!(context, -(multiplicity_3));
    acc.add_to_relation(context, numerator_3, tuple_3);

    // Yield RangeCheck_9_9_E.
    let tuple_4 = &[
        eval!(context, 1847459238),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_4 = eval!(context, -(multiplicity_4));
    acc.add_to_relation(context, numerator_4, tuple_4);

    // Yield RangeCheck_9_9_F.
    let tuple_5 = &[
        eval!(context, 1830681619),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_5 = eval!(context, -(multiplicity_5));
    acc.add_to_relation(context, numerator_5, tuple_5);

    // Yield RangeCheck_9_9_G.
    let tuple_6 = &[
        eval!(context, 1813904000),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_6 = eval!(context, -(multiplicity_6));
    acc.add_to_relation(context, numerator_6, tuple_6);

    // Yield RangeCheck_9_9_H.
    let tuple_7 = &[
        eval!(context, 2065568285),
        eval!(context, range_check_9_9_column_0),
        eval!(context, range_check_9_9_column_1),
    ];
    let numerator_7 = eval!(context, -(multiplicity_7));
    acc.add_to_relation(context, numerator_7, tuple_7);
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
        // Verify this component has 2 ** 18 rows
        let size_bit = component_data.get_n_instances_bit(context, 18);
        eq(context, size_bit, context.one());
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }
}
