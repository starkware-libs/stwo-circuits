pub use crate::cairo_air::component_utils::*;
pub use crate::circuit_air::relations;
pub use crate::circuits::context::{Context, Var};
pub use crate::circuits::ivalue::IValue;
pub use crate::circuits::ops::eq;
pub use crate::eval;
pub use crate::stark_verifier::constraint_eval::{
    CircuitEval, ComponentData, CompositionConstraintAccumulator, RelationUse,
};
pub use itertools::chain;
pub use num_traits::One;
pub use num_traits::Zero;
pub use serde::{Deserialize, Serialize};
pub use stwo::core::air::Component;
pub use stwo::core::channel::Channel;
pub use stwo::core::fields::m31::M31;
pub use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;
pub use stwo::core::fields::qm31::SecureField;
pub use stwo::core::pcs::TreeVec;
pub use stwo::prover::ComponentProver;
pub use stwo::prover::backend::simd::SimdBackend;
pub use stwo_constraint_framework::EvalAtRow;
pub use stwo_constraint_framework::FrameworkComponent;
pub use stwo_constraint_framework::FrameworkEval;
pub use stwo_constraint_framework::RelationEntry;
pub use stwo_constraint_framework::TraceLocationAllocator;
pub use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

#[derive(Clone)]
pub struct RelationUse {
    pub relation_id: &'static str,
    pub uses: u64,
}

/// A column with the numbers [0..(2^log_size)-1].
#[derive(Debug, Clone)]
pub struct Seq {
    pub log_size: u32,
}
impl Seq {
    pub const fn new(log_size: u32) -> Self {
        Self { log_size }
    }
}
impl PreProcessedColumn for Seq {
    fn log_size(&self) -> u32 {
        self.log_size
    }
    fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId { id: format!("seq_{}", self.log_size) }
    }
}

pub trait PreProcessedColumn: Send + Sync {
    fn log_size(&self) -> u32;
    fn id(&self) -> PreProcessedColumnId;
}
