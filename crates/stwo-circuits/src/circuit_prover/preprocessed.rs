use crate::circuits::circuit::Circuit;
use crate::circuits::circuit::{Eq, Gate};
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
    vec: Vec<BaseField>,
) -> CircleEvaluation<B, BaseField, BitReversedOrder> {
    let col = Col::<B, BaseField>::from_iter(vec);
    CircleEvaluation::new(CanonicCoset::new(col.len().ilog2()).circle_domain(), col)
}

fn fill_binary_op_columns<G: Gate>(
    gates: &[G],
    multiplicities: &[usize],
    columns: &mut [Vec<BaseField>; 4],
) {
    for gate in gates.iter() {
        let [in0, in1] = gate.uses()[..] else { panic!("Expected 2 uses for gate") };
        let [out] = gate.yields()[..] else { panic!("Expected 1 yield for gate") };
        columns[0].push(BaseField::from(in0));
        columns[1].push(BaseField::from(in1));
        columns[2].push(BaseField::from(out));
        columns[3].push(BaseField::from(multiplicities[out]));
    }
}

fn fill_eq_columns(eq_gates: &[Eq], columns: &mut [Vec<BaseField>; 2]) {
    for gate in eq_gates.iter() {
        let [in0, in1] = gate.uses()[..] else { panic!("Expected 2 uses for gate") };
        assert!(gate.yields().is_empty(), "Expected no yields for Eq gate");
        columns[0].push(BaseField::from(in0));
        columns[1].push(BaseField::from(in1));
    }
}

/// Generates the preprocessed trace for the circuit, assuming it is already finalized.
pub fn generate_preprocessed_trace<B: Backend>(
    circuit: &Circuit,
) -> ColumnVec<CircleEvaluation<B, BaseField, BitReversedOrder>> {
    let mut trace = ColumnVec::new();
    let multiplicities = circuit.compute_multiplicities().0;

    // Fill QM31 operations columns.
    let mut qm31_ops_columns = [vec![], vec![], vec![], vec![]];
    fill_binary_op_columns(&circuit.add, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(&circuit.sub, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(&circuit.mul, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(&circuit.pointwise_mul, &multiplicities, &mut qm31_ops_columns);
    qm31_ops_columns.into_iter().for_each(|col| {
        if !col.is_empty() {
            trace.push(vec_to_evaluation(col));
        }
    });

    // Fill Eq columns.
    let mut eq_columns = [vec![], vec![]];
    fill_eq_columns(&circuit.eq, &mut eq_columns);
    eq_columns.into_iter().for_each(|col| {
        if !col.is_empty() {
            trace.push(vec_to_evaluation(col));
        }
    });

    // TODO(Gali): Fill Blake columns.

    trace
}
