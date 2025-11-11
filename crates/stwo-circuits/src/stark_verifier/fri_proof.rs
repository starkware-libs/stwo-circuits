use crate::circuits::blake::HashValue;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::{IValue, NoValue};
use crate::circuits::ops::Guess;

/// Represents the structure of a FRI proof.
pub struct FriConfig {
    /// Log2 of the trace size.
    pub log_trace_size: usize,
    /// Log2 of the blowup factor.
    pub log_blowup_factor: usize,
    /// Number of queries.
    pub n_queries: usize,
    /// Log2 of the number of coefficients in the last layer of FRI.
    pub log_n_last_layer_coefs: usize,
}

impl FriConfig {
    pub fn log_evaluation_domain_size(&self) -> usize {
        self.log_trace_size + self.log_blowup_factor
    }
}

/// Represents the information for the FRI commitment phase of the proof.
pub struct FriCommitProof<T> {
    pub layer_commitments: Vec<HashValue<T>>,
    pub last_layer_coefs: Vec<T>,
}

impl FriCommitProof<Var> {
    /// Validates that the size of the members of the struct are consistent with the config.
    pub fn validate_structure(&self, config: &FriConfig) {
        // Starting from the last layer, each layer increases the log2 of the polynomial degree by
        // one. The final degree should be the same as the trace size.
        assert_eq!(
            config.log_n_last_layer_coefs + self.layer_commitments.len(),
            config.log_trace_size
        );
        assert_eq!(self.last_layer_coefs.len(), 1 << config.log_n_last_layer_coefs);
    }
}

impl<Value: IValue> Guess<Value> for FriCommitProof<Value> {
    type Target = FriCommitProof<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Self::Target {
            layer_commitments: self.layer_commitments.guess(context),
            last_layer_coefs: self.last_layer_coefs.guess(context),
        }
    }
}

pub fn empty_fri_proof(config: &FriConfig) -> FriCommitProof<NoValue> {
    let empty_hash = HashValue(NoValue, NoValue);
    FriCommitProof {
        layer_commitments: vec![empty_hash; config.log_trace_size],
        last_layer_coefs: vec![NoValue; 1 << config.log_n_last_layer_coefs],
    }
}
