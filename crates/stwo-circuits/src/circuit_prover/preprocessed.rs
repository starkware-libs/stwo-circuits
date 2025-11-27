use crate::circuits::circuit::Circuit;
use crate::circuits::circuit::{Eq, Gate};
use itertools::Itertools;
use stwo::core::ColumnVec;
use stwo::core::fields::m31::BaseField;
use stwo::core::poly::circle::CanonicCoset;
use stwo::prover::backend::Backend;
use stwo::prover::backend::Col;
use stwo::prover::backend::Column;
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::CircleEvaluation;

#[cfg(test)]
#[path = "preprocessed_test.rs"]
pub mod test;

fn vec_to_evaluation<B: Backend>(
    vec: Vec<usize>,
) -> CircleEvaluation<B, BaseField, BitReversedOrder> {
    let col = Col::<B, BaseField>::from_iter(vec.into_iter().map(BaseField::from));
    CircleEvaluation::new(CanonicCoset::new(col.len().ilog2()).circle_domain(), col)
}

fn fill_binary_op_columns<G: Gate>(
    gates: &[G],
    multiplicities: &[usize],
    columns: &mut [Vec<usize>; 4],
) {
    for gate in gates.iter() {
        let [in0, in1] = gate.uses()[..] else { panic!("Expected 2 uses for gate") };
        let [out] = gate.yields()[..] else { panic!("Expected 1 yield for gate") };
        columns[0].push(in0);
        columns[1].push(in1);
        columns[2].push(out);
        columns[3].push(multiplicities[out]);
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
    pp_trace: &mut ColumnVec<Vec<usize>>,
) {
    let mut qm31_ops_columns = [vec![], vec![], vec![], vec![]];
    fill_binary_op_columns(&circuit.add, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(&circuit.sub, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(&circuit.mul, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(&circuit.pointwise_mul, &multiplicities, &mut qm31_ops_columns);
    pp_trace.extend(qm31_ops_columns);
}

/// Adds the preprocessed columns of eq component to the preprocessed trace. If the component
/// is empty, no columns are added.
fn add_eq_to_preprocessed_trace(circuit: &Circuit, pp_trace: &mut ColumnVec<Vec<usize>>) {
    let mut eq_columns = [vec![], vec![]];
    fill_eq_columns(&circuit.eq, &mut eq_columns);
    pp_trace.extend(eq_columns);
}

/// Generates the preprocessed trace for the circuit, assuming it is already finalized.
pub fn generate_preprocessed_trace<B: Backend>(
    circuit: &Circuit,
) -> ColumnVec<CircleEvaluation<B, BaseField, BitReversedOrder>> {
    let mut pp_trace = ColumnVec::new();
    let multiplicities = circuit.compute_multiplicities().0;

    // Add QM31 operations columns.
    add_qm31_ops_to_preprocessed_trace(circuit, multiplicities, &mut pp_trace);

    // Add Eq columns.
    add_eq_to_preprocessed_trace(circuit, &mut pp_trace);

    // TODO(Gali): Add Blake columns.

    pp_trace
        .into_iter()
        .filter(|col| !col.is_empty())
        .map(|col| vec_to_evaluation(col))
        .collect_vec()
}
