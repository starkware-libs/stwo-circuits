use crate::circuit_air::components::qm31_ops::Claim;
use crate::circuit_air::components::qm31_ops::InteractionClaim;
use crate::circuit_air::relations;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use crate::circuit_prover::witness::utils::TreeBuilder;
use crate::circuit_prover::witness::utils::pack_values;
use crate::circuits::circuit::{Add, Mul, PointwiseMul, Sub};
use num_traits::One;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::backend::simd::m31::{LOG_N_LANES, N_LANES, PackedM31};
use stwo::prover::backend::simd::qm31::PackedQM31;
use stwo_air_utils::trace::component_trace::ComponentTrace;
use stwo_air_utils_derive::{IterMut, ParIterMut, Uninitialized};
use stwo_constraint_framework::LogupTraceGenerator;
use stwo_constraint_framework::Relation;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

const N_TRACE_COLUMNS: usize = 12;

pub type InputType = [[M31; 4]; 3];
pub type PackedInputType = [[PackedM31; 4]; 3];

pub fn get_columns_from_context(
    add_gates: &[Add],
    sub_gates: &[Sub],
    mul_gates: &[Mul],
    pointwise_mul_gates: &[PointwiseMul],
    values: &[QM31],
) -> Vec<InputType> {
    let mut inputs = vec![];
    for gate in add_gates {
        inputs.push([
            values[gate.in0].to_m31_array(),
            values[gate.in1].to_m31_array(),
            values[gate.out].to_m31_array(),
        ]);
    }
    for gate in sub_gates {
        inputs.push([
            values[gate.in0].to_m31_array(),
            values[gate.in1].to_m31_array(),
            values[gate.out].to_m31_array(),
        ]);
    }
    for gate in mul_gates {
        inputs.push([
            values[gate.in0].to_m31_array(),
            values[gate.in1].to_m31_array(),
            values[gate.out].to_m31_array(),
        ]);
    }
    for gate in pointwise_mul_gates {
        inputs.push([
            values[gate.in0].to_m31_array(),
            values[gate.in1].to_m31_array(),
            values[gate.out].to_m31_array(),
        ]);
    }

    inputs
}

pub fn write_trace(
    add_gates: &[Add],
    sub_gates: &[Sub],
    mul_gates: &[Mul],
    pointwise_mul_gates: &[PointwiseMul],
    values: &[QM31],
    preprocessed_trace: &PreProcessedTrace<SimdBackend>,
    tree_builder: &mut impl TreeBuilder<SimdBackend>,
) -> (Claim, InteractionClaimGenerator) {
    let inputs =
        get_columns_from_context(add_gates, sub_gates, mul_gates, pointwise_mul_gates, values);

    let n_rows = inputs.len();
    assert_ne!(n_rows, 0);
    assert!(n_rows >= N_LANES);
    assert!(n_rows.is_power_of_two());
    let log_size = n_rows.ilog2();

    let packed_inputs = pack_values(&inputs);

    let (trace, lookup_data) = write_trace_simd(packed_inputs, preprocessed_trace);
    tree_builder.extend_evals(trace.to_evals());

    (Claim { log_size }, InteractionClaimGenerator { log_size, lookup_data })
}

fn write_trace_simd(
    inputs: Vec<PackedInputType>,
    preprocessed_trace: &PreProcessedTrace<SimdBackend>,
) -> (ComponentTrace<N_TRACE_COLUMNS>, LookupData) {
    let log_n_packed_rows = inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;
    let (mut trace, mut lookup_data) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
            LookupData::uninitialized(log_n_packed_rows),
        )
    };

    let add_flag = &preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "qm31_ops_add_flag".to_owned() })
        .data;
    let sub_flag = &preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "qm31_ops_sub_flag".to_owned() })
        .data;
    let mul_flag = &preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "qm31_ops_mul_flag".to_owned() })
        .data;
    let pointwise_mul_flag = &preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "qm31_ops_pointwise_mul_flag".to_owned() })
        .data;
    let in0_address = &preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "qm31_ops_in0_address".to_owned() })
        .data;
    let in1_address = &preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "qm31_ops_in1_address".to_owned() })
        .data;
    let out_address = &preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "qm31_ops_out_address".to_owned() })
        .data;
    let mults = &preprocessed_trace
        .get_column(&PreProcessedColumnId { id: "qm31_ops_mults".to_owned() })
        .data;

    (trace.par_iter_mut(), lookup_data.par_iter_mut(), inputs.into_par_iter())
        .into_par_iter()
        .enumerate()
        .for_each(|(row_index, (mut row, lookup_data, qm_31_ops_input))| {
            let _add_flag = add_flag[row_index];
            let _sub_flag = sub_flag[row_index];
            let _mul_flag = mul_flag[row_index];
            let _pointwise_mul_flag = pointwise_mul_flag[row_index];
            let in0_address = in0_address[row_index];
            let in1_address = in1_address[row_index];
            let out_address = out_address[row_index];
            let mults = mults[row_index];

            let in0_col0 = qm_31_ops_input[0][0];
            *row[0] = in0_col0;
            let in0_col1 = qm_31_ops_input[0][1];
            *row[1] = in0_col1;
            let in0_col2 = qm_31_ops_input[0][2];
            *row[2] = in0_col2;
            let in0_col3 = qm_31_ops_input[0][3];
            *row[3] = in0_col3;
            let in1_col4 = qm_31_ops_input[1][0];
            *row[4] = in1_col4;
            let in1_col5 = qm_31_ops_input[1][1];
            *row[5] = in1_col5;
            let in1_col6 = qm_31_ops_input[1][2];
            *row[6] = in1_col6;
            let in1_col7 = qm_31_ops_input[1][3];
            *row[7] = in1_col7;
            let out_col8 = qm_31_ops_input[2][0];
            *row[8] = out_col8;
            let out_col9 = qm_31_ops_input[2][1];
            *row[9] = out_col9;
            let out_col10 = qm_31_ops_input[2][2];
            *row[10] = out_col10;
            let out_col11 = qm_31_ops_input[2][3];
            *row[11] = out_col11;
            *lookup_data.in_0 = [in0_address, in0_col0, in0_col1, in0_col2, in0_col3];
            *lookup_data.in_1 = [in1_address, in1_col4, in1_col5, in1_col6, in1_col7];
            *lookup_data.out = [out_address, out_col8, out_col9, out_col10, out_col11];
            *lookup_data.mults = mults;
        });

    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    in_0: Vec<[PackedM31; 5]>,
    in_1: Vec<[PackedM31; 5]>,
    out: Vec<[PackedM31; 5]>,
    mults: Vec<PackedM31>,
}

pub struct InteractionClaimGenerator {
    log_size: u32,
    lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        gate: &relations::Gate,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

        // Sum logup terms in pairs.
        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.in_0, &self.lookup_data.in_1)
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = gate.combine(values0);
                let denom1: PackedQM31 = gate.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        // Sum last logup term.
        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.out, self.lookup_data.mults)
            .into_par_iter()
            .for_each(|(writer, values, mults)| {
                let denom = gate.combine(values);
                writer.write_frac(-PackedQM31::one() * mults, denom);
            });
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
