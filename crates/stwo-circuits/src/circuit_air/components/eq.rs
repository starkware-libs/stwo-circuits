use crate::circuit_air::relations;
use num_traits::One;
use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;
use stwo::core::pcs::TreeVec;
use stwo_constraint_framework::EvalAtRow;
use stwo_constraint_framework::FrameworkComponent;
use stwo_constraint_framework::FrameworkEval;
use stwo_constraint_framework::RelationEntry;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

pub const N_TRACE_COLUMNS: usize = 8;

pub struct Eval {
    pub log_size: u32,
    pub gate_lookup_elements: relations::Gate,
}

pub fn log_sizes(log_size: u32) -> TreeVec<Vec<u32>> {
    let trace_log_sizes = vec![log_size; N_TRACE_COLUMNS];
    let interaction_log_sizes = vec![log_size; SECURE_EXTENSION_DEGREE];
    TreeVec::new(vec![vec![], trace_log_sizes, interaction_log_sizes])
}

pub type Component = FrameworkComponent<Eval>;

impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        self.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let in0_address =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "eq_in0_address".to_owned() });
        let in1_address =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "eq_in1_address".to_owned() });

        let in0_col0 = eval.next_trace_mask();
        let in0_col1 = eval.next_trace_mask();
        let in0_col2 = eval.next_trace_mask();
        let in0_col3 = eval.next_trace_mask();
        let in1_col4 = eval.next_trace_mask();
        let in1_col5 = eval.next_trace_mask();
        let in1_col6 = eval.next_trace_mask();
        let in1_col7 = eval.next_trace_mask();

        // in0 col 0 equals in1 col 4.
        eval.add_constraint(in0_col0.clone() - in1_col4.clone());

        // in0 col 1 equals in1 col 5.
        eval.add_constraint(in0_col1.clone() - in1_col5.clone());

        // in0 col 2 equals in1 col 6.
        eval.add_constraint(in0_col2.clone() - in1_col6.clone());

        // in0 col 3 equals in1 col 7.
        eval.add_constraint(in0_col3.clone() - in1_col7.clone());

        eval.add_to_relation(RelationEntry::new(
            &self.gate_lookup_elements,
            E::EF::one(),
            &[
                in0_address.clone(),
                in0_col0.clone(),
                in0_col1.clone(),
                in0_col2.clone(),
                in0_col3.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.gate_lookup_elements,
            E::EF::one(),
            &[
                in1_address.clone(),
                in1_col4.clone(),
                in1_col5.clone(),
                in1_col6.clone(),
                in1_col7.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}
