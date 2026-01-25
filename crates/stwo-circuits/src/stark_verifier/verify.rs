use itertools::{Itertools, chain, izip};
use stwo::core::circle::CirclePoint;

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::eq;
use crate::circuits::simd::Simd;
use crate::eval;
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::circle::{add_points, generator_point};
use crate::stark_verifier::constraint_eval::compute_composition_polynomial;
use crate::stark_verifier::extract_bits::extract_bits;
use crate::stark_verifier::fri::{fri_commit, fri_decommit};
use crate::stark_verifier::merkle::decommit_eval_domain_samples;
use crate::stark_verifier::oods::{
    collect_oods_responses, compute_fri_input, extract_expected_composition_eval, period_generators,
};
use crate::stark_verifier::proof::{Proof, ProofConfig};
use crate::stark_verifier::select_queries::{
    get_query_selection_input_from_channel, select_queries,
};
use crate::stark_verifier::statement::{EvaluateArgs, OodsSamples, Statement};

pub const LOG_SIZE_BITS: u32 = 5;

// The number of bits required to represent the size of the largest component supported.
// Note that this is one more than log2(max_component_size), because 2**n is a (n+1)-bit
// number.
pub const MAX_TRACE_SIZE_BITS: u32 = 29;

/// Logup security is defined by the `QM31` space (~124 bits) + `INTERACTION_POW_BITS` -
/// log2(number of relation terms).
/// The number of relation terms is defined as n_terms * n_relations * n_uses, where:
/// n_terms = number of terms in each relation (the size of the relation entry) < 2^7,
/// n_relations = number of different relations ids < 2^6,
/// n_uses is bounded by the characteristic of the field = 2^31.
/// E.g. assuming a 100-bit security target, the witness may contain up to
/// 1 << (24 + INTERACTION_POW_BITS) relation terms.
#[cfg(not(test))]
pub const INTERACTION_POW_BITS: u32 = 24;
#[cfg(test)]
pub const INTERACTION_POW_BITS: u32 = 8; // Lower value for faster tests

#[cfg(test)]
#[path = "verify_test.rs"]
pub mod test;

pub fn validate_logup_sum(
    context: &mut Context<impl IValue>,
    public_logup_sum: Var,
    claimed_sums: &[Var],
) {
    let mut log_up_sum = public_logup_sum;
    for claimed_sum in claimed_sums {
        log_up_sum = eval!(context, (log_up_sum) + (*claimed_sum));
    }
    eq(context, log_up_sum, context.zero());
}

