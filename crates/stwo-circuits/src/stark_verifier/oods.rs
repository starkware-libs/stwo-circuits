use indexmap::IndexMap;
use itertools::{Itertools, chain, zip_eq};
use stwo::core::circle::CirclePoint;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::circuits::EXTENSION_DEGREE;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::{IValue, NoValue, qm31_from_u32s};
use crate::circuits::ops::{Guess, conj, div, from_partial_evals};
use crate::circuits::simd::Simd;
use crate::circuits::wrappers::M31Wrapper;
use crate::eval;
use crate::stark_verifier::circle::{add_points, double_x, generator_point};
use crate::stark_verifier::proof::{Proof, ProofConfig};
use crate::stark_verifier::select_queries::Queries;

const COMPOSITION_SPLIT: usize = 2;
pub const N_COMPOSITION_COLUMNS: usize = COMPOSITION_SPLIT * EXTENSION_DEGREE;

#[cfg(test)]
#[path = "oods_test.rs"]
pub mod test;

/// Represents the sampled values in the evaluation domain.
#[derive(Debug)]
pub struct EvalDomainSamples<T> {
    /// `data[trace_idx][query_idx][column_idx]` is the `M31` value of the column `column_idx` in
    /// trace `trace_idx` at the `query_idx` query.
    data: Vec<Vec<Vec<M31Wrapper<T>>>>,
}

impl<T> EvalDomainSamples<T> {
    pub fn n_traces(&self) -> usize {
        self.data.len()
    }

    /// Returns the sampled value for the given trace, query, and column.
    pub fn at(&self, trace_idx: usize, query_idx: usize, column_idx: usize) -> &T {
        self.data[trace_idx][query_idx][column_idx].get()
    }

    /// Returns the data vector for the given trace.
    pub fn data_for_trace(&self, trace_idx: usize) -> &[Vec<M31Wrapper<T>>] {
        &self.data[trace_idx]
    }

    /// Validates that the size of the vectors in the struct are consistent with the
    /// config parameters.
    pub fn validate_structure(&self, n_columns_per_trace: &[usize], n_queries: usize) {
        for (trace_data, n_columns) in zip_eq(&self.data, n_columns_per_trace) {
            assert_eq!(trace_data.len(), n_queries);
            for query_data in trace_data {
                assert_eq!(query_data.len(), *n_columns);
            }
        }
    }
}

impl EvalDomainSamples<QM31> {
    /// Constructs a new [EvalDomainSamples] from the given data.
    pub fn from_m31s(data: Vec<Vec<Vec<M31>>>) -> Self {
        Self {
            data: data
                .iter()
                .map(|v| v.iter().map(|v| v.iter().map(|v| (*v).into()).collect()).collect())
                .collect(),
        }
    }
}

impl<Value: IValue> Guess<Value> for EvalDomainSamples<Value> {
    type Target = EvalDomainSamples<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        EvalDomainSamples { data: self.data.guess(context) }
    }
}

pub fn empty_eval_domain_samples(
    n_columns_per_trace: &[usize],
    n_queries: usize,
) -> EvalDomainSamples<NoValue> {
    EvalDomainSamples {
        data: n_columns_per_trace
            .iter()
            .map(|n_columns| vec![vec![M31Wrapper::from(NoValue); *n_columns]; n_queries])
            .collect(),
    }
}

/// Computes the expected value of the composition polynomial at the OODS point, based on the
/// broken composition polynomial commitment.
pub fn extract_expected_composition_eval(
    context: &mut Context<impl IValue>,
    composition_eval_at_oods: &[Var; N_COMPOSITION_COLUMNS],
    oods_point: CirclePoint<Var>,
    log_evaluation_domain_size: usize,
) -> Var {
    let composition_eval_at_oods_left =
        from_partial_evals(context, composition_eval_at_oods[0..4].try_into().unwrap());
    let composition_eval_at_oods_right =
        from_partial_evals(context, composition_eval_at_oods[4..8].try_into().unwrap());

    // Compute:
    //  `x = pi^{log_evaluation_domain_size - 2}(oods_point.x) = pi(pi(...pi(oods_point.x)...))`.
    let mut x = oods_point.x;
    for _ in 0..log_evaluation_domain_size - 2 {
        x = double_x(context, x);
    }

    eval!(context, (composition_eval_at_oods_left) + ((x) * (composition_eval_at_oods_right)))
}

