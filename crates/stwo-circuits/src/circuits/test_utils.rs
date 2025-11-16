use stwo::core::fields::qm31::QM31;

use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::simd::Simd;

/// Constructs a [Simd] from a vector of `u32` values.
pub fn simd_from_u32s(context: &mut TraceContext, mut values: Vec<u32>) -> Simd {
    let original_len = values.len();
    let n_qm31 = original_len.div_ceil(4);
    // Pad with zeros if necessary.
    values.resize(n_qm31 * 4, 0);

    let res = (0..n_qm31)
        .map(|i| {
            let value = qm31_from_u32s(
                values[i * 4],
                values[i * 4 + 1],
                values[i * 4 + 2],
                values[i * 4 + 3],
            );
            context.new_var(value)
        })
        .collect();

    Simd::from_packed(res, original_len)
}

/// Given a [Simd], returns the values of the packed `QM31`s.
pub fn packed_values(context: &TraceContext, simd: &Simd) -> Vec<QM31> {
    simd.get_packed().iter().map(|v| context.get(*v)).collect()
}
