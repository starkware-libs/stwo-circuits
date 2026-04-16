use std::mem::MaybeUninit;
use std::sync::Arc;

use crate::witness::components::blake_g;
use crate::witness::components::blake_g_gate;
use crate::witness::components::blake_gate;
use crate::witness::components::blake_output;
use crate::witness::components::blake_round;
use crate::witness::components::blake_round_sigma;
use crate::witness::components::eq;
use crate::witness::components::m_31_to_u_32;
use crate::witness::components::qm31_ops;
use crate::witness::components::range_check_15;
use crate::witness::components::range_check_16;
use crate::witness::components::triple_xor;
use crate::witness::components::triple_xor_32;
use crate::witness::components::verify_bitwise_xor_4;
use crate::witness::components::verify_bitwise_xor_7;
use crate::witness::components::verify_bitwise_xor_8;
use crate::witness::components::verify_bitwise_xor_9;
use crate::witness::components::verify_bitwise_xor_12;
use circuit_air::circuit_claim::CircuitClaim;
use circuit_air::circuit_claim::CircuitInteractionClaim;
use circuit_air::circuit_claim::CircuitInteractionElements;
use circuit_common::Qm31OpsTraceGenerator;
use circuit_common::preprocessed::PreProcessedTrace;
use itertools::Itertools;
use num_traits::Zero;
use rayon::scope;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::TreeBuilder;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::poly::circle::PolyOps;
use stwo::prover::poly::twiddles::TwiddleTree;

pub struct TraceGenerator {
    pub qm31_ops_trace_generator: Qm31OpsTraceGenerator,
}

