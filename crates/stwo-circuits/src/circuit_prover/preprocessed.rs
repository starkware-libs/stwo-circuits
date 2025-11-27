use crate::circuits::circuit::Circuit;
use itertools::Itertools;
use stwo::core::ColumnVec;
use stwo::core::fields::m31::BaseField;
use stwo::core::poly::circle::CanonicCoset;
use stwo::prover::backend::Col;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::CircleEvaluation;

#[cfg(test)]
#[path = "preprocessed_test.rs"]
pub mod test;

fn to_col(vec: Vec<BaseField>) -> Col<SimdBackend, BaseField> {
    Col::<SimdBackend, BaseField>::from_iter(vec)
}

fn col_to_evaluation(
    col: Col<SimdBackend, BaseField>,
) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
    CircleEvaluation::new(CanonicCoset::new(col.length.ilog2()).circle_domain(), col)
}

macro_rules! process_gate {
    ($gates:expr, [$($field:ident),+]) => {{
        $(
            let $field = $gates.iter().map(|gate| BaseField::from(gate.$field)).collect_vec();
        )+

        vec![$($field),+].into_iter().map(|field| col_to_evaluation(to_col(field))).collect_vec()
    }};
}

/// Generates the preprocessed trace for the circuit, assuming it is already finalized.
pub fn generate_preprocessed_trace(
    circuit: &Circuit,
) -> ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>> {
    let mut trace = ColumnVec::new();

    if !circuit.add.is_empty() {
        trace.extend(process_gate!(&circuit.add, [in0, in1, out]));
    }
    if !circuit.sub.is_empty() {
        trace.extend(process_gate!(&circuit.sub, [in0, in1, out]));
    }
    if !circuit.mul.is_empty() {
        trace.extend(process_gate!(&circuit.mul, [in0, in1, out]));
    }
    if !circuit.pointwise_mul.is_empty() {
        trace.extend(process_gate!(&circuit.pointwise_mul, [in0, in1, out]));
    }
    if !circuit.eq.is_empty() {
        trace.extend(process_gate!(&circuit.eq, [in0, in1]));
    }
    // TODO(Gali): Add blake gates

    trace
}
