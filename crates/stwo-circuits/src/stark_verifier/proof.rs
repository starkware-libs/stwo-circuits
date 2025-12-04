use crate::circuits::blake::HashValue;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::{IValue, NoValue};
use crate::circuits::ops::Guess;
use crate::stark_verifier::fri_proof::{FriConfig, FriProof, empty_fri_proof};
use crate::stark_verifier::merkle::{AuthPath, AuthPaths};
use crate::stark_verifier::oods::{
    EvalDomainSamples, N_COMPOSITION_COLUMNS, empty_eval_domain_samples,
};

pub const N_TRACES: usize = 4;

/// Represents the structure of a proof.
pub struct ProofConfig {
    // TODO(lior): Add a check on the total security bits of the protocol given parameters
    //   such as `n_proof_of_work_bits`, `fri.n_queries`, etc.
    pub n_proof_of_work_bits: usize,

    // AIR structure.
    pub n_preprocessed_columns: usize,
    pub n_trace_columns: usize,
    pub n_interaction_columns: usize,
    // Per column in the interaction trace, an indicator of whether it is a cumulative sum column.
    // This is used to determine whether to include a sample point at the previous point in the
    // OODS response.
    pub cumulative_sum_columns: Vec<bool>,

    pub fri: FriConfig,
}
impl ProofConfig {
    /// Returns the log2 of the size of the trace.
    pub fn log_trace_size(&self) -> usize {
        self.fri.log_trace_size
    }

    /// Returns the log2 of the size of the evaluation domain.
    pub fn log_evaluation_domain_size(&self) -> usize {
        self.fri.log_evaluation_domain_size()
    }

    /// Returns the number of queries.
    pub fn n_queries(&self) -> usize {
        self.fri.n_queries
    }

    /// Returns the number of columns for each of the traces.
    pub fn n_columns_per_trace(&self) -> [usize; N_TRACES] {
        [
            self.n_preprocessed_columns,
            self.n_trace_columns,
            self.n_interaction_columns,
            N_COMPOSITION_COLUMNS,
        ]
    }
}

/// The values of the interaction trace at the OODS point and the previous point.
pub struct InteractionAtOods<T> {
    /// For each column, the value at the OODS point and optionally the value at the previous point
    /// (`oods_point - trace_generator`).
    // TODO(lior): Make the second element optional.
    pub value: Vec<(T, T)>,
}

impl<T> InteractionAtOods<T> {
    /// Returns the number of columns.
    pub fn n_columns(&self) -> usize {
        self.value.len()
    }
}

impl<T: Copy> InteractionAtOods<T> {
    /// Returns the value at the OODS point.
    pub fn at_oods(&self, idx: usize) -> T {
        self.value[idx].0
    }

    /// Returns the value at the previous point (`oods_point - trace_generator`).
    pub fn at_prev(&self, idx: usize) -> T {
        self.value[idx].1
    }

    /// Returns a flattened list of the values at the OODS point and the previous point.
    pub fn flattened(&self) -> Vec<T> {
        self.value.iter().flat_map(|(a, b)| [*b, *a]).collect()
    }
}

impl<Value: IValue> Guess<Value> for InteractionAtOods<Value> {
    type Target = InteractionAtOods<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        InteractionAtOods { value: self.value.guess(context) }
    }
}

pub struct Proof<T> {
    // Merkle roots.
    pub preprocessed_root: HashValue<T>,
    pub trace_root: HashValue<T>,
    pub interaction_root: HashValue<T>,
    pub composition_polynomial_root: HashValue<T>,

    // Evaluations at the OODS point and the previous point.
    pub preprocessed_columns_at_oods: Vec<T>,
    pub trace_at_oods: Vec<T>,
    pub interaction_at_oods: InteractionAtOods<T>,
    pub composition_eval_at_oods: [T; N_COMPOSITION_COLUMNS],

    // Evaluations at the evaluation domain.
    pub eval_domain_samples: EvalDomainSamples<T>,
    pub eval_domain_auth_paths: AuthPaths<T>,

    pub proof_of_work_nonce: T,
    pub fri: FriProof<T>,
    // TODO(lior): Add missing fields.
}
impl<T> Proof<T> {
    /// Validates that the size of the members of the struct are consistent with the config.
    pub fn validate_structure(&self, config: &ProofConfig) {
        // Validate preprocessed_columns_at_oods.
        assert_eq!(self.preprocessed_columns_at_oods.len(), config.n_preprocessed_columns);

        // Validate trace_at_oods.
        assert_eq!(self.trace_at_oods.len(), config.n_trace_columns);

        // Validate interaction_at_oods.
        assert_eq!(self.interaction_at_oods.n_columns(), config.n_interaction_columns);

        // Validate eval_domain_samples.
        self.eval_domain_samples
            .validate_structure(&config.n_columns_per_trace(), config.n_queries());

        // Validate FRI.
        self.fri.validate_structure(&config.fri);
    }

    /// Returns the list of all 4 roots.
    pub fn merkle_roots(&self) -> [HashValue<T>; N_TRACES]
    where
        T: Copy,
    {
        [
            self.preprocessed_root,
            self.trace_root,
            self.interaction_root,
            self.composition_polynomial_root,
        ]
    }
}

pub fn empty_proof(config: &ProofConfig) -> Proof<NoValue> {
    let auth_path =
        AuthPath(vec![HashValue(NoValue, NoValue); config.log_evaluation_domain_size()]);

    Proof {
        preprocessed_root: HashValue(NoValue, NoValue),
        trace_root: HashValue(NoValue, NoValue),
        interaction_root: HashValue(NoValue, NoValue),
        composition_polynomial_root: HashValue(NoValue, NoValue),
        preprocessed_columns_at_oods: vec![NoValue; config.n_preprocessed_columns],
        trace_at_oods: vec![NoValue; config.n_trace_columns],
        interaction_at_oods: InteractionAtOods {
            value: vec![(NoValue, NoValue); config.n_interaction_columns],
        },
        composition_eval_at_oods: [NoValue; N_COMPOSITION_COLUMNS],
        eval_domain_samples: empty_eval_domain_samples(
            &config.n_columns_per_trace(),
            config.n_queries(),
        ),
        eval_domain_auth_paths: AuthPaths {
            data: vec![vec![auth_path; config.n_queries()]; N_TRACES],
        },
        proof_of_work_nonce: NoValue,
        fri: empty_fri_proof(&config.fri),
    }
}

impl<Value: IValue> Guess<Value> for Proof<Value> {
    type Target = Proof<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Proof {
            preprocessed_root: self.preprocessed_root.guess(context),
            trace_root: self.trace_root.guess(context),
            interaction_root: self.interaction_root.guess(context),
            composition_polynomial_root: self.composition_polynomial_root.guess(context),
            preprocessed_columns_at_oods: self.preprocessed_columns_at_oods.guess(context),
            trace_at_oods: self.trace_at_oods.guess(context),
            interaction_at_oods: self.interaction_at_oods.guess(context),
            composition_eval_at_oods: self.composition_eval_at_oods.guess(context),
            eval_domain_samples: self.eval_domain_samples.guess(context),
            eval_domain_auth_paths: self.eval_domain_auth_paths.guess(context),
            proof_of_work_nonce: self.proof_of_work_nonce.guess(context),
            fri: self.fri.guess(context),
        }
    }
}
