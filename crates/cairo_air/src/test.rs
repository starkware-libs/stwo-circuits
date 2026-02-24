use itertools::Itertools;
use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;

use cairo_air::utils::{binary_deserialize_from_file, binary_serialize_to_file};
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::time::Instant;

use crate::all_components::all_components;
use crate::preprocessed_columns::MAX_SEQUENCE_LOG_SIZE;
use crate::statement::CairoStatement;
use crate::statement::{MEMORY_VALUES_LIMBS, PUBLIC_DATA_LEN};
use crate::verify::verify_cairo;
use cairo_air::{CairoProof, PreProcessedTraceVariant};
use circuit_air::components::N_COMPONENTS;
use circuit_air::statement::{
    CircuitStatement, INTERACTION_POW_BITS as CIRCUIT_INTERACTION_POW_BITS,
};
use circuits::{context::Context, ivalue::NoValue, ops::Guess};
use circuits_stark_verifier::{
    empty_component::EmptyComponent,
    proof::{Claim, ProofConfig, empty_proof},
    proof_from_stark_proof::{pack_component_log_sizes, pack_enable_bits, proof_from_stark_proof},
    verify::verify,
};
use stwo::core::fri::FriConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;
use stwo_cairo_dev_utils::utils::get_compiled_cairo_program_path;
use stwo_cairo_dev_utils::vm_utils::{ProgramType, run_and_adapt};
use stwo_cairo_prover::prover::{ChannelHash, ProverParameters, prove_cairo};

/// Logup security is defined by the `QM31` space (~124 bits) + `INTERACTION_POW_BITS` -
/// log2(number of relation terms).
/// The number of relation terms is defined as n_terms * n_relations * n_uses, where:
/// n_terms = number of terms in each relation (the size of the relation entry) < 2^7,
/// n_relations = number of different relations ids < 2^6,
/// n_uses is bounded by the characteristic of the field = 2^31.
/// E.g. assuming a 100-bit security target, the witness may contain up to
/// 1 << (24 + INTERACTION_POW_BITS) relation terms.
pub const INTERACTION_POW_BITS: u32 = 24;

use circuits::context::Var;
use circuits::ivalue::IValue;
use circuits_stark_verifier::constraint_eval::ComponentDataTrait;
use circuits_stark_verifier::proof::InteractionAtOods;

pub struct TestComponentData {
    trace: Vec<Var>,
    interaction_trace: Vec<InteractionAtOods<Var>>,
    claimed_sum: Var,
    n_instances_var: Var,
    n_instances_bits: Vec<Var>,
}

impl TestComponentData {
    pub fn from_values(
        context: &mut Context<QM31>,
        trace_values: &[QM31],
        interaction_values: &[QM31],
        last_row_sum: QM31,
        claimed_sum: QM31,
        n_instances: usize,
    ) -> Self {
        let trace = trace_values.iter().map(|v| context.new_var(*v)).collect_vec();
        let mut interaction_trace = interaction_values
            .iter()
            .flat_map(|v| v.to_m31_array())
            .map(|m31| InteractionAtOods { at_oods: context.new_var(m31.into()), at_prev: None })
            .collect_vec();
        if !interaction_trace.is_empty() {
            let last_row_sum_m31s = last_row_sum.to_m31_array();
            let interaction_trace_len = interaction_trace.len();
            for i in 0..4 {
                interaction_trace[interaction_trace_len - 4 + i].at_prev =
                    Some(context.new_var(last_row_sum_m31s[i].into()));
            }
        }
        let n_instances_bits = (0..31)
            .map(|bit_pos| {
                let bit = (n_instances >> bit_pos) & 1;
                context.new_var(bit.into())
            })
            .collect_vec();
        Self {
            trace,
            interaction_trace,
            claimed_sum: context.new_var(claimed_sum),
            n_instances_var: context.new_var(n_instances.into()),
            n_instances_bits,
        }
    }
}

impl<Value: IValue> ComponentDataTrait<Value> for TestComponentData {
    fn trace_columns(&self) -> &[Var] {
        &self.trace
    }

    fn interaction_columns(&self) -> &[InteractionAtOods<Var>] {
        &self.interaction_trace
    }

    fn n_instances(&self) -> Var {
        self.n_instances_var
    }

    fn claimed_sum(&self) -> Var {
        self.claimed_sum
    }

    fn get_n_instances_bit(&self, _context: &mut Context<Value>, bit: usize) -> Var {
        self.n_instances_bits[bit]
    }

    fn max_component_size_bits(&self) -> usize {
        self.n_instances_bits.len()
    }
}

