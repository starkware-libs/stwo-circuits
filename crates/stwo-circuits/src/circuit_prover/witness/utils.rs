use stwo::core::channel::MerkleChannel;
use stwo::core::fields::m31::M31;
use stwo::core::pcs::TreeSubspan;
use stwo::prover::backend::Backend;
use stwo::prover::backend::BackendForChannel;
use stwo::prover::backend::simd::conversion::Pack;
use stwo::prover::backend::simd::m31::N_LANES;
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::CircleEvaluation;

pub fn pack_values<T: Pack + Copy>(values: &[T]) -> Vec<T::SimdType> {
    values.chunks_exact(N_LANES).map(|c| T::pack(c.try_into().unwrap())).collect()
}

/// Extenders of a commitment-tree with evaluations.
pub trait TreeBuilder<B: Backend> {
    fn extend_evals(
        &mut self,
        columns: impl IntoIterator<Item = CircleEvaluation<B, M31, BitReversedOrder>>,
    ) -> TreeSubspan;
}

impl<B: BackendForChannel<MC>, MC: MerkleChannel> TreeBuilder<B>
    for stwo::prover::TreeBuilder<'_, '_, B, MC>
{
    fn extend_evals(
        &mut self,
        columns: impl IntoIterator<Item = CircleEvaluation<B, M31, BitReversedOrder>>,
    ) -> TreeSubspan {
        self.extend_evals(columns)
    }
}
