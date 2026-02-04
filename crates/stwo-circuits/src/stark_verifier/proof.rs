use crate::circuits::blake::HashValue;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::{IValue, NoValue};
use crate::circuits::ops::Guess;
use crate::stark_verifier::fri_proof::{FriConfig, FriProof, empty_fri_proof};
use crate::stark_verifier::merkle::{AuthPath, AuthPaths};
use crate::stark_verifier::oods::{
    EvalDomainSamples, N_COMPOSITION_COLUMNS, empty_eval_domain_samples,
};
use crate::stark_verifier::statement::Statement;
use itertools::{Itertools, zip_eq};
use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;
use stwo::core::pcs::PcsConfig;

pub const N_TRACES: usize = 4;

/// Represents the structure of a proof.
pub struct ProofConfig {
    // TODO(lior): Add a check on the total security bits of the protocol given parameters
    //   such as `n_proof_of_work_bits`, `fri.n_queries`, etc.
    pub n_proof_of_work_bits: u32,

    // AIR structure.
    pub n_preprocessed_columns: usize,
    pub n_trace_columns: usize,
    pub n_interaction_columns: usize,
    pub trace_columns_per_component: Vec<usize>,
    pub interaction_columns_per_component: Vec<usize>,

    // Per column in the interaction trace, an indicator of whether it is a cumulative sum column.
    // This is used to determine whether to include a sample point at the previous point in the
    // OODS response.
    pub cumulative_sum_columns: Vec<bool>,

    // Number of components in the AIR.
    pub n_components: usize,

    pub fri: FriConfig,
    pub interaction_pow_bits: u32,
}
impl ProofConfig {
    pub fn from_statement<Value: IValue>(
        statement: &impl Statement<Value>,
        log_trace_size: usize,
        pcs_config: &PcsConfig,
        interaction_pow_bits: u32,
    ) -> Self {
        let components = statement.get_components();
        let n_preprocessed_columns = statement.get_preprocessed_column_ids().len();
        let trace_columns_per_component =
            components.iter().map(|c| c.trace_columns()).collect_vec();
        let interaction_columns_per_component =
            components.iter().map(|c| c.interaction_columns()).collect_vec();

        Self::new(
            components.len(),
            trace_columns_per_component,
            interaction_columns_per_component,
            n_preprocessed_columns,
            log_trace_size,
            pcs_config,
            interaction_pow_bits,
        )
    }

    pub fn new(
        n_components: usize,
        trace_columns_per_component: Vec<usize>,
        interaction_columns_per_component: Vec<usize>,
        n_preprocessed_columns: usize,
        log_trace_size: usize,
        pcs_config: &PcsConfig,
        interaction_pow_bits: u32,
    ) -> Self {
        let n_interaction_columns = interaction_columns_per_component.iter().sum();
        let mut cumulative_sum_columns = Vec::with_capacity(n_interaction_columns);
        for n_interaction_columns_in_component in &interaction_columns_per_component {
            // The last SECURE_EXTENSION_DEGREE interaction columns of every component are
            // cumulative sum columns.
            assert!(
                *n_interaction_columns_in_component >= SECURE_EXTENSION_DEGREE,
                "Expected at least {SECURE_EXTENSION_DEGREE} interaction columns per component"
            );
            cumulative_sum_columns.extend(vec![
                false;
                *n_interaction_columns_in_component
                    - SECURE_EXTENSION_DEGREE
            ]);
            cumulative_sum_columns.extend(vec![true; SECURE_EXTENSION_DEGREE]);
        }

        let PcsConfig {
            pow_bits,
            fri_config:
                stwo::core::fri::FriConfig { log_blowup_factor, n_queries, log_last_layer_degree_bound },
        } = pcs_config;

        Self {
            n_proof_of_work_bits: *pow_bits,
            n_preprocessed_columns,
            n_trace_columns: trace_columns_per_component.iter().sum(),
            n_interaction_columns,
            trace_columns_per_component,
            interaction_columns_per_component,
            n_components,
            cumulative_sum_columns,
            fri: FriConfig {
                log_trace_size,
                log_blowup_factor: *log_blowup_factor as usize,
                n_queries: *n_queries,
                log_n_last_layer_coefs: *log_last_layer_degree_bound as usize,
            },
            interaction_pow_bits,
        }
    }

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

/// The values of an interaction column at the OODS point and the previous point.
#[derive(Clone)]
pub struct InteractionAtOods<T> {
    /// The value at the OODS point and optionally the value at the previous point
    /// (`oods_point - trace_generator`).
    pub at_oods: T,
    pub at_prev: Option<T>,
}

impl<Value: IValue> Guess<Value> for InteractionAtOods<Value> {
    type Target = InteractionAtOods<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        InteractionAtOods {
            at_oods: self.at_oods.guess(context),
            at_prev: self.at_prev.map(|at_prev| at_prev.guess(context)),
        }
    }
}

