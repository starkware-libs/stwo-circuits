use itertools::zip_eq;
use stwo::core::circle::CirclePoint;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::circuits::EXTENSION_DEGREE;
use crate::circuits::circuit::Var;
use crate::circuits::context::Context;
use crate::circuits::ivalue::{IValue, NoValue};
use crate::circuits::ops::{Guess, from_partial_evals, pointwise_mul};
use crate::eval;
use crate::stark_verifier::circle::double_x;

const COMPOSITION_SPLIT: usize = 2;
pub const N_COMPOSITION_COLUMNS: usize = COMPOSITION_SPLIT * EXTENSION_DEGREE;

#[cfg(test)]
#[path = "oods_test.rs"]
pub mod test;

/// Represents a value that should be in the base field `M31`.
///
/// Using the [Guess] trait on [M31Wrapper] and gates that guarantee that the guessed value is
/// indeed in the base field `M31`.
#[derive(Clone, Debug)]
pub struct M31Wrapper<T>(T);

impl<T> M31Wrapper<T> {
    fn get(&self) -> &T {
        &self.0
    }
}

impl From<M31> for M31Wrapper<QM31> {
    fn from(value: M31) -> Self {
        M31Wrapper(value.into())
    }
}

impl From<NoValue> for M31Wrapper<NoValue> {
    fn from(_: NoValue) -> Self {
        M31Wrapper(NoValue)
    }
}

impl<Value: IValue> Guess<Value> for M31Wrapper<Value> {
    type Target = M31Wrapper<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        let value = self.0.guess(context);
        // Mask the value with `1 + 0 * i + 0 * u + 0 * iu` to ensure (in the circuit) it is
        // in the base field `M31`.
        let masked_value = pointwise_mul(context, value, context.one());
        M31Wrapper(masked_value)
    }
}

/// Represents the sampled values in the evaluation domain.
#[derive(Debug)]
pub struct EvalDomainSamples<T> {
    /// `data[trace_idx][query_idx][column_idx]` is the `M31` value of the column `column_idx` in
    /// trace `trace_idx` at the `query_idx` query.
    data: Vec<Vec<Vec<M31Wrapper<T>>>>,
}

impl<T> EvalDomainSamples<T> {
    /// Returns the sampled value for the given trace, query, and column.
    pub fn at(&self, trace_idx: usize, query_idx: usize, column_idx: usize) -> &T {
        self.data[trace_idx][query_idx][column_idx].get()
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
            .map(|n_columns| vec![vec![M31Wrapper(NoValue); *n_columns]; n_queries])
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
