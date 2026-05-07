use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_verifier::{
    statement::{INTERACTION_POW_BITS, all_circuit_components},
    verify::{CircuitConfig, CircuitPublicData},
};
use circuits::{
    blake::HashValue,
    context::Context,
    ivalue::{IValue, NoValue},
};
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use num_traits::Zero;
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};

use crate::{
    padding::{pad_components_to_target_counts, qm31_ops_n_rows},
    verify::{
        MetadataTree, SubCircuitConfig, SubCircuitInput, build_multiverifier_circuit,
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
    let proof_config = ProofConfig::from_components(
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
        preprocessed_column_ids: pp_subcircuit.preprocessed_trace.ids().clone(),
        preprocessed_root: HashValue(QM31::from(0), QM31::from(0)),
    };
    // Use a closure to bypass lack of Clone
    let make_input = || SubCircuitInput {
        proof: empty_proof(&proof_config),
        circuit_public_data: CircuitPublicData { output_values: vec![QM31::zero(); N_OUTPUTS] },
        config: subcircuit_config.clone(),
        is_multiverifier: false,
    };
    let subcircuit_config = SubCircuitConfig {
        pcs_config: subcircuit_config.config,
        n_outputs: N_OUTPUTS,
        preprocessed_column_ids: subcircuit_config.preprocessed_column_ids.clone(),
    };
    let empty_metadata = empty_metadata(N_OUTPUTS);
    let metadata_tree = MetadataTree::<NoValue>::commit(empty_metadata.clone(), empty_metadata);
    let mut multiverifier_context = build_multiverifier_circuit::<NoValue>(
        make_input(),
        make_input(),
        subcircuit_config,
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
    println!(
        "Ingested two proofs of trace_log_size = {} and fri_config = {:?} --> Multiverifier has trace_log_size = {}",
        pp_subcircuit.params.trace_log_size, pcs_config.fri_config, pp.params.trace_log_size
    );
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
