use circuit_common::N_LANES;
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
use stwo::core::{fields::qm31::QM31, fri::FriConfig, pcs::PcsConfig};

use crate::verify::{MultiverifierInput, SharedConfig, build_multiverifier_circuit};
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::CommitmentTreeProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::mempool::BaseColumnPool;
use stwo::prover::poly::circle::PolyOps;

/// Builds a `NoValue` multiverifier and preprocesses it. The multiverifier is build by feeding it
/// two identical proofs of a circuit.
pub fn get_preprocessed_multiverifier_from_circuit(
    preprocessed_leaf_circuit: &PreprocessedCircuit,
    pcs_config: PcsConfig,
) -> (PreprocessedCircuit, Context<NoValue>) {
    let all_circuit_components = &all_circuit_components::<NoValue>();
    let proof_config = ProofConfig::new(
        all_circuit_components,
        vec![true; all_circuit_components.len()],
        preprocessed_leaf_circuit.preprocessed_trace.n_columns(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );
    let subcircuit_config = CircuitConfig {
        config: pcs_config,
        output_addresses: preprocessed_leaf_circuit.params.output_addresses.clone(),
        preprocessed_column_log_sizes: preprocessed_leaf_circuit.preprocessed_trace.log_sizes(),
        preprocessed_root: HashValue(QM31::from(0), QM31::from(0)),
    };
    let empty_input = || MultiverifierInput {
        proof: empty_proof(&proof_config),
        preprocessed_root: subcircuit_config.preprocessed_root,
        hashed_outputs: [QM31::from(0); 2],
    };
    let shared_config = SharedConfig {
        proof_config: ProofConfig::new(
            all_circuit_components,
            vec![true; all_circuit_components.len()],
            proof_config.n_preprocessed_columns,
            &subcircuit_config.config,
            INTERACTION_POW_BITS,
        ),
        preprocessed_column_log_sizes: subcircuit_config.preprocessed_column_log_sizes.clone(),
        output_addresses: subcircuit_config.output_addresses,
    };
    let mut multiverifier_context =
        build_multiverifier_circuit::<NoValue>(empty_input(), empty_input(), &shared_config);
    let preprocessed_multiverifier_circuit =
        PreprocessedCircuit::preprocess_circuit(&mut multiverifier_context);
    (preprocessed_multiverifier_circuit, multiverifier_context)
}

#[expect(dead_code)]
pub fn get_preprocessed_root(
    preprocessed_circuit: &PreprocessedCircuit,
    log_blowup_factor: u32,
) -> HashValue<QM31> {
    let lifting_log_size = preprocessed_circuit.params.trace_log_size + log_blowup_factor;
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(lifting_log_size).circle_domain().half_coset,
    );
    let preprocessed_trace = preprocessed_circuit.preprocessed_trace.get_trace::<SimdBackend>();
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

#[derive(Clone)]
pub struct ComponentLogSizes {
    pub eq: u32,
    pub qm31_ops: u32,
    pub m31_to_u32: u32,
    pub triple_xor: u32,
    pub blake_g_gate: u32,
}

impl std::fmt::Display for ComponentLogSizes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "eq={:>2}  qm31_ops={:>2}  m31_to_u32={:>2}  triple_xor={:>2}  blake_g_gate={:>2}",
            self.eq, self.qm31_ops, self.m31_to_u32, self.triple_xor, self.blake_g_gate
        ))
    }
}

pub fn compute_component_sizes(context: &Context<impl IValue>) -> ComponentLogSizes {
    let circuit = &context.circuit;
    let qm31_ops_n_rows = circuit.add.len()
        + circuit.sub.len()
        + circuit.mul.len()
        + circuit.pointwise_mul.len()
        + circuit.permutation.iter().map(|p| p.inputs.len() + p.outputs.len()).sum::<usize>();
    ComponentLogSizes {
        eq: padded_log_size(circuit.eq.len()),
        qm31_ops: padded_log_size(qm31_ops_n_rows),
        m31_to_u32: padded_log_size(circuit.m31_to_u32.len()),
        triple_xor: padded_log_size(circuit.triple_xor.len()),
        blake_g_gate: padded_log_size(circuit.blake_g_gate.len()),
    }
}

fn padded_log_size(n_rows: usize) -> u32 {
    std::cmp::max(n_rows.next_power_of_two(), N_LANES).ilog2()
}

pub const fn get_pcs_config(trace_log_size: u32, log_blowup_factor: u32) -> PcsConfig {
    let (pow_bits, n_queries) = match log_blowup_factor {
        1 => (26, 70),
        2 => (26, 35),
        3 => (27, 23),
        _ => panic!("Unsupported log blowup factor."),
    };
    let fri_config =
        FriConfig { log_blowup_factor, log_last_layer_degree_bound: 0, n_queries, fold_step: 4 };
    PcsConfig { pow_bits, fri_config, lifting_log_size: Some(trace_log_size + log_blowup_factor) }
}
