use itertools::{Itertools, chain};

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::eq;
use crate::circuits::simd::Simd;
use crate::eval;
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::extract_bits::extract_bits;
use crate::stark_verifier::fri::{fri_commit, fri_decommit};
use crate::stark_verifier::merkle::decommit_eval_domain_samples;
use crate::stark_verifier::oods::{
    collect_oods_responses, compute_fri_input, extract_expected_composition_eval,
};
use crate::stark_verifier::proof::{Proof, ProofConfig};
use crate::stark_verifier::select_queries::{
    get_query_selection_input_from_channel, select_queries,
};
use crate::stark_verifier::statement::{EvaluateArgs, OodsSamples, Statement};

pub const LOG_SIZE_BITS: usize = 5;

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

pub fn verify(
    context: &mut Context<impl IValue>,
    proof: &Proof<Var>,
    config: &ProofConfig,
    statement: &impl Statement,
) {
    proof.validate_structure(config);

    let mut channel = Channel::new(context);

    let component_log_sizes =
        Simd::from_packed(proof.component_log_sizes.clone(), config.n_components);

    // Range check the component log sizes.
    let component_log_size_bits = extract_bits::<LOG_SIZE_BITS>(context, &component_log_sizes);

    channel.mix_qm31s(context, proof.component_log_sizes.iter().cloned());

    let column_log_sizes = statement.column_log_sizes(Simd::unpack(context, &component_log_sizes));

    // Mix the trace commitments into the channel.
    channel.mix_commitment(context, proof.preprocessed_root);
    channel.mix_commitment(context, proof.trace_root);

    // TODO(lior): Add proof of work before drawing the interaction elements.

    // Pick the interaction elements.
    let [interaction_z, interaction_alpha] = channel.draw_two_qm31s(context);

    let public_logup_sum = statement.public_logup_sum(context, [interaction_z, interaction_alpha]);
    validate_logup_sum(context, public_logup_sum, &proof.claimed_sums);

    channel.mix_qm31s(context, proof.claimed_sums.iter().cloned());
    channel.mix_commitment(context, proof.interaction_root);

    // Draw a random QM31 coefficient for the composition polynomial.
    let composition_polynomial_coeff = channel.draw_qm31(context);

    channel.mix_commitment(context, proof.composition_polynomial_root);

    // Draw a random point for the OODS.
    let oods_point = channel.draw_point(context);

    // Compute the composition evaluation at the OODS point from `proof.*_at_oods` and compare
    // to `proof.composition_eval_at_oods`.

    let component_sizes_simd = Simd::pow2(context, &component_log_size_bits);
    let component_sizes = Simd::unpack(context, &component_sizes_simd);
    let composition_eval = statement.evaluate(
        context,
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
            claimed_sums: &proof.claimed_sums,
            component_sizes: &component_sizes,
        },
    );
    let expected_composition_eval = extract_expected_composition_eval(
        context,
        &proof.composition_eval_at_oods,
        oods_point,
        config.log_evaluation_domain_size(),
    );
    eq(context, composition_eval, expected_composition_eval);

    // Verify the values in `proof.trace_at_oods` and `proof.composition_eval_at_oods`.
    // Start by adding the values to the channel.
    channel.mix_qm31s(
        context,
        chain!(
            proof.preprocessed_columns_at_oods.iter().cloned(),
            proof.trace_at_oods.iter().cloned(),
            proof
                .interaction_at_oods
                .iter()
                .flat_map(|interaction| [interaction.at_prev, interaction.at_oods]),
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

    decommit_eval_domain_samples(
        context,
        config.n_queries(),
        &column_log_sizes,
        &proof.eval_domain_samples,
        &proof.eval_domain_auth_paths,
        &bits,
        &proof.merkle_roots(),
    );

    // Compute FRI input.
    let oods_responses = collect_oods_responses(context, config, oods_point, proof);
    let fri_input = compute_fri_input(
        context,
        &oods_responses,
        &queries,
        &proof.eval_domain_samples,
        oods_quotient_coef,
    );

    fri_decommit(context, &proof.fri, &config.fri, &fri_input, &bits, &queries.points, &fri_alphas);
}
