use crate::components::prelude::*;
use circuits_stark_verifier::constraint_eval::{ComponentDataTrait, RelationUse};

pub const N_TRACE_COLUMNS: usize = 4;
pub const N_INTERACTION_COLUMNS: usize = 4;

pub struct Eval {
    pub log_size: u32,
    pub common_lookup_elements: relations::CommonLookupElements,
}

pub type Component = FrameworkComponent<Eval>;

impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        self.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let m31_gate_relation_id = E::F::from(M31::from(378353459));
        let in0_address =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "eq_in0_address".to_owned() });
        let in1_address =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "eq_in1_address".to_owned() });

        let in_col0 = eval.next_trace_mask();
        let in_col1 = eval.next_trace_mask();
        let in_col2 = eval.next_trace_mask();
        let in_col3 = eval.next_trace_mask();

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                m31_gate_relation_id.clone(),
                in0_address.clone(),
                in_col0.clone(),
                in_col1.clone(),
                in_col2.clone(),
                in_col3.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                m31_gate_relation_id.clone(),
                in1_address.clone(),
                in_col0.clone(),
                in_col1.clone(),
                in_col2.clone(),
                in_col3.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}

pub struct CircuitEqComponent {}

impl<Value: IValue + 'static> CircuitEval<Value> for CircuitEqComponent {
    fn name(&self) -> String {
        "eq".to_string()
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        let m31_gate_relation_id = context.constant(SecureField::from(M31::from(378353459)));
        let in0_address =
            acc.get_preprocessed_column(&PreProcessedColumnId { id: "eq_in0_address".to_owned() });
        let in1_address =
            acc.get_preprocessed_column(&PreProcessedColumnId { id: "eq_in1_address".to_owned() });

        let [in_col0, in_col1, in_col2, in_col3] = *component_data.trace_columns() else {
            panic!("Expected {N_TRACE_COLUMNS} trace columns")
        };

        acc.add_to_relation(
            context,
            context.one(),
            &[m31_gate_relation_id, in0_address, in_col0, in_col1, in_col2, in_col3],
        );
        acc.add_to_relation(
            context,
            context.one(),
            &[m31_gate_relation_id, in1_address, in_col0, in_col1, in_col2, in_col3],
        );
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &[RelationUse { relation_id: "gate", uses: 2 }]
    }
}
