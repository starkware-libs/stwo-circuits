// This file was created by the AIR team.

#![allow(unused_parens)]
use crate::circuit_air::components::blake_output::{Claim, InteractionClaim, N_TRACE_COLUMNS};

use crate::circuit_prover::witness::components::prelude::*;

pub type PackedInputType = [PackedUInt32; 8];

pub struct ClaimGenerator {
    pub packed_inputs: Vec<PackedInputType>,
    preprocessed_trace: Arc<PreProcessedTrace>,
}

impl ClaimGenerator {
    pub fn new(
        packed_inputs: Vec<PackedInputType>,
        preprocessed_trace: Arc<PreProcessedTrace>,
    ) -> Self {
        Self { packed_inputs, preprocessed_trace }
    }

    pub fn write_trace(
        mut self,
    ) -> (ComponentTrace<N_TRACE_COLUMNS>, Claim, InteractionClaimGenerator) {
        assert!(!self.packed_inputs.is_empty());
        let n_vec_rows = self.packed_inputs.len();
        let packed_size = n_vec_rows.next_power_of_two();
        let log_size = packed_size.ilog2() + LOG_N_LANES;
        self.packed_inputs.resize(packed_size, *self.packed_inputs.first().unwrap());

        let (trace, lookup_data) = write_trace_simd(self.packed_inputs, &self.preprocessed_trace);

        (trace, Claim { log_size }, InteractionClaimGenerator { log_size, lookup_data })
    }
}