/// An OODS response, claiming that the value of column `column_idx` in trace `trace_idx`
/// at (OODS) point `pt` is `value`.
///
/// `pt` can be either the OODS point itself or its previous point:
///   `(OODS point) - (trace generator)`.
pub struct OodsResponse {
    /// The index of the trace.
    pub trace_idx: usize,
    /// The index of the column.
    pub column_idx: usize,
    /// The point to evaluate the column at.
    pub pt: CirclePoint<Var>,
    /// The expected value of the column at `pt`.
    pub value: Var,
}

/// Computes the list of [OodsResponse]s that will be validated using FRI.
///
/// The order is consistent with the order dictated by the stwo prover.
pub fn collect_oods_responses(
    context: &mut Context<impl IValue>,
    config: &ProofConfig,
    oods_point: CirclePoint<Var>,
    proof: &Proof<Var>,
) -> Vec<OodsResponse> {
    // The generator of the trace subgroup on the circle.
    let trace_gen: CirclePoint<M31> = generator_point(config.log_trace_size());
    // The negation of the trace generator, as `CirclePoint<Var>`.
    let neg_trace_gen: CirclePoint<Var> = CirclePoint {
        x: context.constant(trace_gen.x.into()),
        y: context.constant((-trace_gen.y).into()),
    };
    // The point: `oods_point - neg_trace_gen`.
    let oods_point_at_prev_row = add_points(context, &oods_point, &neg_trace_gen);

    // The order below is the order dictated by the stwo prover.
    // First, we have all the `OodsResponse`s for the OODS point, then for the previous point.
    chain!(
        (0..config.n_preprocessed_columns).map(|column_idx| OodsResponse {
            trace_idx: 0,
            column_idx,
            pt: oods_point,
            value: proof.preprocessed_columns_at_oods[column_idx],
        }),
        (0..config.n_trace_columns).map(|column_idx| OodsResponse {
            trace_idx: 1,
            column_idx,
            pt: oods_point,
            value: proof.trace_at_oods[column_idx],
        }),
        (0..config.n_interaction_columns).map(|column_idx| OodsResponse {
            trace_idx: 2,
            column_idx,
            pt: oods_point,
            value: proof.interaction_at_oods.at_oods(column_idx),
        }),
        (0..N_COMPOSITION_COLUMNS).map(|column_idx| OodsResponse {
            trace_idx: 3,
            column_idx,
            pt: oods_point,
            value: proof.composition_eval_at_oods[column_idx],
        }),
        (0..config.n_interaction_columns).map(|column_idx| OodsResponse {
            trace_idx: 2,
            column_idx,
            pt: oods_point_at_prev_row,
            value: proof.interaction_at_oods.at_prev(column_idx),
        }),
    )
    .collect()
}