pub struct Claim<T> {
    // Every QM31 hold 4 bit.
    // TODO(ilya): Consider packing 29 bits into one M31.
    pub packed_enable_bits: Vec<T>,

    // The log sizes of the components in the AIR.
    // Every QM31 hold up to 4 component log sizes.
    pub packed_component_log_sizes: Vec<T>,

    // Claimed sum for each component in the AIR.
    pub claimed_sums: Vec<T>,
}
impl<Value: IValue> Guess<Value> for Claim<Value> {
    type Target = Claim<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Claim {
            packed_enable_bits: self.packed_enable_bits.guess(context),
            packed_component_log_sizes: self.packed_component_log_sizes.guess(context),
            claimed_sums: self.claimed_sums.guess(context),
        }
    }
}

pub struct Proof<T> {
    pub channel_salt: T,

    // Merkle roots.
    pub preprocessed_root: HashValue<T>,
    pub trace_root: HashValue<T>,
    pub interaction_root: HashValue<T>,
    pub composition_polynomial_root: HashValue<T>,

    // Claim.
    pub claim: Claim<T>,

    // Evaluations at the OODS point and the previous point.
    pub preprocessed_columns_at_oods: Vec<T>,
    pub trace_at_oods: Vec<T>,
    pub interaction_at_oods: Vec<InteractionAtOods<T>>,
    pub composition_eval_at_oods: [T; N_COMPOSITION_COLUMNS],

    // Evaluations at the evaluation domain.
    pub eval_domain_samples: EvalDomainSamples<T>,
    pub eval_domain_auth_paths: AuthPaths<T>,

    pub proof_of_work_nonce: T,
    pub interaction_pow_nonce: T,
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
        assert_eq!(self.interaction_at_oods.len(), config.n_interaction_columns);
        for (interaction_at_oods, is_cumulative_sum) in
            zip_eq(&self.interaction_at_oods, &config.cumulative_sum_columns)
        {
            assert_eq!(interaction_at_oods.at_prev.is_some(), *is_cumulative_sum);
        }

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
        interaction_at_oods: config
            .cumulative_sum_columns
            .iter()
            .map(|is_cumulative_sum| {
                if *is_cumulative_sum {
                    InteractionAtOods { at_oods: NoValue, at_prev: Some(NoValue) }
                } else {
                    InteractionAtOods { at_oods: NoValue, at_prev: None }
                }
            })
            .collect(),
        claim: Claim {
            packed_enable_bits: vec![NoValue; config.n_components.div_ceil(4)],
            packed_component_log_sizes: vec![NoValue; config.n_components.div_ceil(4)],
            claimed_sums: vec![NoValue; config.n_components],
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
        interaction_pow_nonce: NoValue,
        fri: empty_fri_proof(&config.fri),
        channel_salt: NoValue,
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
            claim: self.claim.guess(context),
            preprocessed_columns_at_oods: self.preprocessed_columns_at_oods.guess(context),
            trace_at_oods: self.trace_at_oods.guess(context),
            interaction_at_oods: self.interaction_at_oods.guess(context),
            composition_eval_at_oods: self.composition_eval_at_oods.guess(context),
            eval_domain_samples: self.eval_domain_samples.guess(context),
            eval_domain_auth_paths: self.eval_domain_auth_paths.guess(context),
            proof_of_work_nonce: self.proof_of_work_nonce.guess(context),
            interaction_pow_nonce: self.interaction_pow_nonce.guess(context),
            fri: self.fri.guess(context),
            channel_salt: self.channel_salt.guess(context),
        }
    }
}
