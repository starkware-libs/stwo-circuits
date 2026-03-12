use crate::constraint_eval::CircuitEval;
use crate::fri_proof::{FriConfig, FriProof, empty_fri_proof};
use crate::merkle::{AuthPath, AuthPaths};
use crate::oods::{EvalDomainSamples, N_COMPOSITION_COLUMNS, empty_eval_domain_samples};
use crate::statement::Statement;
use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::ivalue::{IValue, NoValue};
use circuits::ops::Guess;
use itertools::{Itertools, zip_eq};

use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;
use stwo::core::pcs::PcsConfig;

pub const N_TRACES: usize = 4;

/// Proof info: trace size and proof size breakdown in M31 elements.
///
/// Fields that scale with `n_queries` store the per-query cost; use `total()` for the full size.
#[derive(Debug, Clone)]
pub struct ProofInfo {
    pub log_trace_size: usize,
    pub log_blowup_factor: usize,
    pub n_queries: usize,
    pub n_columns_per_trace: [usize; N_TRACES],
    // Fixed-size fields (not scaled by n_queries).
    pub fixed: usize,
    pub claim: usize,
    pub oods: usize,
    pub fri_commitments: usize,
    pub fri_last_layer: usize,
    // Per-query fields (total = per_query * n_queries).
    pub eval_samples_per_query: usize,
    pub eval_auth_per_query: usize,
    pub fri_auth_per_query: usize,
    pub fri_circle_siblings_per_query: usize,
    pub fri_line_coset_vals_per_query: usize,
}

impl ProofInfo {
    pub fn total(&self) -> usize {
        let q = self.n_queries;
        self.fixed
            + self.claim
            + self.oods
            + self.fri_commitments
            + self.fri_last_layer
            + (self.eval_samples_per_query
                + self.eval_auth_per_query
                + self.fri_auth_per_query
                + self.fri_circle_siblings_per_query
                + self.fri_line_coset_vals_per_query)
                * q
    }
}

