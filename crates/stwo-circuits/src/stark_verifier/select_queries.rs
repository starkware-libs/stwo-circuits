use stwo::core::circle::CirclePoint;

use crate::circuits::context::Context;
use crate::circuits::ivalue::IValue;
use crate::circuits::simd::Simd;
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::circle::{add_points_simd, generator_point_simd};
use crate::stark_verifier::extract_bits::extract_bits;

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

/// Information regarding the chosen queries.
pub struct Queries {
    /// A vector with the bits of all the queries. `bits[i]` is a [Simd] with the `i`-th bit of each
    /// of the queries (`i = 0` is the LSB).
    pub bits: Vec<Simd>,
    /// The circle points from the evaluation domain that correspond to the chosen queries.
    pub points: CirclePoint<Simd>,
}

/// Takes the data obtained from [get_select_queries_input], and returns [Queries].
pub fn select_queries(
    context: &mut Context<impl IValue>,
    input: &Simd,
    log_domain_size: usize,
) -> Queries {
    let bits = extract_bits(context, input)[0..log_domain_size].to_vec();

    // Construct the circle point for the query.
    let mut point = generator_point_simd(context, log_domain_size + 1, input.len());

    for (i, bit) in bits.iter().skip(1).enumerate() {
        let cur_gen_pt = generator_point_simd(context, i + 1, input.len());
        // Select between `point` and `point + cur_gen_pt`.
        let point_if_bit = add_points_simd(context, &point, &cur_gen_pt);
        point = CirclePoint {
            x: Simd::select(context, bit, &point.x, &point_if_bit.x),
            y: Simd::select(context, bit, &point.y, &point_if_bit.y),
        };
    }

    // Handle the first bit, which may negate the sign of `y`.
    let zero = Simd::zero(context, input.len());
    let neg_y = Simd::sub(context, &zero, &point.y);
    point.y = Simd::select(context, &bits[0], &point.y, &neg_y);

    Queries { bits, points: point }
}
