use crate::circuits::context::Context;
use crate::circuits::ivalue::IValue;
use crate::circuits::simd::Simd;
use crate::stark_verifier::channel::Channel;

#[cfg(test)]
#[path = "select_queries_test.rs"]
pub mod test;

/// Fetches the input for query selection from the channel, by drawing `ceil(n_queries / 8)`
/// pairs of `QM31` values.
pub fn get_select_queries_input(
    context: &mut Context<impl IValue>,
    channel: &mut Channel,
    n_queries: usize,
) -> Simd {
    let mut v = Vec::new();
    for _ in 0..n_queries.div_ceil(8) {
        v.extend_from_slice(&channel.draw_two_qm31s(context));
    }
    let n_qm31s = n_queries.div_ceil(4);

    Simd::from_packed(v[0..n_qm31s].to_vec(), n_queries)
}
