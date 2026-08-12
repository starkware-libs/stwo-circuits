use itertools::Itertools;
use rstest::rstest;
use stwo::core::fields::m31::M31;

use crate::sort_queries::{QuerySorter, generate_sort_keys};
use crate::verify::LOG_SIZE_BITS;
use circuits::context::{TraceContext, Var};
use circuits::ops::Guess;
use circuits::wrappers::M31Wrapper;

/// Builds `Var`s holding the given (small, concrete) log-size values, mirroring how callers such
/// as `merkle::decommit_eval_domain_samples` supply `column_log_sizes`.
fn log_size_vars(context: &mut TraceContext, log_sizes: &[u32]) -> Vec<Var> {
    log_sizes.iter().map(|&log_size| context.constant(M31::from(log_size).into())).collect_vec()
}

/// `generate_sort_keys` yields `key_i = (log_size_i * column_idx_bound + i) * u`. Feeding it
/// non-decreasing log sizes (as documented on `generate_sort_keys`) produces strictly increasing
/// keys, which `verify_sorted_keys` must accept. Reordering the log sizes so they are no longer
/// non-decreasing makes some derived key decrease, which `verify_sorted_keys` must reject via the
/// `extract_bits` range check on the (now out-of-range) diff.
#[rstest]
#[case::ascending_log_sizes(&[0, 2, 2, 5], true)]
#[case::descending_log_sizes(&[5, 2, 2, 0], false)]
fn test_verify_sorted_keys(#[case] log_sizes: &[u32], #[case] expected_valid: bool) {
    let mut context = TraceContext::default();
    let column_log_sizes = log_size_vars(&mut context, log_sizes);
    let column_idx_bits = column_log_sizes.len().next_power_of_two().ilog2();
    let key_bits = LOG_SIZE_BITS + column_idx_bits;
    assert!(key_bits <= 30);

    let sort_keys = generate_sort_keys(&mut context, &column_log_sizes, column_idx_bits);

    // `verify_sorted_keys` only reads `self.key_bits`; `sort_keys`/`sorted_keys` are irrelevant to
    // this direct call, so a bare struct literal is enough (test module is a child of
    // `sort_queries`, so private fields are accessible).
    let sorter = QuerySorter { key_bits, sort_keys: vec![], sorted_keys: vec![] };
    sorter.verify_sorted_keys(&mut context, &sort_keys);

    assert_eq!(context.is_circuit_valid(), expected_valid);
}

/// An empty key slice (the `sorted_keys.first()` early return) must be a complete no-op: no gates
/// are added and the circuit stays trivially valid.
#[test]
fn test_verify_sorted_keys_empty_is_noop() {
    let mut context = TraceContext::default();
    let sorter = QuerySorter { key_bits: 7, sort_keys: vec![], sorted_keys: vec![] };

    sorter.verify_sorted_keys(&mut context, &[]);

    context.validate_circuit();
}

/// End-to-end test through the public API: `QuerySorter::new` + `sort` must return the input
/// query values reordered into a permutation (same multiset), with a circuit that validates,
/// even when the columns are not given in log-size order.
#[test]
fn test_sort_permutes_and_validates() {
    let mut context = TraceContext::default();
    // Deliberately out of order, to exercise real reordering rather than a no-op permutation.
    let column_log_sizes = log_size_vars(&mut context, &[4, 2, 6, 2, 5]);
    let mut sorter = QuerySorter::new(&mut context, &column_log_sizes);

    let values = [111, 222, 333, 444, 555].map(M31::from);
    let query_values: Vec<M31Wrapper<Var>> =
        values.iter().map(|v| M31Wrapper::from(*v).guess(&mut context)).collect_vec();

    let sorted = sorter.sort(&mut context, query_values);

    let mut sorted_values = sorted.iter().map(|v| context.get(*v.get())).collect_vec();
    let mut expected_values = values.map(Into::into).to_vec();
    sorted_values.sort();
    expected_values.sort();
    assert_eq!(sorted_values, expected_values);

    context.validate_circuit();
}

/// Sorting the same sorter twice must take the cheaper "matches previous sort" branch and agree
/// on the resulting order, since both calls share the same per-column sort keys.
#[test]
fn test_sort_twice_is_consistent() {
    let mut context = TraceContext::default();
    let column_log_sizes = log_size_vars(&mut context, &[2, 4, 4]);
    let mut sorter = QuerySorter::new(&mut context, &column_log_sizes);

    let values = [7, 1, 9].map(M31::from);
    let make_query_values = |context: &mut TraceContext| -> Vec<M31Wrapper<Var>> {
        values.iter().map(|v| M31Wrapper::from(*v).guess(context)).collect_vec()
    };

    let first_query_values = make_query_values(&mut context);
    let first = sorter.sort(&mut context, first_query_values);
    let second_query_values = make_query_values(&mut context);
    let second = sorter.sort(&mut context, second_query_values);

    let first_values = first.iter().map(|v| context.get(*v.get())).collect_vec();
    let second_values = second.iter().map(|v| context.get(*v.get())).collect_vec();
    assert_eq!(first_values, second_values);

    context.validate_circuit();
}
