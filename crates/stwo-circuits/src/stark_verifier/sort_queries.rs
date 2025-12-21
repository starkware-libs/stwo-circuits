use std::iter::zip;

use itertools::{Itertools, zip_eq};
use stwo::core::fields::FieldExpOps;

use crate::{
    circuits::{
        context::{Context, Var},
        ivalue::{IValue, qm31_from_u32s},
        ops::{eq, pointwise_mul},
        simd::Simd,
        wrappers::M31Wrapper,
    },
    eval,
    stark_verifier::verify::LOG_SIZE_BITS,
};

const COLUMN_IDX_BITS: usize = 16;

const COLUMN_IDX_BOUND: usize = 1 << COLUMN_IDX_BITS;

use crate::stark_verifier::extract_bits::extract_bits;

/// Generates the column indices for the columns.
/// The column indices are the values 1..n_columns.
pub fn generate_column_indices<Value: IValue>(
    context: &mut Context<Value>,
    n_columns: usize,
) -> Vec<Var> {
    let mut column_idx = context.zero();
    let mut column_indices = vec![column_idx];
    for _ in 1..n_columns {
        column_idx = eval!(context, (column_idx) + (context.one()));
        column_indices.push(column_idx);
    }
    column_indices
}

/// Generates the keys for sorting the query values by the column indices and log sizes.
/// the key for the i'th column is (log_size * COLUMN_IDX_BOUND + column_idx) * u.
/// Those keys guarantee a stable sort of the query values by the column indices and log sizes.
///
/// column_indices hold the values 1..N, where N >= column_log_sizes.len().
/// column_log_sizes hold the log sizes of the columns.
pub fn generate_sort_keys<Value: IValue>(
    context: &mut Context<Value>,
    column_indices: &[Var],
    column_log_sizes: &[Var],
) -> Vec<Var> {
    assert!(column_indices.len() >= column_log_sizes.len());
    let u = context.constant(qm31_from_u32s(0, 0, 0, 1));
    let shift = context.constant(COLUMN_IDX_BOUND.into());
    let mut sort_keys = Vec::with_capacity(column_log_sizes.len());
    for (log_size, column_idx) in zip(column_log_sizes, column_indices) {
        let shifted_log_size = eval!(context, (*log_size) * (shift));
        let tag = eval!(context, (shifted_log_size) + (*column_idx));
        let sort_key = eval!(context, (u) * (tag));
        sort_keys.push(sort_key);
    }

    sort_keys
}

/// Verifies that the sorted keys are indeed sorted.
fn verify_sorted_keys<Value: IValue>(context: &mut Context<Value>, sorted_keys: &[Var]) {
    let u_inverse = context.constant(qm31_from_u32s(0, 0, 0, 1).inverse());

    // extract key from u * key.
    let Some(mut prev) = sorted_keys.first() else {
        // if there are no sorted keys, return.
        return;
    };
    let diffs = sorted_keys
        .iter()
        .map(|curr| {
            let u_diff = eval!(context, (*curr) - (*prev));
            prev = curr;

            // extract diff from u * diff.
            let diff = eval!(context, (u_diff) * (u_inverse));
            M31Wrapper::new_unsafe(diff)
        })
        .collect_vec();

    let packed = Simd::pack(context, &diffs);
    // call extract_bits to range check the values.
    const RANGE_CHECK_BITS: usize = LOG_SIZE_BITS + COLUMN_IDX_BITS;

    // The verificaion needs to fail if one of the diffs is negative.
    //
    // 0 - (RANGE_CHECK_BITS - 1) < 0
    // => 2**31 - 1 - 2**(RANGE_CHECK_BITS >= 2**RANGE_CHECK_BITS
    // => 2**31 - 2**(RANGE_CHECK_BITS +1) >= 1
    // => RANGE_CHECK_BITS < 30
    const _: () = assert!(RANGE_CHECK_BITS < 30);
    // Add static assert.
    let _bits = extract_bits::<RANGE_CHECK_BITS>(context, &packed);
}

/// Sorts the query values by the sort keys and returns the sorted values and keys.
/// Assumes that the sorting of the keys is validated by the caller.
pub fn sort_query_values<Value: IValue>(
    context: &mut Context<Value>,
    query_values: &[M31Wrapper<Var>],
    sort_keys: &[Var],
    opt_sorted_keys: &mut Option<Vec<Var>>,
) -> Vec<M31Wrapper<Var>> {
    let tagged_values = zip_eq(query_values, sort_keys)
        .map(|(value, sort_key)| eval!(context, (*value.get()) + (*sort_key)))
        .collect_vec();

    let sorted =
        IValue::sort_by_u_cord(&tagged_values.iter().map(|var| context.get(*var)).collect_vec())
            .iter()
            .map(|value| context.new_var(*value))
            .collect_vec();

    context.permute(&tagged_values, &sorted);

    let mut sorted_keys = Vec::with_capacity(sorted.len());
    let sorted_values = sorted
        .iter()
        .map(|var| {
            let value = pointwise_mul(context, *var, context.one());

            let u_key = eval!(context, (*var) - (value));
            sorted_keys.push(u_key);

            M31Wrapper::new_unsafe(value)
        })
        .collect_vec();

    if let Some(prev_sorted_keys) = opt_sorted_keys {
        // Check that the sorting is the same as in the previous iteration.
        for (value, prev_value) in zip_eq(sorted_keys, prev_sorted_keys) {
            eq(context, value, *prev_value);
        }
    } else {
        verify_sorted_keys(context, &sorted_keys);
        // Store the sorted keys for the next iteration and for future verification.
        *opt_sorted_keys = Some(sorted_keys);
    };

    sorted_values
}
