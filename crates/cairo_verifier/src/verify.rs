use std::sync::Arc;

use crate::all_components::all_components;
use crate::statement::{
    AUX_DATA_FIXED_LEN, CairoStatement, MEMORY_VALUES_LIMBS, serialize_cairo_claim,
};
use cairo_air::CairoProof;
use cairo_air::flat_claims::FlatClaim;
use circuit_common::N_RESERVED;
use circuits::blake::HashValue;
use circuits::context::{Context, FinalizedContext};
use circuits::ivalue::{IValue, NoValue};
use circuits::ops::Guess;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::proof::{Proof, ProofConfig, empty_proof};
use circuits_stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use circuits_stark_verifier::verify::verify;
use indexmap::IndexMap;
use itertools::zip_eq;
use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;

/// Logup security is defined by the `QM31` space (~124 bits) + `INTERACTION_POW_BITS` -
/// log2(number of relation terms).
/// The number of relation terms is defined as n_terms * n_relations * n_uses, where:
/// n_terms = number of terms in each relation (the size of the relation entry) < 2^7,
/// n_relations = number of different relations ids < 2^6,
/// n_uses is bounded by the characteristic of the field = 2^31.
/// E.g. assuming a 100-bit security target, the witness may contain up to
/// 1 << (24 + INTERACTION_POW_BITS) relation terms.
pub const INTERACTION_POW_BITS: u32 = 24;

/// The preprocessed roots are taken from stwo_cairo's
/// `export_circuit_cairo_verifier_preprocessed_roots()`, where the small canonical preproessed
/// trace is lifted to the lifting_log_size specified. Notice that lifting_log_size = 20 +
/// log_blowup_factor.
pub fn get_preprocessed_root(lifting_log_size: u32) -> HashValue<QM31> {
    let root = match lifting_log_size {
        21 => [
            1318760383, 1968650180, 1092022781, 246736179, 768637788, 1782650371, 1631388100,
            1492376542,
        ],
        22 => [
            2019947850, 1578675143, 1485624323, 207118193, 636087281, 1354843492, 2101876892,
            721181021,
        ],
        23 => [
            403551725, 1198969136, 1544105195, 2074510234, 916191583, 1646435042, 649872328,
            1026506463,
        ],
        _ => panic!("Unsupported lifting_log_size: {lifting_log_size}"),
    };
    root.into()
}

/// Configuration for the circuit that verifies the Cairo AIR.
///
/// Bundles everything the verifier needs that is fixed for a given Cairo program and proof
/// configuration: the STARK parameters, the program itself, and the preprocessed trace root.
pub struct CairoVerifierConfig {
    /// STARK proof configuration (component shapes, FRI parameters, PoW bits, etc.).
    pub proof_config: ProofConfig,
    /// One flag per component in the full list of components (in the order returned by
    /// `all_components()`), indicating whether it is enabled.
    pub enabled_bits: Vec<bool>,
    /// The Cairo program being verified. Each memory cell is encoded as `MEMORY_VALUES_LIMBS`
    /// nine-bit M31 limbs.
    pub program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
    /// Number of public outputs produced by the program.
    pub n_outputs: usize,
    /// Merkle root of the preprocessed (constant) trace columns.
    pub preprocessed_root: HashValue<QM31>,
    /// Which preprocessed trace variant to use (e.g. small canonical vs lifted).
    pub preprocessed_trace_variant: PreProcessedTraceVariant,
}

/// Verifies a [Proof] for a fixed [CairoVerifierConfig].
pub fn verify_fixed_cairo_circuit(
    verifier_config: &CairoVerifierConfig,
    proof: Proof<QM31>,
    serialized_aux_data: Vec<M31>,
    outputs: Vec<[M31; MEMORY_VALUES_LIMBS]>,
) -> Result<FinalizedContext<QM31>, String> {
    if outputs.len() != verifier_config.n_outputs {
        return Err("The proof claim does not match the expected number of outputs.".to_string());
    }
    let context = build_fixed_cairo_circuit(verifier_config, proof, serialized_aux_data, outputs);

    // Check the verifier circuit gates topology only in test mode.
    #[cfg(test)]
    context.check_vars_used();
    #[cfg(test)]
    context.circuit().check_yields();
    // Always validate the circuit values.
    if !context.is_circuit_valid() {
        return Err("Verification failed".to_string());
    }
    Ok(context)
}

