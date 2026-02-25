use cairo_air::CairoProof;
use cairo_air::flat_claims::FlatClaim;
use circuit_prover::prover::prove_circuit;
use circuit_prover::prover::verify_circuit;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use itertools::Itertools;
use itertools::zip_eq;
use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;

use cairo_air::utils::{binary_deserialize_from_file, binary_serialize_to_file};
use std::array;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;

use crate::all_components::all_components;
use crate::preprocessed_columns::MAX_SEQUENCE_LOG_SIZE;
use crate::preprocessed_columns::PREPROCESSED_COLUMNS_ORDER;
use crate::privacy::privacy_components;
use crate::statement::CairoStatement;
use crate::statement::{MEMORY_VALUES_LIMBS, PUBLIC_DATA_LEN};
use crate::verify::CairoVerifierConfig;
use crate::verify::prepare_cairo_proof_for_circuit_verifier;
use crate::verify::verify_fixed_cairo_circuit;
use cairo_air::PreProcessedTraceVariant;
use circuit_prover::prover::preprare_circuit_proof_for_circuit_verifier;
use circuits::{context::Context, ivalue::NoValue, ops::Guess};
use circuits_stark_verifier::{
    empty_component::EmptyComponent,
    proof::{ProofConfig, empty_proof},
    verify::verify,
};
use stwo::core::fri::FriConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo_cairo_dev_utils::utils::get_compiled_cairo_program_path;
use stwo_cairo_dev_utils::vm_utils::{ProgramType, run_and_adapt};
use stwo_cairo_prover::prover::{ChannelHash, ProverParameters, prove_cairo};

use cairo_air::verifier::INTERACTION_POW_BITS;
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

/// Circuit Verifies a [CairoProof].
fn verify_cairo(proof: &CairoProof<Blake2sM31MerkleHasher>) -> Result<Context<QM31>, String> {
    let FlatClaim { component_enable_bits, component_log_sizes: _, public_data: _ } =
        proof.claim.flatten_claim();

    let components = HashSet::from_iter(
        zip_eq(all_components::<QM31>().into_keys(), &component_enable_bits)
            .filter(|(_, enable_bit)| **enable_bit)
            .map(|(component_name, _)| component_name),
    );

    verify_cairo_with_component_set(proof, components)
}

/// Verifies a [CairoProof] with a given set of components.
fn verify_cairo_with_component_set(
    cairo_proof: &CairoProof<Blake2sM31MerkleHasher>,
    component_set: HashSet<&str>,
) -> Result<Context<QM31>, String> {
    let FlatClaim { component_enable_bits, component_log_sizes: _, public_data: _ } =
        cairo_proof.claim.flatten_claim();
    let components: Vec<Box<dyn CircuitEval<QM31>>> =
        zip_eq(all_components::<QM31>().into_iter(), &component_enable_bits)
            .map(|((component_name, component), &enable_bit)| {
                let component_in_set = component_set.contains(component_name);
                if component_in_set != enable_bit {
                    return Err(format!(
                        "Proof was produced with the wrong components set: expected the component '{}' to be {} according to the component set, but it is {} in the proof.",
                        component_name,
                        if component_in_set { "enabled" } else { "disabled" },
                        if enable_bit { "enabled" } else { "disabled" }
                    ));
                }
                Ok(if enable_bit { component } else { Box::new(EmptyComponent {}) })
            })
            .try_collect()?;

    let proof_config = ProofConfig::from_components(
        &components,
        PREPROCESSED_COLUMNS_ORDER.len(),
        &cairo_proof.extended_stark_proof.proof.config,
        INTERACTION_POW_BITS,
    );

    let (proof, public_data) = prepare_cairo_proof_for_circuit_verifier(cairo_proof, &proof_config);
    let (public_claim, outputs, program) = public_data.pack_into_u32s();
    let outputs = outputs
        .chunks_exact(MEMORY_VALUES_LIMBS)
        .map(|chunk| array::from_fn(|i| M31::from_u32_unchecked(chunk[i])))
        .collect_vec();
    let program = program
        .chunks_exact(MEMORY_VALUES_LIMBS)
        .map(|chunk| array::from_fn(|i| M31::from_u32_unchecked(chunk[i])))
        .collect_vec();

    let verifier_config = CairoVerifierConfig {
        proof_config,
        program,
        n_outputs: cairo_proof.claim.public_data.public_memory.output.len(),
    };

    verify_fixed_cairo_circuit(verifier_config, proof, public_claim, outputs)
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

fn get_proof_file_path(test_name: &str) -> PathBuf {
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
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();

    verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();
}

#[test]
fn test_verify_privacy_with_recursion() {
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();

    let mut context = verify_cairo(&cairo_proof).unwrap();
    let circuit_proof = prove_circuit(&mut context);
    let pcs_config = circuit_proof.pcs_config;
    let (proof, public_data) = preprare_circuit_proof_for_circuit_verifier(circuit_proof);
    verify_circuit(pcs_config, proof, public_data).unwrap();
}