impl std::fmt::Display for ProofInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let q = self.n_queries;
        let [p, t, i, c] = self.n_columns_per_trace;
        writeln!(f, "Proof info:")?;
        writeln!(f, "  log_trace_size:       {:>10}", self.log_trace_size)?;
        writeln!(f, "  log_blowup_factor:    {:>10}", self.log_blowup_factor)?;
        writeln!(f, "  n_queries:            {:>10}", self.n_queries)?;
        let total_columns = p + t + i + c;
        writeln!(f, "  n_columns_per_trace:  {total_columns:>10} = [{p}, {t}, {i}, {c}]")?;
        writeln!(f)?;
        writeln!(f, "Proof size breakdown (M31 elements)")?;
        writeln!(f, "  fixed:               {:>10}", self.fixed)?;
        writeln!(f, "  claim:               {:>10}", self.claim)?;
        writeln!(f, "  oods:                {:>10}", self.oods)?;
        writeln!(
            f,
            "  eval_samples:        {:>10} = {} * {q}",
            self.eval_samples_per_query * q,
            total_columns
        )?;
        writeln!(
            f,
            "  eval_auth_paths:     {:>10} = {} * {q}",
            self.eval_auth_per_query * q,
            self.eval_auth_per_query
        )?;
        writeln!(f, "  fri_commitments:     {:>10}", self.fri_commitments)?;
        writeln!(f, "  fri_last_layer:      {:>10}", self.fri_last_layer)?;
        writeln!(
            f,
            "  fri_auth:            {:>10} = {} * {q}",
            self.fri_auth_per_query * q,
            self.fri_auth_per_query
        )?;
        writeln!(
            f,
            "  fri_circle_siblings: {:>10} = {} * {q}",
            self.fri_circle_siblings_per_query * q,
            self.fri_circle_siblings_per_query
        )?;
        writeln!(
            f,
            "  fri_line_coset_vals: {:>10} = {} * {q}",
            self.fri_line_coset_vals_per_query * q,
            self.fri_line_coset_vals_per_query
        )?;
        writeln!(f, "  ─────────────────────────────")?;
        let total = self.total();
        writeln!(f, "  total (M31 elements): {total:>10}")?;
        writeln!(f, "  total (bytes):        {:>10}", total * 4)
    }
}

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
        pcs_config: &PcsConfig,
        interaction_pow_bits: u32,
    ) -> Self {
        let components = statement.get_components();
        let n_preprocessed_columns = statement.get_preprocessed_column_ids().len();
        Self::from_components(components, n_preprocessed_columns, pcs_config, interaction_pow_bits)
    }

    pub fn from_components<Value: IValue>(
        components: &[Box<dyn CircuitEval<Value>>],
        n_preprocessed_columns: usize,
        pcs_config: &PcsConfig,
        interaction_pow_bits: u32,
    ) -> Self {
        let trace_columns_per_component =
            components.iter().map(|c| c.trace_columns()).collect_vec();
        let interaction_columns_per_component =
            components.iter().map(|c| c.interaction_columns()).collect_vec();
        Self::new(
            components.len(),
            trace_columns_per_component,
            interaction_columns_per_component,
            n_preprocessed_columns,
            pcs_config,
            interaction_pow_bits,
        )
    }

    pub fn new(
        n_components: usize,
        trace_columns_per_component: Vec<usize>,
        interaction_columns_per_component: Vec<usize>,
        n_preprocessed_columns: usize,
        pcs_config: &PcsConfig,
        interaction_pow_bits: u32,
    ) -> Self {
        let n_interaction_columns = interaction_columns_per_component.iter().sum();
        let mut cumulative_sum_columns = Vec::with_capacity(n_interaction_columns);
        for n_interaction_columns_in_component in &interaction_columns_per_component {
            if *n_interaction_columns_in_component == 0 {
                continue;
            }
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
                stwo::core::fri::FriConfig {
                    log_blowup_factor,
                    n_queries,
                    log_last_layer_degree_bound,
                    line_fold_step,
                },
            lifting_log_size: Some(lifting_log_size),
        } = pcs_config
        else {
            panic!("Lifting log size must be set");
        };

        let log_trace_size = (*lifting_log_size - log_blowup_factor) as usize;

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
                line_fold_step: *line_fold_step as usize,
            },
            interaction_pow_bits,
        }
    }

    /// Returns an iterator over the enabled components.
    pub fn enabled_components(&self) -> impl Iterator<Item = bool> {
        // A real component need to interact with the other components or the public logup sum and
        // therefore it must have some interaction columns.
        self.interaction_columns_per_component
            .iter()
            .map(|interaction_columns| *interaction_columns > 0)
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

    /// Returns the proof size breakdown in M31 elements, computed from config alone.
    pub fn proof_info(&self) -> ProofInfo {
        use crate::fri_proof::compute_all_line_fold_steps;

        let n_queries = self.fri.n_queries;
        let log_eval_domain = self.fri.log_evaluation_domain_size();
        let n_cumsum = self.cumulative_sum_columns.iter().filter(|&&b| b).count();

        // Fixed scalars: channel_salt + 4 roots + proof_of_work_nonce + interaction_pow_nonce.
        let fixed = (1 + 4 * 2 + 1 + 1) * 4;

        // Claim (serialized in packed format).
        let n_enable_qm31 = self.n_components.div_ceil(4);
        let packed_enable_bits = n_enable_qm31.div_ceil(8); // 8 QM31s per u32
        let packed_log_sizes = n_enable_qm31; // 1 u32 per QM31
        let n_enabled = self.enabled_components().filter(|&b| b).count();
        let claimed_sums = n_enabled * 4; // QM31 per enabled component
        let claim = packed_enable_bits + packed_log_sizes + claimed_sums;

        // OODS evaluations.
        let oods = (self.n_preprocessed_columns
            + self.n_trace_columns
            + self.n_interaction_columns
            + n_cumsum
            + N_COMPOSITION_COLUMNS)
            * 4;

        // Eval domain samples per query: M31 values, kept as-is.
        let n_columns_per_trace = self.n_columns_per_trace();
        let total_columns: usize = n_columns_per_trace.iter().sum();
        let eval_samples_per_query = total_columns;

        // Eval domain auth paths per query: N_TRACES trees, each path of depth
        // log_eval_domain, each node is a HashValue (2 QM31).
        let eval_auth_per_query = N_TRACES * log_eval_domain * 2 * 4;

        // FRI proof.
        let line_degree_log_ratio = self.fri.log_trace_size - 1 - self.fri.log_n_last_layer_coefs;
        let all_line_fold_steps =
            compute_all_line_fold_steps(line_degree_log_ratio, self.fri.line_fold_step);
        let n_fri_layers = 1 + all_line_fold_steps.len();

        // FRI commitments: one HashValue per layer.
        let fri_commitments = n_fri_layers * 2 * 4;

        // FRI last layer coefs.
        let fri_last_layer = (1 << self.fri.log_n_last_layer_coefs) * 4;

        // FRI auth paths per query.
        let all_fold_steps_with_circle = {
            let mut v = vec![1usize];
            v.extend_from_slice(&all_line_fold_steps);
            v
        };
        let mut log_layer_size = log_eval_domain;
        let fri_auth_per_query: usize = all_fold_steps_with_circle
            .iter()
            .map(|step| {
                let depth = log_layer_size - step;
                let count = depth * 2 * 4;
                log_layer_size -= step;
                count
            })
            .sum();

        // FRI witness per query: circle_siblings + line coset vals.
        let fri_circle_siblings_per_query = 4;
        let fri_line_coset_vals_per_query: usize =
            all_line_fold_steps.iter().map(|step| (1 << step) * 4).sum();

        let log_trace_size = self.fri.log_trace_size;
        let log_blowup_factor = self.fri.log_blowup_factor;

        ProofInfo {
            log_trace_size,
            log_blowup_factor,
            n_queries,
            n_columns_per_trace,
            fixed,
            claim,
            oods,
            fri_commitments,
            fri_last_layer,
            eval_samples_per_query,
            eval_auth_per_query,
            fri_auth_per_query,
            fri_circle_siblings_per_query,
            fri_line_coset_vals_per_query,
        }
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
#[derive(Clone, Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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
