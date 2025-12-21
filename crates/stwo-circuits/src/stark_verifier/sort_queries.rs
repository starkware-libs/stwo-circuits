use std::iter::zip;

use itertools::{Itertools, zip_eq};
use stwo::core::fields::FieldExpOps;

use crate::{
    circuits::{
        context::{Context, Var},
        ivalue::{IValue, qm31_from_u32s},
        ops::{eq, permute, pointwise_mul},
        simd::Simd,
        wrappers::M31Wrapper,
    },
    eval,
    stark_verifier::verify::LOG_SIZE_BITS,
};

const COLUMN_IDX_BITS: usize = 12;
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
fn generate_sort_keys<Value: IValue>(
    context: &mut Context<Value>,
    column_indices: &[Var],
    column_log_sizes: &[Var],
) -> Vec<Var> {
    assert!(column_indices.len() >= column_log_sizes.len());
    let u = context.constant(qm31_from_u32s(0, 0, 1, 0));
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
    let u_inverse = context.constant(qm31_from_u32s(0, 0, 1, 0).inverse());

    let Some(mut prev) = sorted_keys.first() else {
        // if there are no sorted keys, return.
        return;
    };

    let diffs = sorted_keys
        .iter()
        .skip(1)
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
    // 0 - (2**RANGE_CHECK_BITS - 1) < 0
    // => 2**31 - 1 - (2**RANGE_CHECK_BITS - 1) > 2**RANGE_CHECK_BITS
    // => 2**31 - 2**RANGE_CHECK_BITS > 0
    // => RANGE_CHECK_BITS < 30
    const _: () = assert!(RANGE_CHECK_BITS < 30);
    // Add static assert.
    let _bits = extract_bits::<RANGE_CHECK_BITS>(context, &packed);
}

/// Helper struct for sorting query values.
pub struct QuerySorter {
    // The key for the i'th column is (log_size * COLUMN_IDX_BOUND + column_idx) * u.
    // An empty vector if sorting is skipped (or if there are no columns to sort).
    sort_keys: Vec<Var>,

    // The sort keys after sorting or an empty vector if this is the first sort.
    // This is used to verify that the sorting is the same as in the previous sort as it is cheaper
    // using `verify_sorted_keys`.
    sorted_keys: Vec<Var>,
}
impl QuerySorter {
    pub fn new(
        context: &mut Context<impl IValue>,
        column_indices: &[Var],
        column_log_sizes: &[Var],
    ) -> Self {
        let sort_keys = generate_sort_keys(context, column_indices, column_log_sizes);
        Self { sort_keys, sorted_keys: vec![] }
    }

    /// Returns a sorter that skips sorting.
    pub fn skip_sorting() -> Self {
        Self { sort_keys: vec![], sorted_keys: vec![] }
    }

    /// Sorts the query values by the sort keys and returns the sorted values and keys.
    /// Assumes that the sorting of the keys is validated by the caller.
    ///
    /// The sort keys are constructed u*key, where key is an m31 element.
    pub fn sort<Value: IValue>(
        &mut self,
        context: &mut Context<Value>,
        query_values: Vec<M31Wrapper<Var>>,
    ) -> Vec<M31Wrapper<Var>> {
        if self.sort_keys.is_empty() {
            return query_values;
        }

        let tagged_values = zip_eq(query_values, &self.sort_keys)
            .map(|(value, sort_key)| eval!(context, (*value.get()) + (*sort_key)))
            .collect_vec();

        let sorted = permute(context, &tagged_values, IValue::sort_by_u_coordinate);

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

        if self.sorted_keys.is_empty() {
            // If this is the first sort, verify the sorted keys and store them for future
            // verification. Note that if self.sort_keys is empty, sorted_keys is also
            // empty, verify_sorted_keys will do nothing and self.sorted_keys will remain empty.
            verify_sorted_keys(context, &sorted_keys);
            self.sorted_keys = sorted_keys;
        } else {
            // Check that the sorting is the same as in the previous sort.
            for (key, prev_key) in zip_eq(sorted_keys, &self.sorted_keys) {
                eq(context, key, *prev_key);
            }
        };

        sorted_values
    }
}