/// In order to validate the [OodsResponse]s, we use FRI to show that the following rational
/// function is in fact a polynomial:
/// ```plain
///   (-2u) * sum_i (
///       alpha^i * (c * column[column_idx](x, y) - a * y - b) / (d * x - e * y - f)
///   )
/// ```
/// where:
/// ```plain
///    a = conj(value) - value
///    c = conj(pt.y) - pt.y
///    b = value * c - a * pt.y
///
///    d = pt.y - conj(pt.y)
///    e = pt.x - conj(pt.x)
///    f = d * pt.x - e * pt.y
/// ```
///
/// Note that:
/// (1) `d * x - e * y - f` vanishes on the points `pt` and `conj(pt)`.
/// (2) `(a * y + b) / c` evaluates to `value` at `pt.y` and to `conj(value)` at `conj(pt.y)`.
///
/// The function computes the inputs to FRI, which are the evaluations of the above rational
/// function at the given (evaluation domain) queries.
pub fn compute_fri_input(
    context: &mut Context<impl IValue>,
    oods_responses: &[OodsResponse],
    queries: &Queries,
    trace_queries: &EvalDomainSamples<Var>,
    alpha: Var,
) -> Vec<Var> {
    // TODO(lior): Make the function more efficient using similar techniques as in the Cairo version
    //   of the stwo verifier.

    // The coefficients d, e, f for each (pt.x, pt.y).
    let mut def = IndexMap::<(usize, usize), (Var, Var, Var)>::new();

    for r in oods_responses {
        let key = (r.pt.x.idx, r.pt.y.idx);
        if def.contains_key(&key) {
            continue;
        }

        let pt_x_conj = conj(context, r.pt.x);
        let pt_y_conj = conj(context, r.pt.y);
        let d = eval!(context, (r.pt.y) - (pt_y_conj));
        let e = eval!(context, (r.pt.x) - (pt_x_conj));
        let f = eval!(context, ((d) * (r.pt.x)) - ((e) * (r.pt.y)));

        def.insert(key, (d, e, f));
    }

    let query_point_x = Simd::unpack(context, &queries.points.x);
    let query_point_y = Simd::unpack(context, &queries.points.y);

    // Denominator inverse for each (pt.x, pt.y, query).
    let mut denominator_inverse = IndexMap::<(usize, usize, usize), Var>::new();

    for ((pt_x, pt_y), (d, e, f)) in def.iter() {
        for (query_idx, (q_x, q_y)) in zip_eq(&query_point_x, &query_point_y).enumerate() {
            // Compute `d * q_x - e * q_y - f`.
            let denominator = eval!(context, ((*d) * (*q_x)) - (((*e) * (*q_y)) + (*f)));
            let denominator_inv = div(context, context.one(), denominator);

            denominator_inverse.insert((*pt_x, *pt_y, query_idx), denominator_inv);
        }
    }

    // The coefficients `a, b, c` for each response.
    let abc = oods_responses
        .iter()
        .map(|r| {
            let pt_y_conj = conj(context, r.pt.y);
            let r_value_conj = conj(context, r.value);
            let a = eval!(context, (r_value_conj) - (r.value));
            let c = eval!(context, (pt_y_conj) - (r.pt.y));
            let b = eval!(context, ((r.value) * (c)) - ((a) * (r.pt.y)));

            (a, b, c)
        })
        .collect_vec();

    let minus_two_u = context.constant(-qm31_from_u32s(0, 0, 2, 0));

    let mut fri_queries = Vec::new();
    for (query_idx, (_q_x, q_y)) in zip_eq(&query_point_x, &query_point_y).enumerate() {
        let mut quotient_coef = context.one();
        let mut sum = context.zero();

        for ((a, b, c), r) in zip_eq(abc.iter(), oods_responses.iter()) {
            let query_value_at_column = *trace_queries.at(r.trace_idx, query_idx, r.column_idx);

            // Compute c * column[column_idx](q_x, q_y) - a * q_y - b.
            let numerator =
                eval!(context, ((*c) * (query_value_at_column)) - ((*b) + ((*a) * (*q_y))));

            // Fetch the inverse of the denominator from `denominator_inverse`.
            let denominator_inv =
                denominator_inverse.get(&(r.pt.x.idx, r.pt.y.idx, query_idx)).unwrap();

            // Compute the quotient: numerator / denominator.
            let quotient = eval!(context, (numerator) * (*denominator_inv));

            // Add `quotient_coef * quotient` to `sum`.
            sum = eval!(context, (sum) + ((quotient_coef) * (quotient)));

            // Compute the next quotient coefficient (alpha^i).
            quotient_coef = eval!(context, (quotient_coef) * (alpha));
        }

        // Multiply by `-2u` for compatibility with the stwo prover.
        sum = eval!(context, (sum) * (minus_two_u));

        fri_queries.push(sum);
    }

    fri_queries
}
