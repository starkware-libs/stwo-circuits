use std::fs::File;

use cairo_air::utils::binary_deserialize_from_file;
use circuit_air::statement::all_circuit_components;
use circuit_air::verify::{
    CircuitConfig, CircuitPublicData, build_verification_circuit, verify_circuit,
};
use circuit_common::finalize::finalize_context;
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{
    BaseColumnPool, CircuitProof, SimdBackend, preprare_circuit_proof_for_circuit_verifier,
    prove_circuit, prove_circuit_assignment,
};
use circuits::blake::HashValue;
use circuits::context::Context;
use circuits::ivalue::IValue;
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use itertools::Itertools;
use num_traits::Zero;
use stwo::core::fields::qm31::QM31;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;

use crate::privacy::{
    PRIVACY_CAIRO_VERIFIER_CONSTS_HASH, PRIVACY_RECURSION_CIRCUIT_CONSTS_HASH,
    PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT, privacy_cairo_verifier_config, privacy_components,
};
use crate::test::{verify_cairo, verify_cairo_with_component_set};
use crate::utils::get_proof_file_path;
use crate::verify::build_cairo_verifier_circuit;

#[expect(dead_code)]
fn privacy_circuit_preprocessed_root() -> HashValue<QM31> {
    PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT.into()
}

/// Verifies with a circuit a proof of execution of another circuit.
///
/// If `preprocessed_root` is `None`, the verifier takes the preprocessed root from the input proof.
/// This is unsound and is only done to make testing easier.
fn verify_circuit_proof(
    preprocessed_circuit: &PreprocessedCircuit,
    circuit_proof: CircuitProof,
    preprocessed_root: Option<HashValue<QM31>>,
) -> Context<QM31> {
    let preprocessed_column_ids = preprocessed_circuit.preprocessed_trace.ids();
    let proof_config = ProofConfig::from_components(
        &all_circuit_components::<QM31>(),
        preprocessed_column_ids.len(),
        &circuit_proof.pcs_config,
        circuit_air::statement::INTERACTION_POW_BITS,
    );
    let mut circuit_config = CircuitConfig {
        config: circuit_proof.pcs_config,
        output_addresses: preprocessed_circuit.params.output_addresses.clone(),
        n_blake_gates: preprocessed_circuit.params.n_blake_gates,
        preprocessed_column_ids,
        preprocessed_root: HashValue(QM31::zero(), QM31::zero()),
    };
    let (proof, public_data) =
        preprare_circuit_proof_for_circuit_verifier(circuit_proof, &proof_config);
    circuit_config.preprocessed_root = preprocessed_root.unwrap_or(proof.preprocessed_root);
    verify_circuit(circuit_config, proof, public_data).unwrap()
}

/// Compares the topology of two contexts.
/// Note that the values are not compared.
fn compare_contexts_topology<Value: IValue, OtherValue: IValue + 'static>(
    context_a: &Context<Value>,
    context_b: &Context<OtherValue>,
) {
    // TODO(Gali): Consider comparing unused and maybe unused vars.
    assert_eq!(context_a.circuit, context_b.circuit);
    assert_eq!(context_a.stats, context_b.stats);
    assert_eq!(context_a.guessed_vars, context_b.guessed_vars);
    let constants_a = context_a.constants().iter().map(|(k, v)| (*k, v.idx)).collect_vec();
    let constants_b = context_b.constants().iter().map(|(k, v)| (*k, v.idx)).collect_vec();
    assert_eq!(constants_a, constants_b);
}

#[test]
fn test_verify_privacy() {
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();

    // Verify the proof.
    let context = verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();

    // Build the verifier circuit via NoValue.
    let log_blowup_factor = 2;
    let const_config = privacy_cairo_verifier_config(log_blowup_factor);
    let novalue_context = build_cairo_verifier_circuit(&const_config);

    // Check that building the verifier circuit via NoValue produces the same topology.
    compare_contexts_topology(&context, &novalue_context);
}