#[test]
fn test_verify() {
    let mut pcs_config = PcsConfig::default();
    pcs_config.lifting_log_size =
        Some(MAX_SEQUENCE_LOG_SIZE as u32 + pcs_config.fri_config.log_blowup_factor);

    let mut novalue_context = Context::<NoValue>::default();
    let output_len = 1;
    let program_len = 128;
    let flat_claim = vec![M31::zero(); PUBLIC_DATA_LEN + output_len + program_len];
    let outputs = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; output_len];
    let program = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; program_len];
    let mut statement = CairoStatement::new(&mut novalue_context, flat_claim, outputs, program);
    // Remove the pedersen points table component since it requires long preprocessed columns, which
    // are not supported.
    let pedersen_points_index =
        all_components::<NoValue>().get_full("pedersen_points_table_window_bits_18").unwrap().0;
    statement.components[pedersen_points_index] = Box::new(EmptyComponent {});

    let config = ProofConfig::from_statement(&statement, &pcs_config, INTERACTION_POW_BITS);

    let empty_proof = empty_proof(&config);

    let proof_vars = empty_proof.guess(&mut novalue_context);
    verify(&mut novalue_context, &proof_vars, &config, &statement);
    novalue_context.finalize_guessed_vars();
    novalue_context.check_vars_used();
    novalue_context.circuit.check_yields();

    println!("Stats: {:?}", novalue_context.stats);
}

pub fn get_proof_file_path(test_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test_data/")
        .join(test_name)
        .join("proof.bin")
}

#[test]
fn test_verify_all_opcodes() {
    let proof_path = get_proof_file_path("all_opcode_components");
    let low_blowup_factor = 1;

    if std::env::var("FIX_PROOF").is_ok() {
        let compiled_program =
            get_compiled_cairo_program_path("test_prove_verify_all_opcode_components");
        let input = run_and_adapt(&compiled_program, ProgramType::Json, None).unwrap();
        let prover_params = ProverParameters {
            channel_hash: ChannelHash::Blake2sM31,
            pcs_config: PcsConfig {
                pow_bits: 26,
                fri_config: FriConfig::new(0, low_blowup_factor, 70, 1),
                lifting_log_size: Some(20 + low_blowup_factor),
            },
            preprocessed_trace: PreProcessedTraceVariant::CanonicalSmall,
            channel_salt: 0,
            store_polynomials_coefficients: false,
            include_all_preprocessed_columns: true,
        };
        let cairo_proof = prove_cairo::<Blake2sM31MerkleChannel>(input, prover_params).unwrap();

        let proof_file =
            OpenOptions::new().create(true).write(true).truncate(true).open(&proof_path).unwrap();
        binary_serialize_to_file(&cairo_proof, &proof_file).unwrap();
    }

    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();

    let context = verify_cairo(&cairo_proof).unwrap();
    println!("Stats: {:?}", context.stats);
}

