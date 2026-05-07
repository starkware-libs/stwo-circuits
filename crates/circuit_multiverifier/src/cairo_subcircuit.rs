use std::array;
use std::collections::HashSet;

use cairo_air::CairoProof;
use cairo_air::flat_claims::FlatClaim;
use cairo_air::verifier::INTERACTION_POW_BITS as CAIRO_INTERACTION_POW_BITS;
use circuit_cairo_verifier::all_components::all_components as cairo_all_components;
use circuit_cairo_verifier::statement::{CairoStatement, MEMORY_VALUES_LIMBS, PUBLIC_DATA_LEN};
use circuit_cairo_verifier::verify::{
    CairoVerifierConfig, enabled_components, prepare_cairo_proof_for_circuit_verifier,
};
use circuits::context::{Context, TraceContext};
use circuits::finalize_constants::finalize_constants;
use circuits::ivalue::{IValue, NoValue};
use circuits::ops::{Guess, guess, output};
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::proof::{Proof, ProofConfig, empty_proof};
use circuits_stark_verifier::verify::verify;
use itertools::{Itertools, zip_eq};
use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;

fn add_dummy_outputs<Value: IValue>(context: &mut Context<Value>) {
    let dummy_0 = guess(context, Value::from_qm31(QM31::zero()));
    let dummy_1 = guess(context, Value::from_qm31(QM31::zero()));
    output(context, dummy_0);
    output(context, dummy_1);
}

/// Like [`circuit_cairo_verifier::verify::build_fixed_cairo_circuit`]
pub fn build_fixed_cairo_subcircuit(
    verifier_config: &CairoVerifierConfig,
    proof: Proof<QM31>,
    public_claim: Vec<u32>,
    outputs: Vec<[M31; MEMORY_VALUES_LIMBS]>,
) -> Context<QM31> {
    let config = &verifier_config.proof_config;
    let components = enabled_components(&config.enabled_bits);

    let public_claim = public_claim.iter().map(|u32| M31::from(*u32)).collect_vec();
    let mut context = TraceContext::default();
    let statement = CairoStatement::<QM31>::new(
        &mut context,
        public_claim,
        outputs,
        verifier_config.program.clone(),
        components,
        verifier_config.preprocessed_root,
        verifier_config.preprocessed_trace_variant,
    );

    let proof_vars = proof.guess(&mut context);
    verify(&mut context, &proof_vars, config, &statement);
    add_dummy_outputs(&mut context);
    finalize_constants(&mut context);
    context.finalize_guessed_vars();

    context
}

/// Like [`circuit_cairo_verifier::verify::verify_fixed_cairo_circuit`]
pub fn verify_fixed_cairo_subcircuit(
    verifier_config: &CairoVerifierConfig,
    proof: Proof<QM31>,
    public_claim: Vec<u32>,
    outputs: Vec<[M31; MEMORY_VALUES_LIMBS]>,
) -> Result<Context<QM31>, String> {
    if outputs.len() != verifier_config.n_outputs {
        return Err("The proof claim does not match the expected number of outputs.".to_string());
    }
    let context = build_fixed_cairo_subcircuit(verifier_config, proof, public_claim, outputs);

    #[cfg(test)]
    context.check_vars_used();
    #[cfg(test)]
    context.circuit.check_yields();
    if !context.is_circuit_valid() {
        return Err("Verification failed".to_string());
    }
    Ok(context)
}

/// Like [`circuit_cairo_verifier::verify::build_cairo_verifier_circuit`]
pub fn build_cairo_verifier_subcircuit(verifier_config: &CairoVerifierConfig) -> Context<NoValue> {
    let config = &verifier_config.proof_config;
    let components = enabled_components::<NoValue>(&config.enabled_bits);

    let n_outputs = verifier_config.n_outputs;
    let program_len = verifier_config.program.len();
    let public_data = vec![M31::zero(); PUBLIC_DATA_LEN + n_outputs + program_len];
    let outputs = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; n_outputs];

    let mut context = Context::<NoValue>::default();
    let statement = CairoStatement::<NoValue>::new(
        &mut context,
        public_data,
        outputs,
        verifier_config.program.clone(),
        components,
        verifier_config.preprocessed_root,
        verifier_config.preprocessed_trace_variant,
    );

    let proof_vars = empty_proof(config).guess(&mut context);
    verify(&mut context, &proof_vars, config, &statement);
    add_dummy_outputs(&mut context);
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context
}

/// Like [`circuit_cairo_verifier::test::verify_cairo_with_component_set`].
pub fn verify_cairo_with_component_set(
    cairo_proof: &CairoProof<Blake2sM31MerkleHasher>,
    component_set: HashSet<&str>,
) -> Result<Context<QM31>, String> {
    let FlatClaim { component_enable_bits, component_log_sizes: _, public_data: _ } =
        cairo_proof.claim.flatten_claim();
    let components: indexmap::IndexMap<&'static str, Box<dyn CircuitEval<QM31>>> = zip_eq(
        cairo_all_components::<QM31>().into_iter(),
        &component_enable_bits,
    )
    .filter_map(|((component_name, component), &enable_bit)| {
        let component_in_set = component_set.contains(component_name);
        if component_in_set != enable_bit {
            return Some(Err(format!(
                "Proof was produced with the wrong components set: expected '{}' to be {} but it is {} in the proof.",
                component_name,
                if component_in_set { "enabled" } else { "disabled" },
                if enable_bit { "enabled" } else { "disabled" },
            )));
        }
        if enable_bit { Some(Ok((component_name, component))) } else { None }
    })
    .try_collect()?;

    let proof_config = ProofConfig::new(
        &components,
        component_enable_bits,
        cairo_proof.preprocessed_trace_variant.to_preprocessed_trace().ids().len(),
        &cairo_proof.extended_stark_proof.proof.config,
        CAIRO_INTERACTION_POW_BITS,
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
        .collect();

    let ppt_root = cairo_proof.extended_stark_proof.proof.commitments[0];
    let verifier_config = CairoVerifierConfig {
        preprocessed_root: ppt_root.into(),
        proof_config,
        program,
        n_outputs: cairo_proof.claim.public_data.public_memory.output.len(),
        preprocessed_trace_variant: cairo_proof.preprocessed_trace_variant,
    };

    verify_fixed_cairo_subcircuit(&verifier_config, proof, public_claim, outputs)
}
