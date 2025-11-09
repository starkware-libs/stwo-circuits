use crate::circuits::blake::HashValue;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::Guess;

/// Represents the structure of a FRI proof.
pub struct FriConfig {
    pub log_trace_size: usize,
    pub log_evaluation_domain_size: usize,
    pub n_queries: usize,
    pub log_n_last_layer_coefs: usize,
}

/// Represents the information for the FRI commitment phase of the proof.
pub struct FriCommitProof<T> {
    pub roots: Vec<HashValue<T>>,
    pub last_layer_coefs: Vec<T>,
}

impl FriCommitProof<Var> {
    /// Validates that the size of the members of the struct are consistent with the config.
    pub fn validate_structure(&self, config: &FriConfig) {
        assert_eq!(self.roots.len(), config.log_trace_size);
        assert_eq!(self.last_layer_coefs.len(), 1 << config.log_n_last_layer_coefs);
    }
}

impl<Value: IValue> Guess<Value> for FriCommitProof<Value> {
    type Target = FriCommitProof<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Self::Target {
            roots: self.roots.guess(context),
            last_layer_coefs: self.last_layer_coefs.guess(context),
        }
    }
}