#[test]
fn test_verify_privacy() {
    let started_at = Instant::now();
    let checkpoint = |label: &str| {
        println!(
            "[test_verify_privacy][{:>8} ms] {}",
            started_at.elapsed().as_millis(),
            label
        );
    };
    checkpoint("start");

    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof: CairoProof<Blake2sM31MerkleHasher> =
        binary_deserialize_from_file(&proof_file).unwrap();
    println!("pcs config: {:?}", cairo_proof.extended_stark_proof.proof.config);
    checkpoint("loaded cairo proof");

    let mut cairo_verifier_context = verify_cairo(&cairo_proof).unwrap();
    println!("cairo verifier stats: {:?}", cairo_verifier_context.stats);
    checkpoint("verified cairo proof");

    let circuit_prover::prover::CircuitProof {
        pcs_config: first_pcs_config,
        preprocessed_circuit: first_preprocessed_circuit,
        claim: first_claim_data,
        interaction_pow_nonce: first_interaction_pow_nonce,
        interaction_claim: first_interaction_claim,
        stark_proof: first_stark_proof,
        channel_salt: first_channel_salt,
        ..
    } = circuit_prover::prover::prove_circuit(&mut cairo_verifier_context);
    checkpoint("first prove_circuit done");
    println!("first prove trace_log_size: {}", first_preprocessed_circuit.params.trace_log_size);
    assert!(first_stark_proof.is_ok());
    // let first_stark_proof = first_stark_proof.unwrap();

    // let mut circuit_verifier_context = Context::<QM31>::default();
    // let first_statement = CircuitStatement::new(
    //     &mut circuit_verifier_context,
    //     &first_preprocessed_circuit.params.output_addresses,
    //     &first_claim_data.output_values,
    //     first_preprocessed_circuit.params.n_blake_gates,
    //     first_preprocessed_circuit.preprocessed_trace.ids(),
    // );
    // let first_claim = Claim {
    //     packed_enable_bits: pack_enable_bits(&[true; N_COMPONENTS]),
    //     packed_component_log_sizes: pack_component_log_sizes(&first_claim_data.log_sizes),
    //     claimed_sums: first_interaction_claim.claimed_sums.to_vec(),
    // };
    // let first_config = ProofConfig::from_statement(
    //     &first_statement,
    //     &first_pcs_config,
    //     CIRCUIT_INTERACTION_POW_BITS,
    // );
    // let first_proof = proof_from_stark_proof(
    //     &first_stark_proof,
    //     &first_config,
    //     first_claim,
    //     first_interaction_pow_nonce,
    //     first_channel_salt,
    // );
    // let first_proof_vars = first_proof.guess(&mut circuit_verifier_context);
    // verify(&mut circuit_verifier_context, &first_proof_vars, &first_config, &first_statement);
    // println!("circuit verifier stats: {:?}", circuit_verifier_context.stats);
    // circuit_verifier_context.finalize_guessed_vars();
    // circuit_verifier_context.validate_circuit();
    // checkpoint("first circuit verification done");

    // let circuit_proof = circuit_prover::prover::prove_circuit(&mut circuit_verifier_context);
    // checkpoint("second prove_circuit done");
    // let dump_path = std::env::temp_dir().join("circuit_proof_no_preprocessed.bin");
    // let dump_file = File::create(&dump_path).unwrap();
    // let stark_proof = circuit_proof.stark_proof.as_ref().unwrap();
    // let output_values = circuit_proof
    //     .claim
    //     .output_values
    //     .iter()
    //     .map(|v| v.to_m31_array().map(|x| x.0))
    //     .collect_vec();
    // let claimed_sums = circuit_proof
    //     .interaction_claim
    //     .claimed_sums
    //     .iter()
    //     .map(|v| v.to_m31_array().map(|x| x.0))
    //     .collect_vec();
    // let proof_dump = (
    //     format!("{:?}", circuit_proof.pcs_config),
    //     circuit_proof.claim.log_sizes,
    //     output_values,
    //     circuit_proof.interaction_pow_nonce,
    //     claimed_sums,
    //     circuit_proof.channel_salt,
    //     stark_proof,
    // );
    // // binary_serialize_to_file(&proof_dump, &dump_file).unwrap();
    // println!(
    //     "dumped circuit proof payload (without preprocessed_circuit/components) to {}",
    //     dump_path.display()
    // );
    // let circuit_prover::prover::CircuitProof {
    //     pcs_config: second_pcs_config,
    //     preprocessed_circuit: second_preprocessed_circuit,
    //     claim: second_claim_data,
    //     interaction_pow_nonce: second_interaction_pow_nonce,
    //     interaction_claim: second_interaction_claim,
    //     stark_proof: second_stark_proof,
    //     channel_salt: second_channel_salt,
    //     ..
    // } = circuit_proof;
    // assert!(circuit_proof.stark_proof.is_ok());
    // checkpoint("end");
    // let second_stark_proof = second_stark_proof.unwrap();

    // let mut second_circuit_verifier_context = Context::<QM31>::default();
    // let second_statement = CircuitStatement::new(
    //     &mut second_circuit_verifier_context,
    //     &second_preprocessed_circuit.params.output_addresses,
    //     &second_claim_data.output_values,
    //     second_preprocessed_circuit.params.n_blake_gates,
    //     second_preprocessed_circuit.preprocessed_trace.ids(),
    // );
    // let second_claim = Claim {
    //     packed_enable_bits: pack_enable_bits(&[true; N_COMPONENTS]),
    //     packed_component_log_sizes: pack_component_log_sizes(&second_claim_data.log_sizes),
    //     claimed_sums: second_interaction_claim.claimed_sums.to_vec(),
    // };
    // let second_config = ProofConfig::from_statement(
    //     &second_statement,
    //     &second_pcs_config,
    //     CIRCUIT_INTERACTION_POW_BITS,
    // );
    // let second_proof = proof_from_stark_proof(
    //     &second_stark_proof,
    //     &second_config,
    //     second_claim,
    //     second_interaction_pow_nonce,
    //     second_channel_salt,
    // );
    // let second_proof_vars = second_proof.guess(&mut second_circuit_verifier_context);
    // verify(
    //     &mut second_circuit_verifier_context,
    //     &second_proof_vars,
    //     &second_config,
    //     &second_statement,
    // );
    // println!("second circuit verifier stats: {:?}", second_circuit_verifier_context.stats);
}