pub fn verify<Value: IValue>(
    context: &mut Context<Value>,
    proof: &Proof<Var>,
    config: &ProofConfig,
    statement: &impl Statement<Value>,
) {
    proof.validate_structure(config);

    let mut channel = Channel::new(context);

    // Mix the trace commitments into the channel.
    channel.mix_commitment(context, proof.preprocessed_root);

    let component_log_sizes =
        Simd::from_packed(proof.claim.packed_component_log_sizes.clone(), config.n_components);

    // Range check the component log sizes.
    let component_log_size_bits = extract_bits(context, &component_log_sizes, LOG_SIZE_BITS);
    // TODO(ilya): check that all the component log sizes are smaller than config.log_trace_size().

    channel.mix_qm31s(context, proof.claim.packed_enable_bits.iter().cloned());
    channel.mix_qm31s(context, proof.claim.packed_component_log_sizes.iter().cloned());
    channel.mix_qm31s(context, proof.claim.public_claim.iter().cloned());

    // Mix the trace commitments into the channel.
    channel.mix_commitment(context, proof.trace_root);

    channel.proof_of_work(
        context,
        INTERACTION_POW_BITS.try_into().unwrap(),
        proof.interaction_pow_nonce,
    );
    // Pick the interaction elements.
    let [interaction_z, interaction_alpha] = channel.draw_two_qm31s(context);

    let public_logup_sum =
        statement.public_logup_sum(context, [interaction_z, interaction_alpha], &proof.claim);
    validate_logup_sum(context, public_logup_sum, &proof.claim.claimed_sums);

    channel.mix_qm31s(context, proof.claim.claimed_sums.iter().cloned());
    channel.mix_commitment(context, proof.interaction_root);

    // Draw a random QM31 coefficient for the composition polynomial.
    let composition_polynomial_coeff = channel.draw_qm31(context);

    channel.mix_commitment(context, proof.composition_polynomial_root);

    // Draw a random point for the OODS.
    let oods_point = channel.draw_point(context);

    let component_sizes = Simd::pow2(context, &component_log_size_bits);
    let unpacked_component_sizes = Simd::unpack(context, &component_sizes);
    let component_sizes_bits = extract_bits(context, &component_sizes, MAX_TRACE_SIZE_BITS);

    let simd_enable_bits =
        Simd::from_packed(proof.claim.packed_enable_bits.clone(), config.n_components);
    simd_enable_bits.assert_bits(context);
    let enable_bits = Simd::unpack(context, &simd_enable_bits);

    // Compute the composition evaluation at the OODS point from `proof.*_at_oods` and compare
    // to `proof.composition_eval_at_oods`.
    let composition_eval = compute_composition_polynomial(
        context,
        config,
        statement,
        EvaluateArgs {
            oods_samples: OodsSamples {
                preprocessed_columns: &proof.preprocessed_columns_at_oods,
                trace: &proof.trace_at_oods,
                interaction: &proof.interaction_at_oods,
            },
            pt: oods_point,
            log_domain_size: config.log_trace_size(),
            composition_polynomial_coeff,
            interaction_elements: [interaction_z, interaction_alpha],
            claimed_sums: &proof.claim.claimed_sums,
            enable_bits: &enable_bits,
            component_sizes: &unpacked_component_sizes,
            n_instances_bits: &component_sizes_bits,
        },
    );
    let expected_composition_eval = extract_expected_composition_eval(
        context,
        &proof.composition_eval_at_oods,
        oods_point,
        config.log_evaluation_domain_size(),
    );
    eq(context, composition_eval, expected_composition_eval);

    // The generator of the trace subgroup on the circle.
    let trace_gen = generator_point(config.log_trace_size());

    let period_generators_per_component =
        period_generators(context, trace_gen, &component_sizes_bits);
    let periodicity_sample_points_per_component = period_generators_per_component
        .into_iter()
        .map(|pt| add_points(context, &oods_point, &pt))
        .collect_vec();

    // Verify the values in `proof.trace_at_oods` and `proof.composition_eval_at_oods`.
    // Start by adding the values to the channel. Values belonging to cumulative sum columns are
    // added twice, once for the previous point and once for the OODS point.
    let interaction_at_oods = proof
        .interaction_at_oods
        .iter()
        .flat_map(|interaction| {
            if let Some(interaction_at_prev) = interaction.at_prev {
                vec![interaction_at_prev, interaction.at_oods]
            } else {
                vec![interaction.at_oods]
            }
        })
        .collect_vec();
    channel.mix_qm31s(
        context,
        chain!(
            proof.preprocessed_columns_at_oods.iter().cloned(),
            proof.trace_at_oods.iter().cloned(),
            interaction_at_oods,
            proof.composition_eval_at_oods,
        ),
    );

    // Draw a random challenge for the linear combination of the OODS quotients.
    let oods_quotient_coef = channel.draw_qm31(context);

    // Run the commit phase of FRI.
    let fri_alphas = fri_commit(context, &mut channel, &proof.fri.commit);

    // Proof of work before query selection.
    channel.proof_of_work(context, config.n_proof_of_work_bits, proof.proof_of_work_nonce);

    // Select queries.
    let query_selection_input =
        get_query_selection_input_from_channel(context, &mut channel, config.n_queries());
    let queries =
        select_queries(context, &query_selection_input, config.log_evaluation_domain_size());

    // Check decommitment of trace queries.
    let bits = queries.bits.iter().map(|simd| Simd::unpack(context, simd)).collect_vec();

    let column_log_sizes_by_trace = column_log_sizes_by_trace(context, config, component_log_sizes);
    let periodicity_sample_points_per_column =
        column_periodicity_sample_points(config, &periodicity_sample_points_per_component);

    decommit_eval_domain_samples(
        context,
        config.n_queries(),
        &column_log_sizes_by_trace,
        &proof.eval_domain_samples,
        &proof.eval_domain_auth_paths,
        &bits,
        &proof.merkle_roots(),
    );

    // Compute FRI input.
    let oods_responses = collect_oods_responses(
        context,
        config,
        trace_gen,
        oods_point,
        &periodicity_sample_points_per_column,
        proof,
    );
    let fri_input = compute_fri_input(
        context,
        &oods_responses,
        &queries,
        &proof.eval_domain_samples,
        oods_quotient_coef,
    );

    fri_decommit(context, &proof.fri, &config.fri, &fri_input, &bits, &queries.points, &fri_alphas);
}

// Returns the column_log_sizes_by_trace, which includes the column log sizes for the trace and
// interaction columns.
fn column_log_sizes_by_trace(
    context: &mut Context<impl IValue>,
    config: &ProofConfig,
    component_log_sizes: Simd,
) -> [Vec<Var>; 2] {
    let mut column_log_sizes = [
        Vec::with_capacity(config.n_trace_columns),
        Vec::with_capacity(config.n_interaction_columns),
    ];

    for (n_trace_columns_in_component, n_interaction_columns_in_component, log_size) in izip!(
        &config.trace_columns_per_component,
        &config.interaction_columns_per_component,
        Simd::unpack(context, &component_log_sizes)
    ) {
        column_log_sizes[0].extend(vec![log_size; *n_trace_columns_in_component]);
        column_log_sizes[1].extend(vec![log_size; *n_interaction_columns_in_component]);
    }
    column_log_sizes
}

/// Given the periodicity sample points for each component, returns the periodicity sample points
/// for each column in the interaction trace.
/// The periodicity sample points are the sample points used for the periodicity check.
fn column_periodicity_sample_points(
    config: &ProofConfig,
    sample_points_per_component: &[CirclePoint<Var>],
) -> Vec<CirclePoint<Var>> {
    let mut periodicity_sample_points_per_column = Vec::with_capacity(config.n_interaction_columns);
    for (n_interaction_columns_in_component, sample_point) in
        izip!(&config.interaction_columns_per_component, sample_points_per_component)
    {
        periodicity_sample_points_per_column
            .extend(vec![sample_point; *n_interaction_columns_in_component]);
    }
    periodicity_sample_points_per_column
}
