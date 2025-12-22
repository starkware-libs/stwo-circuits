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

const N_QM31_OPS_COLUMNS: usize = 8;
const N_EQ_COLUMNS: usize = 2;
const N_OP_CODES: usize = 4;

#[derive(Copy, Clone)]
enum OpCode {
    Add,
    Sub,
    Mul,
    PointwiseMul,
}

fn vec_to_evaluation<B: Backend>(
    vec: Vec<usize>,
) -> CircleEvaluation<B, BaseField, BitReversedOrder> {
    let col = Col::<B, BaseField>::from_iter(vec.into_iter().map(BaseField::from));
    CircleEvaluation::new(CanonicCoset::new(col.len().ilog2()).circle_domain(), col)
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
fn add_qm31_ops_to_preprocessed_trace(
    circuit: &Circuit,
    multiplicities: Vec<usize>,
    pp_trace: &mut PreProcessedTrace,
) {
    let mut qm31_ops_columns: [_; N_QM31_OPS_COLUMNS] = std::array::from_fn(|_| vec![]);
    fill_binary_op_columns(&circuit.add, OpCode::Add, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(&circuit.sub, OpCode::Sub, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(&circuit.mul, OpCode::Mul, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(
        &circuit.pointwise_mul,
        OpCode::PointwiseMul,
        &multiplicities,
        &mut qm31_ops_columns,
    );

    let n_columns = pp_trace.columns.len();
    pp_trace.column_indices.extend([
        (PreProcessedColumnId { id: "qm31_ops_add_flag".to_owned() }, n_columns),
        (PreProcessedColumnId { id: "qm31_ops_sub_flag".to_owned() }, n_columns + 1),
        (PreProcessedColumnId { id: "qm31_ops_mul_flag".to_owned() }, n_columns + 2),
        (PreProcessedColumnId { id: "qm31_ops_pointwise_mul_flag".to_owned() }, n_columns + 3),
        (PreProcessedColumnId { id: "qm31_ops_in0_address".to_owned() }, n_columns + 4),
        (PreProcessedColumnId { id: "qm31_ops_in1_address".to_owned() }, n_columns + 5),
        (PreProcessedColumnId { id: "qm31_ops_out_address".to_owned() }, n_columns + 6),
        (PreProcessedColumnId { id: "qm31_ops_mults".to_owned() }, n_columns + 7),
    ]);
    pp_trace.columns.extend(qm31_ops_columns);
}

/// Adds the preprocessed columns of eq component to the preprocessed trace. If the component
/// is empty, no columns are added.
fn add_eq_to_preprocessed_trace(circuit: &Circuit, pp_trace: &mut PreProcessedTrace) {
    let mut eq_columns: [_; N_EQ_COLUMNS] = std::array::from_fn(|_| vec![]);
    fill_eq_columns(&circuit.eq, &mut eq_columns);

    let n_columns = pp_trace.columns.len();
    pp_trace.column_indices.extend([
        (PreProcessedColumnId { id: "eq_in0_address".to_owned() }, n_columns),
        (PreProcessedColumnId { id: "eq_in1_address".to_owned() }, n_columns + 1),
    ]);
    pp_trace.columns.extend(eq_columns);
}

/// A collection of preprocessed columns, whose values are publicly acknowledged, and independent of
/// the proof.
pub struct PreProcessedTrace {
    columns: Vec<Vec<usize>>,
    column_indices: HashMap<PreProcessedColumnId, usize>,
}

impl PreProcessedTrace {
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

    pub fn get_trace<B: Backend>(&self) -> Vec<CircleEvaluation<B, BaseField, BitReversedOrder>> {
        self.columns.iter().map(|c| vec_to_evaluation::<B>(c.clone())).collect()
    }

    pub fn get_column(&self, id: &PreProcessedColumnId) -> &Vec<usize> {
        &self.columns[*self
            .column_indices
            .get(id)
            .unwrap_or_else(|| panic!("Missing preprocessed column {id:?}"))]
    }
}
