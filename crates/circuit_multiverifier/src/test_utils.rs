use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_verifier::{
    statement::{INTERACTION_POW_BITS, all_circuit_components},
    verify::CircuitConfig,
};
use circuits::{
    blake::HashValue,
    context::Context,
    ivalue::{IValue, NoValue},
};
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};

use crate::{
    padding::{pad_components_to_target_counts, qm31_ops_n_rows},
    verify::{
        CommonConfig, Metadata, MetadataTree, MultiverifierInput, build_multiverifier_circuit,
        empty_metadata,
    },
};

pub const N_OUTPUTS: usize = 5;

/// Builds a NoValue context of the multiverifier when it is fed two proofs (with the same
/// parameters) of the subcircuit.
pub fn pp_multiverifier_circuit_from_subcircuit(
    pp_subcircuit: &PreprocessedCircuit,
    pcs_config: PcsConfig,
    target_padding: Option<ComponentSizes>,
) -> (PreprocessedCircuit, Context<NoValue>) {
    let all_circuit_components = &all_circuit_components::<NoValue>();
    let proof_config = ProofConfig::new(
        &all_circuit_components,
        vec![true; all_circuit_components.len()],
        pp_subcircuit.preprocessed_trace.ids().len(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );
    let subcircuit_config = CircuitConfig {
        config: pcs_config,
        output_addresses: pp_subcircuit.params.output_addresses.clone(),
        n_blake_gates: pp_subcircuit.params.n_blake_gates.clone(),
        preprocessed_column_log_sizes: pp_subcircuit.preprocessed_trace.log_sizes(),
        preprocessed_root: HashValue(QM31::from(0), QM31::from(0)),
    };
    // Use a closure to bypass lack of Clone
    let make_input = || MultiverifierInput {
        proof: empty_proof(&proof_config),
        metadata: Metadata::from_config(&subcircuit_config),
        unconstrained_outputs: [QM31::from(0); 2],
        is_multiverifier: false,
    };
    let common_config = CommonConfig {
        proof_config: ProofConfig::new(
            all_circuit_components,
            vec![true; all_circuit_components.len()],
            subcircuit_config.preprocessed_column_log_sizes.len(),
            &subcircuit_config.config,
            INTERACTION_POW_BITS,
        ),
        preprocessed_column_ids: subcircuit_config
            .preprocessed_column_log_sizes
            .keys()
            .cloned()
            .collect(),
    };
    let empty_metadata = empty_metadata(N_OUTPUTS);
    let metadata_tree = MetadataTree::<NoValue>::commit(empty_metadata.clone(), empty_metadata);
    let mut multiverifier_context = build_multiverifier_circuit::<NoValue>(
        make_input(),
        make_input(),
        common_config,
        metadata_tree,
    );
    if let Some(ComponentSizes { eq, qm31_ops, n_blake_gates, n_blake_updates }) = target_padding {
        pad_components_to_target_counts(
            &mut multiverifier_context,
            eq,
            qm31_ops,
            n_blake_gates,
            n_blake_updates,
        );
    }
    let pp = PreprocessedCircuit::preprocess_circuit(&mut multiverifier_context);
    (pp, multiverifier_context)
}

pub fn compute_component_sizes(
    pp_circuit: &PreprocessedCircuit,
    context: &Context<impl IValue>,
) -> ComponentSizes {
    ComponentSizes {
        eq: context.stats.equals,
        qm31_ops: qm31_ops_n_rows(&context.circuit),
        n_blake_gates: pp_circuit.params.n_blake_gates,
        n_blake_updates: context.stats.blake_updates,
    }
}

#[derive(Clone)]
pub struct ComponentSizes {
    pub eq: usize,
    pub qm31_ops: usize,
    pub n_blake_gates: usize,
    pub n_blake_updates: usize,
}

impl std::fmt::Display for ComponentSizes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "eq={}, qm31_ops={}, n_blake={}, blake_compress={}",
            self.eq, self.qm31_ops, self.n_blake_gates, self.n_blake_updates
        ))
    }
}

pub fn get_preprocessed_root(pp: &PreprocessedCircuit, log_blowup_factor: u32) -> HashValue<QM31> {
    use stwo::core::poly::circle::CanonicCoset;
    use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
    use stwo::prover::CommitmentTreeProver;
    use stwo::prover::poly::circle::PolyOps;
    use stwo::prover::mempool::BaseColumnPool;
    use stwo::prover::backend::simd::SimdBackend;

    assert!(log_blowup_factor > 0);
    let lifting_log_size = pp.params.trace_log_size + log_blowup_factor;
    // Compute the multi's preprocessed_root via the commitment tree.
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(lifting_log_size).circle_domain().half_coset,
    );
    let preprocessed_trace = pp.preprocessed_trace.get_trace::<SimdBackend>();
    let preprocessed_trace_polys = SimdBackend::interpolate_columns(preprocessed_trace, &twiddles);
    let preprocessed_tree = CommitmentTreeProver::<SimdBackend, Blake2sM31MerkleChannel>::new(
        preprocessed_trace_polys,
        log_blowup_factor,
        &twiddles,
        true,
        Some(lifting_log_size),
        &BaseColumnPool::<SimdBackend>::new(),
    );
    preprocessed_tree.commitment.root().into()
}
