use crate::circuits::circuit::Circuit;
use crate::circuits::circuit::{Eq, Gate};
use itertools::Itertools;
use std::collections::HashMap;
use stwo::core::fields::m31::BaseField;
use stwo::core::poly::circle::CanonicCoset;
use stwo::prover::backend::Backend;
use stwo::prover::backend::Col;
use stwo::prover::backend::Column;
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::CircleEvaluation;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

#[cfg(test)]
#[path = "preprocessed_test.rs"]
pub mod test;

const N_OP_CODES: usize = 4;
#[derive(Copy, Clone)]
enum OpCode {
    Add,
    Sub,
    Mul,
    PointwiseMul,
}

fn col_to_evaluation<B: Backend>(
    col: Col<B, BaseField>,
) -> CircleEvaluation<B, BaseField, BitReversedOrder> {
    CircleEvaluation::new(CanonicCoset::new(col.len().ilog2()).circle_domain(), col)
}

fn vec_to_col<B: Backend>(vec: Vec<usize>) -> Col<B, BaseField> {
    Col::<B, BaseField>::from_iter(vec.into_iter().map(BaseField::from))
}

fn fill_binary_op_columns<G: Gate>(
    gates: &[G],
    op_code: OpCode,
    multiplicities: &[usize],
    columns: &mut [Vec<usize>; 8],
) {
    for gate in gates.iter() {
        let [in0, in1] = gate.uses()[..] else { panic!("Expected 2 uses for gate") };
        let [out] = gate.yields()[..] else { panic!("Expected 1 yield for gate") };
        let op_code_idx = op_code as usize;
        (0..N_OP_CODES).for_each(|i| {
            columns[i].push(if i == op_code_idx { 1 } else { 0 });
        });
        columns[4].push(in0);
        columns[5].push(in1);
        columns[6].push(out);
        columns[7].push(multiplicities[out]);
    }
}

fn fill_eq_columns(eq_gates: &[Eq], columns: &mut [Vec<usize>; 2]) {
    for gate in eq_gates.iter() {
        let [in0, in1] = gate.uses()[..] else { panic!("Expected 2 uses for gate") };
        assert!(gate.yields().is_empty(), "Expected no yields for Eq gate");
        columns[0].push(in0);
        columns[1].push(in1);
    }
}

/// Adds the preprocessed columns of qm31_ops component to the preprocessed trace. If the component
/// is empty, no columns are added.
fn add_qm31_ops_to_preprocessed_trace<B: Backend>(
    circuit: &Circuit,
    multiplicities: Vec<usize>,
    pp_trace: &mut PreProcessedTrace<B>,
) {
    let mut qm31_ops_columns = [vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![]];
    fill_binary_op_columns(&circuit.add, OpCode::Add, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(&circuit.sub, OpCode::Sub, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(&circuit.mul, OpCode::Mul, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(
        &circuit.pointwise_mul,
        OpCode::PointwiseMul,
        &multiplicities,
        &mut qm31_ops_columns,
    );
    if !qm31_ops_columns[0].is_empty() {
        let n_columns = pp_trace.columns.len();
        pp_trace
            .column_indices
            .insert(PreProcessedColumnId { id: "qm31_ops_add_flag".to_owned() }, n_columns);
        pp_trace
            .column_indices
            .insert(PreProcessedColumnId { id: "qm31_ops_sub_flag".to_owned() }, n_columns + 1);
        pp_trace
            .column_indices
            .insert(PreProcessedColumnId { id: "qm31_ops_mul_flag".to_owned() }, n_columns + 2);
        pp_trace.column_indices.insert(
            PreProcessedColumnId { id: "qm31_ops_pointwise_mul_flag".to_owned() },
            n_columns + 3,
        );
        pp_trace
            .column_indices
            .insert(PreProcessedColumnId { id: "qm31_ops_in0_address".to_owned() }, n_columns + 4);
        pp_trace
            .column_indices
            .insert(PreProcessedColumnId { id: "qm31_ops_in1_address".to_owned() }, n_columns + 5);
        pp_trace
            .column_indices
            .insert(PreProcessedColumnId { id: "qm31_ops_out_address".to_owned() }, n_columns + 6);
        pp_trace
            .column_indices
            .insert(PreProcessedColumnId { id: "qm31_ops_mults".to_owned() }, n_columns + 7);
        pp_trace.columns.extend(qm31_ops_columns.into_iter().map(|col| vec_to_col::<B>(col)));
    }
}

/// Adds the preprocessed columns of eq component to the preprocessed trace. If the component
/// is empty, no columns are added.
fn add_eq_to_preprocessed_trace<B: Backend>(
    circuit: &Circuit,
    pp_trace: &mut PreProcessedTrace<B>,
) {
    let mut eq_columns = [vec![], vec![]];
    fill_eq_columns(&circuit.eq, &mut eq_columns);
    if !eq_columns[0].is_empty() {
        let n_columns = pp_trace.columns.len();
        pp_trace.column_indices.insert(PreProcessedColumnId { id: "eq_lhs".to_owned() }, n_columns);
        pp_trace
            .column_indices
            .insert(PreProcessedColumnId { id: "eq_rhs".to_owned() }, n_columns + 1);
        pp_trace.columns.extend(eq_columns.into_iter().map(|col| vec_to_col::<B>(col)));
    }
}

/// A collection of preprocessed columns, whose values are publicly acknowledged, and independent of
/// the proof.
pub struct PreProcessedTrace<B: Backend> {
    columns: Vec<Col<B, BaseField>>,
    column_indices: HashMap<PreProcessedColumnId, usize>,
}

impl<B: Backend> PreProcessedTrace<B> {
    /// Generates the preprocessed trace for the circuit, assuming it is already finalized.
    pub fn generate_preprocessed_trace(circuit: &Circuit) -> Self {
        let mut pp_trace = Self { columns: vec![], column_indices: HashMap::new() };
        let multiplicities = circuit.compute_multiplicities().0;

        // Add QM31 operations columns.
        add_qm31_ops_to_preprocessed_trace(circuit, multiplicities, &mut pp_trace);

        // Add Eq columns.
        add_eq_to_preprocessed_trace(circuit, &mut pp_trace);

        // TODO(Gali): Add Blake columns.

        pp_trace
    }

    pub fn log_sizes(&self) -> Vec<u32> {
        self.columns.iter().map(|c| c.len().ilog2()).collect()
    }

    pub fn ids(&self) -> Vec<PreProcessedColumnId> {
        self.column_indices.keys().cloned().sorted_by_key(|k| self.column_indices[k]).collect()
    }

    pub fn get_trace(&self) -> Vec<CircleEvaluation<B, BaseField, BitReversedOrder>> {
        self.columns.iter().map(|c| col_to_evaluation::<B>(c.clone())).collect()
    }

    pub fn get_column(&self, id: &PreProcessedColumnId) -> &Col<B, BaseField> {
        &self.columns[*self
            .column_indices
            .get(id)
            .unwrap_or_else(|| panic!("Missing preprocessed column {id:?}"))]
    }
}