#[allow(clippy::useless_conversion)]
#[allow(clippy::double_parens)]
#[allow(non_snake_case)]
fn write_trace_simd(
    packed_inputs: Vec<PackedInputType>,
    preprocessed_trace: &PreProcessedTrace,
) -> (ComponentTrace<N_TRACE_COLUMNS>, LookupData) {
    let log_n_packed_rows = packed_inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;
    let (mut trace, mut lookup_data) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
            LookupData::uninitialized(log_n_packed_rows),
        )
    };

    let M31_0 = PackedM31::broadcast(M31::from(0));
    let M31_1061955672 = PackedM31::broadcast(M31::from(1061955672));
    let M31_65536 = PackedM31::broadcast(M31::from(65536));
    let final_state_addr = preprocessed_trace
        .get_packed_column(&PreProcessedColumnId { id: "final_state_addr".to_owned() });
    let blake_output0_addr = preprocessed_trace
        .get_packed_column(&PreProcessedColumnId { id: "blake_output0_addr".to_owned() });
    let blake_output1_addr = preprocessed_trace
        .get_packed_column(&PreProcessedColumnId { id: "blake_output1_addr".to_owned() });
    let mults0 = preprocessed_trace
        .get_packed_column(&PreProcessedColumnId { id: "blake_output0_mults".to_owned() });
    let mults1 = preprocessed_trace
        .get_packed_column(&PreProcessedColumnId { id: "blake_output1_mults".to_owned() });

    (trace.par_iter_mut(), lookup_data.par_iter_mut(), packed_inputs.into_par_iter())
        .into_par_iter()
        .enumerate()
        .for_each(|(row_index, (row, lookup_data, blake_output_input))| {
            let final_state_addr = final_state_addr[row_index];
            let blake_output0_addr = blake_output0_addr[row_index];
            let blake_output1_addr = blake_output1_addr[row_index];
            let input_final_state_limb0_limb_0_col0 = blake_output_input[0].low().as_m31();
            *row[0] = input_final_state_limb0_limb_0_col0;
            let input_final_state_limb0_limb_1_col1 = blake_output_input[0].high().as_m31();
            *row[1] = input_final_state_limb0_limb_1_col1;
            let input_final_state_limb1_limb_0_col2 = blake_output_input[1].low().as_m31();
            *row[2] = input_final_state_limb1_limb_0_col2;
            let input_final_state_limb1_limb_1_col3 = blake_output_input[1].high().as_m31();
            *row[3] = input_final_state_limb1_limb_1_col3;
            let input_final_state_limb2_limb_0_col4 = blake_output_input[2].low().as_m31();
            *row[4] = input_final_state_limb2_limb_0_col4;
            let input_final_state_limb2_limb_1_col5 = blake_output_input[2].high().as_m31();
            *row[5] = input_final_state_limb2_limb_1_col5;
            let input_final_state_limb3_limb_0_col6 = blake_output_input[3].low().as_m31();
            *row[6] = input_final_state_limb3_limb_0_col6;
            let input_final_state_limb3_limb_1_col7 = blake_output_input[3].high().as_m31();
            *row[7] = input_final_state_limb3_limb_1_col7;
            let input_final_state_limb4_limb_0_col8 = blake_output_input[4].low().as_m31();
            *row[8] = input_final_state_limb4_limb_0_col8;
            let input_final_state_limb4_limb_1_col9 = blake_output_input[4].high().as_m31();
            *row[9] = input_final_state_limb4_limb_1_col9;
            let input_final_state_limb5_limb_0_col10 = blake_output_input[5].low().as_m31();
            *row[10] = input_final_state_limb5_limb_0_col10;
            let input_final_state_limb5_limb_1_col11 = blake_output_input[5].high().as_m31();
            *row[11] = input_final_state_limb5_limb_1_col11;
            let input_final_state_limb6_limb_0_col12 = blake_output_input[6].low().as_m31();
            *row[12] = input_final_state_limb6_limb_0_col12;
            let input_final_state_limb6_limb_1_col13 = blake_output_input[6].high().as_m31();
            *row[13] = input_final_state_limb6_limb_1_col13;
            let input_final_state_limb7_limb_0_col14 = blake_output_input[7].low().as_m31();
            *row[14] = input_final_state_limb7_limb_0_col14;
            let input_final_state_limb7_limb_1_col15 = blake_output_input[7].high().as_m31();
            *row[15] = input_final_state_limb7_limb_1_col15;
            let output_limb0_col16 = ((input_final_state_limb0_limb_0_col0)
                + ((input_final_state_limb0_limb_1_col1) * (M31_65536)));
            *row[16] = output_limb0_col16;
            let output_limb1_col17 = ((input_final_state_limb1_limb_0_col2)
                + ((input_final_state_limb1_limb_1_col3) * (M31_65536)));
            *row[17] = output_limb1_col17;
            let output_limb2_col18 = ((input_final_state_limb2_limb_0_col4)
                + ((input_final_state_limb2_limb_1_col5) * (M31_65536)));
            *row[18] = output_limb2_col18;
            let output_limb3_col19 = ((input_final_state_limb3_limb_0_col6)
                + ((input_final_state_limb3_limb_1_col7) * (M31_65536)));
            *row[19] = output_limb3_col19;
            let output_limb4_col20 = ((input_final_state_limb4_limb_0_col8)
                + ((input_final_state_limb4_limb_1_col9) * (M31_65536)));
            *row[20] = output_limb4_col20;
            let output_limb5_col21 = ((input_final_state_limb5_limb_0_col10)
                + ((input_final_state_limb5_limb_1_col11) * (M31_65536)));
            *row[21] = output_limb5_col21;
            let output_limb6_col22 = ((input_final_state_limb6_limb_0_col12)
                + ((input_final_state_limb6_limb_1_col13) * (M31_65536)));
            *row[22] = output_limb6_col22;
            let output_limb7_col23 = ((input_final_state_limb7_limb_0_col14)
                + ((input_final_state_limb7_limb_1_col15) * (M31_65536)));
            *row[23] = output_limb7_col23;
            *lookup_data.blake_output_0 = [
                M31_1061955672,
                final_state_addr,
                input_final_state_limb0_limb_0_col0,
                input_final_state_limb0_limb_1_col1,
                input_final_state_limb1_limb_0_col2,
                input_final_state_limb1_limb_1_col3,
                input_final_state_limb2_limb_0_col4,
                input_final_state_limb2_limb_1_col5,
                input_final_state_limb3_limb_0_col6,
                input_final_state_limb3_limb_1_col7,
                input_final_state_limb4_limb_0_col8,
                input_final_state_limb4_limb_1_col9,
                input_final_state_limb5_limb_0_col10,
                input_final_state_limb5_limb_1_col11,
                input_final_state_limb6_limb_0_col12,
                input_final_state_limb6_limb_1_col13,
                input_final_state_limb7_limb_0_col14,
                input_final_state_limb7_limb_1_col15,
            ];
            *lookup_data.gate_0 = [
                M31_0,
                blake_output0_addr,
                output_limb0_col16,
                output_limb1_col17,
                output_limb2_col18,
                output_limb3_col19,
            ];
            *lookup_data.gate_1 = [
                M31_0,
                blake_output1_addr,
                output_limb4_col20,
                output_limb5_col21,
                output_limb6_col22,
                output_limb7_col23,
            ];
        });
    lookup_data.mults_0 = mults0;
    lookup_data.mults_1 = mults1;
    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    blake_output_0: Vec<[PackedM31; 18]>,
    gate_0: Vec<[PackedM31; 6]>,
    gate_1: Vec<[PackedM31; 6]>,
    mults_0: Vec<PackedM31>,
    mults_1: Vec<PackedM31>,
}

pub struct InteractionClaimGenerator {
    log_size: u32,
    lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        common_lookup_elements: &relations::CommonLookupElements,
    ) -> (Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>, InteractionClaim) {
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

        // Sum logup terms in pairs.
        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.blake_output_0,
            &self.lookup_data.gate_0,
            self.lookup_data.mults_0,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1, mult)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                eprintln!("Mult 0: {:?}", mult);
                writer.write_frac(denom1 - (denom0 * mult), denom0 * denom1);
            });
        col_gen.finalize_col();

        // Sum last logup term.
        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.gate_1, self.lookup_data.mults_1)
            .into_par_iter()
            .for_each(|(writer, values, mult)| {
                               eprintln!("Mult 1: {:?}", mult);

                let denom = common_lookup_elements.combine(values);
                writer.write_frac(-PackedQM31::one() * mult, denom);
            });
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();

        (trace, InteractionClaim { claimed_sum })
    }
}
