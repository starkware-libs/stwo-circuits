use stwo::prover::backend::simd::conversion::Pack;
use stwo::prover::backend::simd::m31::N_LANES;

pub fn pack_values<T: Pack + Copy>(values: &[T]) -> Vec<T::SimdType> {
    values.chunks_exact(N_LANES).map(|c| T::pack(c.try_into().unwrap())).collect()
}
