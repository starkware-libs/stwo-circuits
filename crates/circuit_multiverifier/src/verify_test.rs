use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_verifier::{
    statement::{INTERACTION_POW_BITS, all_circuit_components},
    verify::{CircuitConfig, CircuitPublicData},
};
use circuits::{
    blake::{HashValue, blake},
    context::Context,
    ivalue::NoValue,
};
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use num_traits::Zero;
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};

use super::{Input, build_multiverifier_circuit};

/// Builds a synthetic [`CircuitConfig`] backed by realistic preprocessed column ids.
///
/// The column ids must match what the standard 15 components look up by name (e.g.
/// `finalize_flag`, `blake_sigma_0`, `seq_15`), so we obtain them by running
/// [`PreprocessedCircuit::preprocess_circuit`] on a throwaway context with a single
/// blake gate. The other fields are stub values: the multiverifier consumes them
/// only as M31 constants for logup terms / metadata hashes, so any plausible values
/// drive the build.
fn synthetic_circuit_config() -> CircuitConfig {
    let mut throwaway = Context::<NoValue>::default();
    let zero = throwaway.zero();
    blake(&mut throwaway, &[zero], 1);
    let pp = PreprocessedCircuit::preprocess_circuit(&mut throwaway);

    let mut pcs_config = PcsConfig::default();
    pcs_config.lifting_log_size = Some(23);

    CircuitConfig {
        config: pcs_config,
        // Multiverifier hardcodes a 5-slot output layout; matching length is required
        // by `zip_eq` in `SubCircuitStatement::public_logup_sum`.
        output_addresses: vec![0, 1, 2, 3, 4],
        n_blake_gates: pp.params.n_blake_gates.max(1),
        preprocessed_column_ids: pp.preprocessed_trace.ids(),
        preprocessed_root: HashValue(QM31::zero(), QM31::zero()),
    }
}

fn make_input() -> Input<NoValue> {
    let config = synthetic_circuit_config();
    let components = all_circuit_components::<NoValue>();
    let proof_config = ProofConfig::from_components(
        &components,
        vec![true; components.len()],
        config.preprocessed_column_ids.len(),
        &config.config,
        INTERACTION_POW_BITS,
    );
    let proof = empty_proof(&proof_config);
    let circuit_public_data = CircuitPublicData {
        // Multiverifier reads slots [2] and [3]; provide four dummies.
        output_values: vec![QM31::zero(); 4],
    };
    Input { proof, circuit_public_data, config }
}

/// Topology smoke test: build the multiverifier circuit with `NoValue` and verify
/// the structural invariants.
///
/// Currently expected to fail at `check_yields()` because
/// `build_multiverifier_circuit` is missing a `finalize_guessed_vars()` call —
/// guessed Vars (`metadata_root`, the per-proof `Metadata`, `other_hash`, the
/// `output2/3` payloads, the proof witness Vars) end up with zero yields. Once
/// that call is added the test should pass.
#[test]
fn multiverifier_circuit_builds() {
    let p1 = make_input();
    let p2 = make_input();

    let context = build_multiverifier_circuit::<NoValue>(p1, p2);

    context.check_vars_used();
    context.circuit.check_yields();
    println!("{:?}", context.stats);
}