pub fn write_trace(
    context_values: &[QM31],
    preprocessed_trace: Arc<PreProcessedTrace>,
    output_addresses: &[usize],
    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sM31MerkleChannel>,
    trace_generator: &TraceGenerator,
    twiddles: &TwiddleTree<SimdBackend>,
) -> (CircuitClaim, CircuitInteractionClaimGenerator) {
    let preprocessed_trace_ref = preprocessed_trace.as_ref();

    // Parent scope: eq/qm31_ops traces run as spawns alongside everything else.
    let mut eq_result = MaybeUninit::uninit();
    let mut qm31_ops_result = MaybeUninit::uninit();
    let mut verify_bitwise_xor_8_state = MaybeUninit::uninit();
    let mut verify_bitwise_xor_12_state = MaybeUninit::uninit();
    let mut verify_bitwise_xor_4_state = MaybeUninit::uninit();
    let mut verify_bitwise_xor_7_state = MaybeUninit::uninit();
    let mut verify_bitwise_xor_9_state = MaybeUninit::uninit();
    let mut range_check_16_state = MaybeUninit::uninit();
    let mut range_check_15_state = MaybeUninit::uninit();
    let mut triple_xor_32_state = MaybeUninit::uninit();
    let mut blake_gate_claim_generator = MaybeUninit::uninit();
    let mut blake_round_generator = MaybeUninit::uninit();
    let mut blake_round_sigma_generator = MaybeUninit::uninit();
    let mut blake_g_generator = MaybeUninit::uninit();
    let mut blake_gate_polys_result = MaybeUninit::uninit();
    let mut blake_gate_interaction_claim_gen_result = MaybeUninit::uninit();
    let mut blake_round_polys_result = MaybeUninit::uninit();
    let mut blake_round_log_size_result = MaybeUninit::uninit();
    let mut blake_round_interaction_claim_gen_result = MaybeUninit::uninit();
    let mut blake_round_sigma_polys_result = MaybeUninit::uninit();
    let mut blake_round_sigma_interaction_claim_gen_result = MaybeUninit::uninit();
    let mut blake_g_polys_result = MaybeUninit::uninit();
    let mut blake_g_claim_icg = MaybeUninit::uninit();
    let mut blake_output_polys_result = MaybeUninit::uninit();
    let mut blake_output_claim_icg = MaybeUninit::uninit();
    let mut triple_xor_32_polys_result = MaybeUninit::uninit();
    let mut triple_xor_32_claim_icg = MaybeUninit::uninit();
    let mut verify_bitwise_xor_8_result = MaybeUninit::uninit();
    let mut verify_bitwise_xor_12_result = MaybeUninit::uninit();
    let mut verify_bitwise_xor_4_result = MaybeUninit::uninit();
    let mut verify_bitwise_xor_7_result = MaybeUninit::uninit();
    let mut verify_bitwise_xor_9_result = MaybeUninit::uninit();
    let mut range_check_15_result = MaybeUninit::uninit();
    let mut range_check_16_result = MaybeUninit::uninit();
    let mut blake_g_trace_data = MaybeUninit::uninit();
    let mut triple_xor_32_trace_data = MaybeUninit::uninit();
    let mut triple_xor_trace_data = MaybeUninit::uninit();
    let mut triple_xor_claim_icg = MaybeUninit::uninit();
    let mut triple_xor_polys_result = MaybeUninit::uninit();
    let mut blake_g_gate_trace_data = MaybeUninit::uninit();
    let mut blake_g_gate_claim_icg = MaybeUninit::uninit();
    let mut blake_g_gate_polys_result = MaybeUninit::uninit();
    let mut m_31_to_u_32_trace_data = MaybeUninit::uninit();
    let mut m_31_to_u_32_claim_icg = MaybeUninit::uninit();
    let mut m_31_to_u_32_polys_result = MaybeUninit::uninit();
    scope(|s| {
        // Eq and qm31_ops traces run for the full duration of the parent scope.
        s.spawn(|_| {
            let (trace, log_size, lookup_data) = qm31_ops::write_trace(
                context_values,
                preprocessed_trace_ref,
                &trace_generator.qm31_ops_trace_generator,
            );
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            qm31_ops_result.write((polys, log_size, lookup_data));
        });
        s.spawn(|_| {
            let (trace, log_size, lookup_data) =
                eq::write_trace(context_values, preprocessed_trace_ref);
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            eq_result.write((polys, log_size, lookup_data));
        });

        // Initialize claim generators in a subscope.
        scope(|s| {
            s.spawn(|_| {
                verify_bitwise_xor_8_state
                    .write(verify_bitwise_xor_8::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                verify_bitwise_xor_12_state
                    .write(verify_bitwise_xor_12::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                verify_bitwise_xor_4_state
                    .write(verify_bitwise_xor_4::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                verify_bitwise_xor_7_state
                    .write(verify_bitwise_xor_7::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                verify_bitwise_xor_9_state
                    .write(verify_bitwise_xor_9::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                range_check_16_state
                    .write(range_check_16::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                range_check_15_state
                    .write(range_check_15::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                triple_xor_32_state.write(triple_xor_32::ClaimGenerator::new());
            });
            s.spawn(|_| {
                blake_gate_claim_generator
                    .write(blake_gate::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                blake_round_generator.write(blake_round::ClaimGenerator::default());
            });
            s.spawn(|_| {
                blake_round_sigma_generator
                    .write(blake_round_sigma::ClaimGenerator::new(preprocessed_trace.clone()));
            });
            s.spawn(|_| {
                blake_g_generator.write(blake_g::ClaimGenerator::new());
            });
        });

        // SAFETY: Claim generators were initialized by the subscope above.
        // Move xor/range-check states out of MaybeUninit into owned locals so we can
        // borrow them for blake_gate/blake_g/triple_xor_32, then move them into spawns.
        let verify_bitwise_xor_8_state = unsafe { verify_bitwise_xor_8_state.assume_init() };
        let verify_bitwise_xor_12_state = unsafe { verify_bitwise_xor_12_state.assume_init() };
        let verify_bitwise_xor_4_state = unsafe { verify_bitwise_xor_4_state.assume_init() };
        let verify_bitwise_xor_7_state = unsafe { verify_bitwise_xor_7_state.assume_init() };
        let verify_bitwise_xor_9_state = unsafe { verify_bitwise_xor_9_state.assume_init() };
        let range_check_16_state = unsafe { range_check_16_state.assume_init() };
        let range_check_15_state = unsafe { range_check_15_state.assume_init() };
        let mut triple_xor_32_state = unsafe { triple_xor_32_state.assume_init() };
        let blake_gate_claim_generator = unsafe { blake_gate_claim_generator.assume_init() };
        let mut blake_round_generator = unsafe { blake_round_generator.assume_init() };
        let blake_round_sigma_generator = unsafe { blake_round_sigma_generator.assume_init() };
        let mut blake_g_generator = unsafe { blake_g_generator.assume_init() };

        // Sequential blake gate/round/sigma (they depend on each other).
        // Spawn interpolation immediately after each write_trace.
        let (blake_gate_trace, blake_gate_icg, blake_message_state, blake_output_component_input) =
            blake_gate_claim_generator.write_trace(
                context_values,
                preprocessed_trace_ref,
                &verify_bitwise_xor_8_state,
                &range_check_16_state,
                &range_check_15_state,
                &mut blake_round_generator,
                &mut triple_xor_32_state,
            );
        blake_gate_interaction_claim_gen_result.write(blake_gate_icg);
        s.spawn(|_| {
            blake_gate_polys_result
                .write(SimdBackend::interpolate_columns(blake_gate_trace.to_evals(), twiddles));
        });

        let (blake_round_trace, blake_round_ls, blake_round_icg) = blake_round_generator
            .write_trace(
                &blake_round_sigma_generator,
                &blake_message_state,
                &mut blake_g_generator,
            );
        blake_round_log_size_result.write(blake_round_ls);
        blake_round_interaction_claim_gen_result.write(blake_round_icg);
        s.spawn(|_| {
            blake_round_polys_result
                .write(SimdBackend::interpolate_columns(blake_round_trace.to_evals(), twiddles));
        });

        let (blake_round_sigma_trace, _blake_round_sigma_claim, blake_round_sigma_icg) =
            blake_round_sigma_generator.write_trace();
        blake_round_sigma_interaction_claim_gen_result.write(blake_round_sigma_icg);
        s.spawn(|_| {
            blake_round_sigma_polys_result.write(SimdBackend::interpolate_columns(
                blake_round_sigma_trace.to_evals(),
                twiddles,
            ));
        });

        // blake_output is independent — spawn write_trace + interpolation directly.
        s.spawn(|_| {
            let blake_output_generator = blake_output::ClaimGenerator::new(
                blake_output_component_input,
                preprocessed_trace.clone(),
            );
            let (trace, claim, icg) = blake_output_generator.write_trace();
            blake_output_polys_result
                .write(SimdBackend::interpolate_columns(trace.to_evals(), twiddles));
            blake_output_claim_icg.write((claim, icg));
        });

        // blake_g, triple_xor_32, triple_xor gate, m31_to_u32, and blake_g_gate mutate
        // xor/range-check states through shared refs, so they must complete before those
        // states are consumed.
        scope(|s| {
            s.spawn(|_| {
                let (trace, claim, icg) = blake_g_generator.write_trace(
                    &verify_bitwise_xor_8_state,
                    &verify_bitwise_xor_12_state,
                    &verify_bitwise_xor_4_state,
                    &verify_bitwise_xor_7_state,
                    &verify_bitwise_xor_9_state,
                );
                blake_g_trace_data.write(trace);
                blake_g_claim_icg.write((claim, icg));
            });
            s.spawn(|_| {
                let (trace, claim, icg) =
                    triple_xor_32_state.write_trace(&verify_bitwise_xor_8_state);
                triple_xor_32_trace_data.write(trace);
                triple_xor_32_claim_icg.write((claim, icg));
            });
            s.spawn(|_| {
                let (trace, claim, icg) = triple_xor::write_trace(
                    context_values,
                    preprocessed_trace_ref,
                    &verify_bitwise_xor_8_state,
                );
                triple_xor_trace_data.write(trace);
                triple_xor_claim_icg.write((claim, icg));
            });
            s.spawn(|_| {
                let (trace, claim, icg) = m_31_to_u_32::write_trace(
                    context_values,
                    preprocessed_trace_ref,
                    &range_check_16_state,
                );
                m_31_to_u_32_trace_data.write(trace);
                m_31_to_u_32_claim_icg.write((claim, icg));
            });
            s.spawn(|_| {
                let (trace, claim, icg) = blake_g_gate::write_trace(
                    context_values,
                    preprocessed_trace_ref,
                    &verify_bitwise_xor_8_state,
                    &verify_bitwise_xor_12_state,
                    &verify_bitwise_xor_4_state,
                    &verify_bitwise_xor_9_state,
                    &verify_bitwise_xor_7_state,
                );
                blake_g_gate_trace_data.write(trace);
                blake_g_gate_claim_icg.write((claim, icg));
            });
        });

        // SAFETY: The subscope above guarantees blake_g, triple_xor_32, triple_xor,
        // m_31_to_u_32, and blake_g_gate have finished mutating xor/range-check states.
        // Interpolations and xor/range-check write_trace calls can now run in parallel.
        s.spawn(|_| {
            let trace = unsafe { blake_g_trace_data.assume_init() };
            blake_g_polys_result
                .write(SimdBackend::interpolate_columns(trace.to_evals(), twiddles));
        });
        s.spawn(|_| {
            let trace = unsafe { triple_xor_32_trace_data.assume_init() };
            triple_xor_32_polys_result
                .write(SimdBackend::interpolate_columns(trace.to_evals(), twiddles));
        });
        s.spawn(|_| {
            let trace = unsafe { triple_xor_trace_data.assume_init() };
            triple_xor_polys_result
                .write(SimdBackend::interpolate_columns(trace.to_evals(), twiddles));
        });
        s.spawn(|_| {
            let trace = unsafe { m_31_to_u_32_trace_data.assume_init() };
            m_31_to_u_32_polys_result
                .write(SimdBackend::interpolate_columns(trace.to_evals(), twiddles));
        });
        s.spawn(|_| {
            let trace = unsafe { blake_g_gate_trace_data.assume_init() };
            blake_g_gate_polys_result
                .write(SimdBackend::interpolate_columns(trace.to_evals(), twiddles));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = verify_bitwise_xor_8_state.write_trace();
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            verify_bitwise_xor_8_result.write((polys, interaction_claim_gen));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = verify_bitwise_xor_12_state.write_trace();
            // verify_bitwise_xor_12 returns Vec<CircleEvaluation> directly, not ComponentTrace.
            let polys = SimdBackend::interpolate_columns(trace, twiddles);
            verify_bitwise_xor_12_result.write((polys, interaction_claim_gen));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = verify_bitwise_xor_4_state.write_trace();
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            verify_bitwise_xor_4_result.write((polys, interaction_claim_gen));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = verify_bitwise_xor_7_state.write_trace();
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            verify_bitwise_xor_7_result.write((polys, interaction_claim_gen));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = verify_bitwise_xor_9_state.write_trace();
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            verify_bitwise_xor_9_result.write((polys, interaction_claim_gen));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = range_check_15_state.write_trace();
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            range_check_15_result.write((polys, interaction_claim_gen));
        });
        s.spawn(|_| {
            let (trace, _claim, interaction_claim_gen) = range_check_16_state.write_trace();
            let polys = SimdBackend::interpolate_columns(trace.to_evals(), twiddles);
            range_check_16_result.write((polys, interaction_claim_gen));
        });
    });

    // SAFETY: All MaybeUninit values were initialized by the scope above.
    let (
        (eq_polys, eq_log_size, eq_lookup_data),
        (qm31_ops_polys, qm31_ops_log_size, qm31_ops_lookup_data),
        blake_gate_interaction_claim_gen,
        blake_round_log_size,
        blake_round_interaction_claim_gen,
        blake_round_sigma_interaction_claim_gen,
        blake_g_polys,
        (blake_g_claim, blake_g_interaction_claim_gen),
        blake_output_polys,
        (blake_output_claim, blake_output_interaction_claim_gen),
        triple_xor_32_polys,
        (triple_xor_32_claim, triple_xor_32_interaction_claim_gen),
        triple_xor_polys,
        (triple_xor_claim, triple_xor_interaction_claim_gen),
        m_31_to_u_32_polys,
        (m_31_to_u_32_claim, m_31_to_u_32_interaction_claim_gen),
        blake_g_gate_polys,
        (blake_g_gate_claim, blake_g_gate_interaction_claim_gen),
        (verify_bitwise_xor_8_polys, verify_bitwise_xor_8_interaction_claim_gen),
        (verify_bitwise_xor_12_polys, verify_bitwise_xor_12_interaction_claim_gen),
        (verify_bitwise_xor_4_polys, verify_bitwise_xor_4_interaction_claim_gen),
        (verify_bitwise_xor_7_polys, verify_bitwise_xor_7_interaction_claim_gen),
        (verify_bitwise_xor_9_polys, verify_bitwise_xor_9_interaction_claim_gen),
        (range_check_15_polys, range_check_15_interaction_claim_gen),
        (range_check_16_polys, range_check_16_interaction_claim_gen),
        blake_gate_polys,
        blake_round_polys,
        blake_round_sigma_polys,
    ) = unsafe {
        (
            eq_result.assume_init(),
            qm31_ops_result.assume_init(),
            blake_gate_interaction_claim_gen_result.assume_init(),
            blake_round_log_size_result.assume_init(),
            blake_round_interaction_claim_gen_result.assume_init(),
            blake_round_sigma_interaction_claim_gen_result.assume_init(),
            blake_g_polys_result.assume_init(),
            blake_g_claim_icg.assume_init(),
            blake_output_polys_result.assume_init(),
            blake_output_claim_icg.assume_init(),
            triple_xor_32_polys_result.assume_init(),
            triple_xor_32_claim_icg.assume_init(),
            triple_xor_polys_result.assume_init(),
            triple_xor_claim_icg.assume_init(),
            m_31_to_u_32_polys_result.assume_init(),
            m_31_to_u_32_claim_icg.assume_init(),
            blake_g_gate_polys_result.assume_init(),
            blake_g_gate_claim_icg.assume_init(),
            verify_bitwise_xor_8_result.assume_init(),
            verify_bitwise_xor_12_result.assume_init(),
            verify_bitwise_xor_4_result.assume_init(),
            verify_bitwise_xor_7_result.assume_init(),
            verify_bitwise_xor_9_result.assume_init(),
            range_check_15_result.assume_init(),
            range_check_16_result.assume_init(),
            blake_gate_polys_result.assume_init(),
            blake_round_polys_result.assume_init(),
            blake_round_sigma_polys_result.assume_init(),
        )
    };

    tree_builder.extend_polys(eq_polys);
    tree_builder.extend_polys(qm31_ops_polys);
    tree_builder.extend_polys(blake_gate_polys);
    tree_builder.extend_polys(blake_round_polys);
    tree_builder.extend_polys(blake_round_sigma_polys);
    tree_builder.extend_polys(blake_g_polys);
    tree_builder.extend_polys(blake_output_polys);
    tree_builder.extend_polys(triple_xor_32_polys);
    tree_builder.extend_polys(triple_xor_polys);
    tree_builder.extend_polys(m_31_to_u_32_polys);
    tree_builder.extend_polys(blake_g_gate_polys);
    tree_builder.extend_polys(verify_bitwise_xor_8_polys);
    tree_builder.extend_polys(verify_bitwise_xor_12_polys);
    tree_builder.extend_polys(verify_bitwise_xor_4_polys);
    tree_builder.extend_polys(verify_bitwise_xor_7_polys);
    tree_builder.extend_polys(verify_bitwise_xor_9_polys);
    tree_builder.extend_polys(range_check_15_polys);
    tree_builder.extend_polys(range_check_16_polys);

    let output_values = output_addresses.iter().map(|addr| context_values[*addr]).collect_vec();

    (
        CircuitClaim {
            log_sizes: [
                eq_log_size,
                qm31_ops_log_size,
                blake_gate_interaction_claim_gen.log_size,
                blake_round_log_size.log_size,
                circuit_air::components::blake_round_sigma::LOG_SIZE,
                blake_g_claim.log_size,
                blake_output_claim.log_size,
                triple_xor_32_claim.log_size,
                triple_xor_claim.log_size,
                m_31_to_u_32_claim.log_size,
                blake_g_gate_claim.log_size,
                circuit_air::components::verify_bitwise_xor_8::LOG_SIZE,
                circuit_air::components::verify_bitwise_xor_12::LOG_SIZE,
                circuit_air::components::verify_bitwise_xor_4::LOG_SIZE,
                circuit_air::components::verify_bitwise_xor_7::LOG_SIZE,
                circuit_air::components::verify_bitwise_xor_9::LOG_SIZE,
                circuit_air::components::range_check_15::LOG_SIZE,
                circuit_air::components::range_check_16::LOG_SIZE,
            ],
            output_values,
        },
        CircuitInteractionClaimGenerator {
            eq_lookup_data,
            qm31_ops_lookup_data,
            blake_gate: blake_gate_interaction_claim_gen,
            blake_round: blake_round_interaction_claim_gen,
            blake_round_sigma: blake_round_sigma_interaction_claim_gen,
            blake_g: blake_g_interaction_claim_gen,
            blake_output: blake_output_interaction_claim_gen,
            triple_xor_32: triple_xor_32_interaction_claim_gen,
            triple_xor: triple_xor_interaction_claim_gen,
            m_31_to_u_32: m_31_to_u_32_interaction_claim_gen,
            blake_g_gate: blake_g_gate_interaction_claim_gen,
            verify_bitwise_xor_8: verify_bitwise_xor_8_interaction_claim_gen,
            verify_bitwise_xor_12: verify_bitwise_xor_12_interaction_claim_gen,
            verify_bitwise_xor_4: verify_bitwise_xor_4_interaction_claim_gen,
            verify_bitwise_xor_7: verify_bitwise_xor_7_interaction_claim_gen,
            verify_bitwise_xor_9: verify_bitwise_xor_9_interaction_claim_gen,
            range_check_15: range_check_15_interaction_claim_gen,
            range_check_16: range_check_16_interaction_claim_gen,
        },
    )
}

pub struct CircuitInteractionClaimGenerator {
    pub eq_lookup_data: eq::LookupData,
    pub qm31_ops_lookup_data: qm31_ops::LookupData,
    pub blake_gate: blake_gate::InteractionClaimGenerator,
    pub blake_round: blake_round::InteractionClaimGenerator,
    pub blake_round_sigma: blake_round_sigma::InteractionClaimGenerator,
    pub blake_g: blake_g::InteractionClaimGenerator,
    pub blake_output: blake_output::InteractionClaimGenerator,
    pub triple_xor_32: triple_xor_32::InteractionClaimGenerator,
    pub triple_xor: triple_xor::InteractionClaimGenerator,
    pub m_31_to_u_32: m_31_to_u_32::InteractionClaimGenerator,
    pub blake_g_gate: blake_g_gate::InteractionClaimGenerator,
    pub verify_bitwise_xor_8: verify_bitwise_xor_8::InteractionClaimGenerator,
    pub verify_bitwise_xor_12: verify_bitwise_xor_12::InteractionClaimGenerator,
    pub verify_bitwise_xor_4: verify_bitwise_xor_4::InteractionClaimGenerator,
    pub verify_bitwise_xor_7: verify_bitwise_xor_7::InteractionClaimGenerator,
    pub verify_bitwise_xor_9: verify_bitwise_xor_9::InteractionClaimGenerator,
    pub range_check_15: range_check_15::InteractionClaimGenerator,
    pub range_check_16: range_check_16::InteractionClaimGenerator,
}

pub fn write_interaction_trace(
    circuit_claim: &CircuitClaim,
    circuit_interaction_claim_generator: CircuitInteractionClaimGenerator,
    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sM31MerkleChannel>,
    interaction_elements: &CircuitInteractionElements,
    twiddles: &TwiddleTree<SimdBackend>,
) -> CircuitInteractionClaim {
    let CircuitClaim { log_sizes, output_values: _ } = circuit_claim;
    let mut component_log_size_iter = log_sizes.iter();

    // Extract log sizes before parallel section.
    let eq_log_size = *component_log_size_iter.next().unwrap();
    let qm31_ops_log_size = *component_log_size_iter.next().unwrap();

    // Write all interaction traces in parallel, including interpolation.
    // Preallocate poly slots; each spawned task writes into its own mutable slice.
    let mut all_polys: [Vec<_>; 18] = std::array::from_fn(|_| Vec::new());
    let [
        eq_polys,
        qm31_ops_polys,
        blake_gate_polys,
        blake_round_polys,
        blake_round_sigma_polys,
        blake_g_polys,
        blake_output_polys,
        triple_xor_32_polys,
        triple_xor_polys,
        m_31_to_u_32_polys,
        blake_g_gate_polys,
        verify_bitwise_xor_8_polys,
        verify_bitwise_xor_12_polys,
        verify_bitwise_xor_4_polys,
        verify_bitwise_xor_7_polys,
        verify_bitwise_xor_9_polys,
        range_check_15_polys,
        range_check_16_polys,
    ] = &mut all_polys;
    let mut claimed_sums = [QM31::zero(); 18];
    let [
        eq_claimed_sum,
        qm31_ops_claimed_sum,
        blake_gate_claimed_sum,
        blake_round_claimed_sum,
        blake_round_sigma_claimed_sum,
        blake_g_claimed_sum,
        blake_output_claimed_sum,
        triple_xor_32_claimed_sum,
        triple_xor_claimed_sum,
        m_31_to_u_32_claimed_sum,
        blake_g_gate_claimed_sum,
        verify_bitwise_xor_8_claimed_sum,
        verify_bitwise_xor_12_claimed_sum,
        verify_bitwise_xor_4_claimed_sum,
        verify_bitwise_xor_7_claimed_sum,
        verify_bitwise_xor_9_claimed_sum,
        range_check_15_claimed_sum,
        range_check_16_claimed_sum,
    ] = &mut claimed_sums;
    {
        let eq_lookup_data = circuit_interaction_claim_generator.eq_lookup_data;
        let qm31_ops_lookup_data = circuit_interaction_claim_generator.qm31_ops_lookup_data;
        let blake_gate = circuit_interaction_claim_generator.blake_gate;
        let blake_round = circuit_interaction_claim_generator.blake_round;
        let blake_round_sigma = circuit_interaction_claim_generator.blake_round_sigma;
        let blake_g = circuit_interaction_claim_generator.blake_g;
        let blake_output = circuit_interaction_claim_generator.blake_output;
        let triple_xor_32 = circuit_interaction_claim_generator.triple_xor_32;
        let triple_xor = circuit_interaction_claim_generator.triple_xor;
        let m_31_to_u_32 = circuit_interaction_claim_generator.m_31_to_u_32;
        let blake_g_gate = circuit_interaction_claim_generator.blake_g_gate;
        let verify_bitwise_xor_8 = circuit_interaction_claim_generator.verify_bitwise_xor_8;
        let verify_bitwise_xor_12 = circuit_interaction_claim_generator.verify_bitwise_xor_12;
        let verify_bitwise_xor_4 = circuit_interaction_claim_generator.verify_bitwise_xor_4;
        let verify_bitwise_xor_7 = circuit_interaction_claim_generator.verify_bitwise_xor_7;
        let verify_bitwise_xor_9 = circuit_interaction_claim_generator.verify_bitwise_xor_9;
        let range_check_15 = circuit_interaction_claim_generator.range_check_15;
        let range_check_16 = circuit_interaction_claim_generator.range_check_16;
        scope(|s| {
            s.spawn(|_| {
                let (trace, claimed_sum) = eq::write_interaction_trace(
                    eq_log_size,
                    eq_lookup_data,
                    &interaction_elements.common_lookup_elements,
                );
                *eq_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *eq_claimed_sum = claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claimed_sum) = qm31_ops::write_interaction_trace(
                    qm31_ops_log_size,
                    qm31_ops_lookup_data,
                    &interaction_elements.common_lookup_elements,
                );
                *qm31_ops_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *qm31_ops_claimed_sum = claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = blake_gate
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *blake_gate_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *blake_gate_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = blake_round
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *blake_round_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *blake_round_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = blake_round_sigma
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *blake_round_sigma_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *blake_round_sigma_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) =
                    blake_g.write_interaction_trace(&interaction_elements.common_lookup_elements);
                *blake_g_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *blake_g_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = blake_output
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *blake_output_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *blake_output_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = triple_xor_32
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *triple_xor_32_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *triple_xor_32_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = triple_xor
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *triple_xor_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *triple_xor_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = m_31_to_u_32
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *m_31_to_u_32_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *m_31_to_u_32_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = blake_g_gate
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *blake_g_gate_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *blake_g_gate_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = verify_bitwise_xor_8
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *verify_bitwise_xor_8_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *verify_bitwise_xor_8_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = verify_bitwise_xor_12
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *verify_bitwise_xor_12_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *verify_bitwise_xor_12_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = verify_bitwise_xor_4
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *verify_bitwise_xor_4_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *verify_bitwise_xor_4_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = verify_bitwise_xor_7
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *verify_bitwise_xor_7_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *verify_bitwise_xor_7_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = verify_bitwise_xor_9
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *verify_bitwise_xor_9_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *verify_bitwise_xor_9_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = range_check_15
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *range_check_15_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *range_check_15_claimed_sum = claim.claimed_sum;
            });
            s.spawn(|_| {
                let (trace, claim) = range_check_16
                    .write_interaction_trace(&interaction_elements.common_lookup_elements);
                *range_check_16_polys = SimdBackend::interpolate_columns(trace, twiddles);
                *range_check_16_claimed_sum = claim.claimed_sum;
            });
        });
    }

    tree_builder.extend_polys(all_polys.into_iter().flatten());

    CircuitInteractionClaim { claimed_sums }
}
