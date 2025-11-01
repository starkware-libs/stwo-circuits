use crate::circuits::blake::HashValue;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::{IValue, NoValue};
use crate::circuits::ops::Guess;

/// Represents the structure of a proof.
pub struct ProofConfig {
    pub n_proof_of_work_bits: usize,

    // AIR structure.
    pub n_preprocessed_columns: usize,
    pub n_trace_columns: usize,
    pub n_interaction_columns: usize,
}

pub struct Proof<T> {
    // Merkle roots.
    pub trace_root0: HashValue<T>,
    pub trace_root1: HashValue<T>,
    pub interaction_root: HashValue<T>,
    pub composition_polynomial_root: HashValue<T>,
    // TODO(lior): Add missing fields.
}

pub fn empty_proof(_config: &ProofConfig) -> Proof<NoValue> {
    Proof {
        trace_root0: HashValue(NoValue, NoValue),
        trace_root1: HashValue(NoValue, NoValue),
        interaction_root: HashValue(NoValue, NoValue),
        composition_polynomial_root: HashValue(NoValue, NoValue),
    }
}

impl<Value: IValue> Guess<Value> for Proof<Value> {
    type Target = Proof<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Proof {
            trace_root0: self.trace_root0.guess(context),
            trace_root1: self.trace_root1.guess(context),
            interaction_root: self.interaction_root.guess(context),
            composition_polynomial_root: self.composition_polynomial_root.guess(context),
        }
    }
}
