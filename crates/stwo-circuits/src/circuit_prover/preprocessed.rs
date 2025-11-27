use stwo::core::ColumnVec;
use stwo::core::fields::m31::BaseField;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::backend::simd::m31::PackedBaseField;
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::CircleEvaluation;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

pub trait PreProcessedColumn {
    fn id(&self) -> PreProcessedColumnId;
    fn log_size(&self) -> u32;
    fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>;
    fn packed_at(&self, vec_row: usize) -> PackedBaseField;
}

/// A collection of preprocessed columns, whose values are publicly acknowledged, and independent of
/// the proof.
pub struct PreProcessedTrace {
    columns: Vec<Box<dyn PreProcessedColumn>>,
}

impl PreProcessedTrace {
    pub fn ids(&self) -> Vec<PreProcessedColumnId> {
        self.columns.iter().map(|c| c.id()).collect()
    }

    pub fn log_sizes(&self) -> Vec<u32> {
        self.columns.iter().map(|c| c.log_size()).collect()
    }

    pub fn gen_trace(
        &self,
    ) -> ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>> {
        self.columns.iter().map(|c| c.gen_column_simd()).collect()
    }

    pub fn fibonacci_preprocessed() -> Self {
        let columns = vec![];
        // TODO(Gali): Implement.
        Self { columns }
    }
}