#[test]
fn test_verify_privacy_with_recursion() {
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();

    let mut context = verify_cairo(&cairo_proof).unwrap();
    let preprocessed = PreprocessedCircuit::preprocess_circuit(&mut context);
    let circuit_proof = prove_circuit_assignment(
        context.values(),
        &preprocessed,
        &BaseColumnPool::<SimdBackend>::new(),
    );
    // To test with a precomputed preprocessed root, change `None` to
    // `Some(privacy_circuit_preprocessed_root())`.
    verify_circuit_proof(&preprocessed, circuit_proof, None);
}

#[test]
fn test_privacy_recursion_with_preprocessed_context() {
    // Build the verifier circuit via NoValue and preprocess it.
    let cairo_proof_log_blowup_factor = 2;
    let const_config = privacy_cairo_verifier_config(cairo_proof_log_blowup_factor);
    let mut novalue_context = build_cairo_verifier_circuit(&const_config);
    let preprocessed = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);

    // Verify the privacy proof to get a Context<QM31> with real values.
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();
    let mut assignment_context =
        verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();

    // Prove via the assignment flow: finalize separately, then prove with pre-computed
    // preprocessed data.
    finalize_context(&mut assignment_context);
    let assignment_proof = prove_circuit_assignment(
        assignment_context.values(),
        &preprocessed,
        &BaseColumnPool::<SimdBackend>::new(),
    );
    assert!(assignment_proof.stark_proof.is_ok());

    // Prove via the full flow for comparison.
    let mut full_prove_context =
        verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();
    let full_proof = prove_circuit(&mut full_prove_context);
    assert!(full_proof.stark_proof.is_ok());

    // Verify both circuit proofs and compare the resulting verifier contexts.
    // TODO(Gali): Add verify fixed circuit
    // TODO(Leo): change `None` to `Some(privacy_circuit_preprocessed_root)` once the changes to the
    // circuit become less frequent.
    let assignment_verifier_context = verify_circuit_proof(&preprocessed, assignment_proof, None);

    let full_prove_preprocessed =
        PreprocessedCircuit::from_finalized_circuit(&full_prove_context.circuit);
    let full_verifier_context = verify_circuit_proof(&full_prove_preprocessed, full_proof, None);

    // Compare the verifier contexts.
    compare_contexts_topology(&assignment_verifier_context, &full_verifier_context);
    assert_eq!(assignment_verifier_context.values(), full_verifier_context.values());
}

/// Builds the Cairo verifier circuit with QM31 values for given PCS parameters.
fn build_cairo_verifier_qm31(
    log_blowup_factor: u32,
    n_queries: usize,
    line_fold_step: u32,
) -> Context<QM31> {
    use crate::all_components::all_components;
    use crate::preprocessed_columns::PREPROCESSED_COLUMNS_ORDER;
    use crate::statement::{CairoStatement, MEMORY_VALUES_LIMBS, PUBLIC_DATA_LEN};
    use crate::verify::{CairoVerifierConfig, get_preprocessed_root};
    use cairo_air::verifier::INTERACTION_POW_BITS;
    use circuits::context::TraceContext;
    use circuits::ops::Guess as _;
    use circuits_stark_verifier::constraint_eval::CircuitEval;
    use circuits_stark_verifier::empty_component::EmptyComponent;
    use circuits_stark_verifier::proof::{ProofConfig, empty_proof_generic};
    use circuits_stark_verifier::verify::verify;
    use stwo::core::fields::m31::M31;

    let privacy_set = privacy_components();
    let components: Vec<Box<dyn CircuitEval<QM31>>> = all_components::<QM31>()
        .into_iter()
        .map(|(name, component)| -> Box<dyn CircuitEval<QM31>> {
            if privacy_set.contains(name) { component } else { Box::new(EmptyComponent {}) }
        })
        .collect_vec();

    let lifting_log_size = 20 + log_blowup_factor;
    let pcs_config = PcsConfig {
        pow_bits: 26,
        fri_config: stwo::core::fri::FriConfig::new(
            0,
            log_blowup_factor,
            n_queries,
            line_fold_step,
        ),
        lifting_log_size: Some(lifting_log_size),
    };

    let proof_config = ProofConfig::from_components(
        &components,
        PREPROCESSED_COLUMNS_ORDER.len(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );

    let program_path =
        crate::utils::get_test_data_dir().join("privacy/privacy_simple_bootloader_compiled.json");
    let program = crate::utils::load_program(&program_path);
    let n_outputs = 1;

    let qm31_proof = empty_proof_generic::<QM31>(&proof_config);
    let public_claim = vec![0u32; PUBLIC_DATA_LEN + n_outputs + program.len()];
    let outputs = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; n_outputs];

    let verifier_config = CairoVerifierConfig {
        preprocessed_root: get_preprocessed_root(lifting_log_size),
        proof_config,
        program,
        n_outputs,
    };
    crate::verify::build_fixed_cairo_circuit(&verifier_config, qm31_proof, public_claim, outputs)
}