/// Returns the entries of [`all_components`] whose corresponding bit in `enabled_bits` is set,
/// preserving the order of [`all_components`].
pub fn enabled_components<V: IValue>(
    enabled_bits: &[bool],
) -> IndexMap<&'static str, Box<dyn CircuitEval<V>>> {
    zip_eq(all_components::<V>(), enabled_bits)
        .filter_map(|((name, component), &enabled)| enabled.then_some((name, component)))
        .collect()
}

/// Builds the Cairo verifier circuit context for a fixed circuit configuration.
///
/// The context can be used for proof verification or recursive proving.
pub fn build_fixed_cairo_circuit(
    verifier_config: &CairoVerifierConfig,
    proof: Proof<QM31>,
    serialized_aux_data: Vec<M31>,
    outputs: Vec<[M31; MEMORY_VALUES_LIMBS]>,
) -> FinalizedContext<QM31> {
    let config = &verifier_config.proof_config;

    let mut context = Context::new(N_RESERVED);
    let statement = CairoStatement::<QM31>::new(
        &mut context,
        serialized_aux_data,
        outputs,
        verifier_config.program.clone(),
        verifier_config.enabled_bits.clone(),
        verifier_config.preprocessed_root,
        verifier_config.preprocessed_trace_variant,
    );

    let proof_vars = proof.guess(&mut context);
    verify(&mut context, &proof_vars, config, &statement);

    context.finalize(false)
}

/// Builds the Cairo verifier circuit topology without needing a proof.
///
/// The circuit structure is deterministic given the verifier config, so we can construct it using
/// [NoValue] and an [empty_proof].
pub fn build_cairo_verifier_circuit(
    verifier_config: &CairoVerifierConfig,
) -> FinalizedContext<NoValue> {
    let config = &verifier_config.proof_config;

    let n_outputs = verifier_config.n_outputs;
    let program_len = verifier_config.program.len();
    let n_components = verifier_config.enabled_bits.iter().filter(|b| **b).count();
    let serialized_aux_data =
        vec![M31::zero(); AUX_DATA_FIXED_LEN + n_outputs + program_len + n_components];
    let outputs = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; n_outputs];

    let mut context: Context<NoValue> = Context::new(N_RESERVED);
    let statement = CairoStatement::new(
        &mut context,
        serialized_aux_data,
        outputs,
        verifier_config.program.clone(),
        verifier_config.enabled_bits.clone(),
        verifier_config.preprocessed_root,
        verifier_config.preprocessed_trace_variant,
    );

    let proof_vars = empty_proof(config).guess(&mut context);
    verify(&mut context, &proof_vars, config, &statement);
    context.finalize(false)
}

/// Converts a [CairoProof] to a [Proof] and serialized aux data for the circuit verifier.
pub fn prepare_cairo_proof_for_circuit_verifier(
    proof: &CairoProof<Blake2sM31MerkleHasher>,
    enabled_bits: &[bool],
) -> (Proof<QM31>, Vec<M31>) {
    let CairoProof {
        claim,
        interaction_pow,
        interaction_claim,
        extended_stark_proof,
        channel_salt,
        preprocessed_trace_variant,
    } = proof;

    let flat_claim = claim.flatten_claim();
    let FlatClaim { component_enable_bits, component_log_sizes, public_data: _ } = &flat_claim;
    let claimed_sums = interaction_claim.flatten_interaction_claim();

    let components = all_components::<QM31>()
        .into_iter()
        .zip(component_enable_bits)
        .filter_map(|(component, enable)| enable.then_some(component))
        .collect::<IndexMap<_, _>>();

    let proof_config = ProofConfig::new(
        &components,
        preprocessed_trace_variant.n_columns(),
        &extended_stark_proof.proof.config,
        INTERACTION_POW_BITS,
    );

    debug_assert_eq!(component_enable_bits, enabled_bits);
    debug_assert_eq!(component_log_sizes.len(), proof_config.n_components());
    debug_assert_eq!(claimed_sums.len(), proof_config.n_components());

    let proof = proof_from_stark_proof(
        extended_stark_proof,
        &proof_config,
        claimed_sums,
        *interaction_pow,
        *channel_salt,
    );

    let proof_header = serialize_cairo_claim(&flat_claim);

    (proof, proof_header)
}
