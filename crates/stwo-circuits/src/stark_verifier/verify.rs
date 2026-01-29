use std::collections::HashMap;

use itertools::{Itertools, chain, izip, zip_eq};
use stwo::core::circle::CirclePoint;
use stwo::core::fields::m31::P;

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::eq;
use crate::circuits::simd::Simd;
use crate::circuits::wrappers::M31Wrapper;
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
    enable_bits: &[Var],
) {
    let mut log_up_sum = public_logup_sum;
    for (claimed_sum, enable_bit) in zip_eq(claimed_sums, enable_bits) {
        log_up_sum = eval!(context, (log_up_sum) + ((*claimed_sum) * (*enable_bit)));
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
    // Since LOG_SIZE_BITS is 5, and 2**31 - 1 = M31, we need to check that not all the bits in the
    // component log sizes are ones.
    Simd::assert_not_all_ones(context, &component_log_size_bits);

    let component_sizes = Simd::pow2(context, &component_log_size_bits);
    // Check that the component sizes are at most 2^config.log_trace_size().
    // Note that we need k + 1 bits to represent 2^k.
    let component_sizes_bits =
        extract_bits(context, &component_sizes, config.log_trace_size() as u32 + 1);

    channel.mix_qm31s(context, proof.claim.packed_enable_bits.iter().cloned());
    channel.mix_qm31s(context, proof.claim.packed_component_log_sizes.iter().cloned());
    for claim_to_mix in statement.claims_to_mix(context) {
        channel.mix_qm31s(context, claim_to_mix.iter().cloned());
    }

    // Mix the trace commitments into the channel.
    channel.mix_commitment(context, proof.trace_root);

    channel.proof_of_work(
        context,
        INTERACTION_POW_BITS.try_into().unwrap(),
        proof.interaction_pow_nonce,
    );
    // Pick the interaction elements.
    let [interaction_z, interaction_alpha] = channel.draw_two_qm31s(context);

    let simd_enable_bits =
        Simd::from_packed(proof.claim.packed_enable_bits.clone(), config.n_components);
    simd_enable_bits.assert_bits(context);
    let enable_bits = Simd::unpack(context, &simd_enable_bits);
    let public_logup_sum =
        statement.public_logup_sum(context, [interaction_z, interaction_alpha], &proof.claim);
    validate_logup_sum(context, public_logup_sum, &proof.claim.claimed_sums, &enable_bits);

    channel.mix_qm31s(context, proof.claim.claimed_sums.iter().cloned());
    channel.mix_commitment(context, proof.interaction_root);

    // Draw a random QM31 coefficient for the composition polynomial.
    let composition_polynomial_coeff = channel.draw_qm31(context);

    channel.mix_commitment(context, proof.composition_polynomial_root);

    // Draw a random point for the OODS.
    let oods_point = channel.draw_point(context);

    let unpacked_component_sizes = Simd::unpack(context, &component_sizes);
    check_relation_uses(context, statement, &component_sizes_bits);

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

/// Verify that no relation is used more than P times.
/// For each relation, it verifies that sum(uses_per_row * num_rows) < P
/// where the sum is over all the components that use the relation.
///
/// To avoid overflows when computing the sum, we check
/// sum(uses_per_row * (floor(num_rows / DIV) + 1)) < floor(P / DIV)
/// where DIV = 2 ** NUM_ROWS_SHIFT
fn check_relation_uses<Value: IValue>(
    context: &mut Context<impl IValue>,
    statement: &impl Statement<Value>,
    component_sizes_bits: &[Simd],
) {
    const NUM_ROWS_SHIFT: usize = 16;
    let components = statement.get_components();

    // Check that sum(uses_per_row * (floor(num_rows / DIV) + 1)) cannot overflow even for the
    // maximal num_rows (num_rows = P).
    // This is a sanity check that `NUM_ROWS_SHIFT` is large enough for the given statement, it
    // does not depend on the specific assigment.
    let mut max_shifted_uses_per_relation = HashMap::<&str, u64>::new();
    for component in components.iter() {
        for relation_use in component.relation_uses_per_row() {
            let entry = max_shifted_uses_per_relation.entry(relation_use.relation_id).or_insert(0);
            *entry += relation_use.uses * (((P >> NUM_ROWS_SHIFT) + 1) as u64);
        }
    }
    assert!(max_shifted_uses_per_relation.values().all(|count| *count < (P as u64)));

    // Compute floor(num_rows / DIV) for all components
    let shifted_component_sizes = match component_sizes_bits.get(NUM_ROWS_SHIFT..) {
        Some(high_bits) => Simd::combine_bits(context, high_bits),
        None => Simd::zero(context, components.len()),
    };
    // A variable in the Simd vector might be unused in the case where all the corresponding
    // components don't use any relations.
    Simd::mark_partly_used(context, &shifted_component_sizes);

    // Sum uses_per_row * (floor(num_rows / DIV) + 1) for all relations
    let mut shifted_relation_uses = HashMap::new();
    for (i, component) in components.iter().enumerate() {
        let relation_uses = component.relation_uses_per_row();
        if relation_uses.is_empty() {
            continue;
        }
        let shifted_size = Simd::unpack_idx(context, &shifted_component_sizes, i);
        for relation_use in component.relation_uses_per_row() {
            let entry =
                shifted_relation_uses.entry(relation_use.relation_id).or_insert(context.zero());
            let uses_per_row =
                context.constant(TryInto::<u32>::try_into(relation_use.uses).unwrap().into());
            *entry = eval!(context, (*entry) + (((shifted_size) + (1)) * (uses_per_row)));
        }
    }

    // Verify that the sum is less than floor(P / DIV) by expressing it as a
    // floor(log2(P / DIV))-bit number
    let shifted_use_counts = shifted_relation_uses
        .into_iter()
        .sorted_by_key(|(k, _v)| *k)
        .map(|(_k, v)| M31Wrapper::new_unsafe(v))
        .collect_vec();
    let shifted_use_counts = Simd::pack(context, &shifted_use_counts);
    extract_bits(context, &shifted_use_counts, (P >> NUM_ROWS_SHIFT).ilog2());
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