fn print_circuit_stats(label: &str, context: &Context<QM31>) {
    use circuit_air::statement::all_circuit_components;
    use circuits_stark_verifier::constraint_eval::CircuitEval;

    let c = &context.circuit;
    let perm_rows: usize = c.permutation.iter().map(|p| 2 * p.inputs.len()).sum();
    let qm31_ops_rows =
        c.add.len() + c.sub.len() + c.mul.len() + c.pointwise_mul.len() + perm_rows;
    let qm31_ops_trace = qm31_ops_rows.next_power_of_two();
    let eq_trace = std::cmp::max(c.eq.len().next_power_of_two(), 16);

    println!("=== {label} ===");
    println!(
        "  qm31_ops: rows={} trace=2^{} ({}) | eq: rows={} trace=2^{} ({})",
        qm31_ops_rows,
        qm31_ops_trace.ilog2(),
        qm31_ops_trace,
        c.eq.len(),
        eq_trace.ilog2(),
        eq_trace,
    );

    // Compute total cells for existing AIR components (trace + interaction).
    // qm31_ops: 12 trace + 8 interaction = 20 cols
    // eq: 4 trace + 4 interaction = 8 cols
    let mut total_cells: u64 = 0;
    total_cells += 20 * qm31_ops_trace as u64;
    total_cells += 8 * eq_trace as u64;

    // Blake AIR components (monolithic) — sizes derived from blake_updates.
    let n_blake_compress = c.blake.iter().map(|g| g.input.len()).sum::<usize>();
    let n_blake_compress_padded = std::cmp::max(n_blake_compress.next_power_of_two(), 16);
    let n_blake_output = c.blake.len();
    let n_blake_output_padded = std::cmp::max(n_blake_output.next_power_of_two(), 16);

    if n_blake_compress > 0 {
        // blake_gate: 151+136=287 cols, 1 row per compression
        let blake_gate_cells = 287u64 * n_blake_compress_padded as u64;
        // blake_round: 148+56=204 cols, 10 rows per compression
        let blake_round_rows = std::cmp::max((n_blake_compress * 10).next_power_of_two(), 16);
        let blake_round_cells = 204u64 * blake_round_rows as u64;
        // blake_g: 53+36=89 cols, trace = compress_padded * 2^7
        let blake_g_trace = n_blake_compress_padded * 128;
        let blake_g_cells = 89u64 * blake_g_trace as u64;
        // triple_xor_32: 21+20=41 cols, 8 rows per compression
        let triple_xor_rows = std::cmp::max((n_blake_compress * 8).next_power_of_two(), 16);
        let triple_xor_cells = 41u64 * triple_xor_rows as u64;
        // blake_output: 24+8=32 cols
        let blake_output_cells = 32u64 * n_blake_output_padded as u64;
        // blake_round_sigma: 1+4=5 cols, 16 rows
        let blake_sigma_cells = 5u64 * 16;

        let blake_total = blake_gate_cells + blake_round_cells + blake_g_cells
            + triple_xor_cells + blake_output_cells + blake_sigma_cells;
        total_cells += blake_total;
        println!(
            "  blake components: compress={} (padded {}), output={} (padded {})",
            n_blake_compress, n_blake_compress_padded, n_blake_output, n_blake_output_padded,
        );
        println!(
            "    blake_gate: {}×{}={:.1}M | blake_round: {}×{}={:.1}M | blake_g: {}×{}={:.1}M",
            287, n_blake_compress_padded, blake_gate_cells as f64 / 1e6,
            204, blake_round_rows, blake_round_cells as f64 / 1e6,
            89, blake_g_trace, blake_g_cells as f64 / 1e6,
        );
        println!(
            "    triple_xor: {}×{}={:.1}M | blake_output: {}×{}={:.1}M | sigma: {}×16={:.0}",
            41, triple_xor_rows, triple_xor_cells as f64 / 1e6,
            32, n_blake_output_padded, blake_output_cells as f64 / 1e6,
            5, blake_sigma_cells,
        );
        println!("    blake total: {:.1}M cells", blake_total as f64 / 1e6);
    }

    // New decomposed gate components.
    // From our AIR design:
    // M31ToU32: 6 columns (input_addr, input_value, low, high, inverse, output_addr)
    //   + interaction: 2 lookups × 4 = 8 → total 14 cols
    // BlakeG: 31 columns (6 in_addr + 6 low + 6 high + 4 triple_sum + 4 xor_rot + 4 out_addr + mult)
    //   + interaction: 10 lookups × 4 = 40 → total 71 cols
    // TripleXor: 24 columns (3 in_addr + 3 low + 3 high + 6 splits + 8 xors + out_addr)
    //   + interaction: 4 lookups × 4 = 16 → total 40 cols
    let n_m31_to_u32 = c.m31_to_u32.len();
    let n_blake_g = c.blake_g.len();
    let n_triple_xor = c.triple_xor.len();
    if n_blake_g > 0 {
        let m31_to_u32_trace = std::cmp::max(n_m31_to_u32.next_power_of_two(), 16);
        let blake_g_trace = std::cmp::max(n_blake_g.next_power_of_two(), 16);
        let triple_xor_trace = std::cmp::max(n_triple_xor.next_power_of_two(), 16);

        let m31_cells = 14u64 * m31_to_u32_trace as u64;
        let bg_cells = 71u64 * blake_g_trace as u64;
        let tx_cells = 40u64 * triple_xor_trace as u64;
        let decomposed_total = m31_cells + bg_cells + tx_cells;
        total_cells += decomposed_total;

        println!(
            "  decomposed gates: m31_to_u32={} blake_g={} triple_xor={}",
            n_m31_to_u32, n_blake_g, n_triple_xor
        );
        println!(
            "    m31_to_u32: 14×{}={:.1}M | blake_g: 71×{}={:.1}M | triple_xor: 40×{}={:.1}M",
            m31_to_u32_trace, m31_cells as f64 / 1e6,
            blake_g_trace, bg_cells as f64 / 1e6,
            triple_xor_trace, tx_cells as f64 / 1e6,
        );
        println!("    decomposed total: {:.1}M cells", decomposed_total as f64 / 1e6);
    }

    println!("  TOTAL trace cells: {:.1}M", total_cells as f64 / 1e6);
    println!();
}

#[test]
fn test_privacy_consts() {
    let configs = [
        (2, 35, 4), // blowup=2, 35 queries, fold=4
        (3, 23, 4), // blowup=3, 23 queries, fold=4
    ];
    for (log_blowup, n_queries, fold_step) in configs {
        let label = format!("blowup={log_blowup} queries={n_queries} fold={fold_step}");

        // Monolithic blake.
        unsafe { std::env::remove_var("USE_BLAKE_GATES") };
        let ctx_mono = build_cairo_verifier_qm31(log_blowup, n_queries, fold_step);
        print_circuit_stats(&format!("Monolithic ({label})"), &ctx_mono);

        // Decomposed blake.
        unsafe { std::env::set_var("USE_BLAKE_GATES", "1") };
        let ctx_gates = build_cairo_verifier_qm31(log_blowup, n_queries, fold_step);
        print_circuit_stats(&format!("Decomposed ({label})"), &ctx_gates);
    }
    unsafe { std::env::remove_var("USE_BLAKE_GATES") };
}
