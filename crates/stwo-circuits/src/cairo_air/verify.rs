use crate::cairo_air::statement::CairoStatement;
use crate::cairo_air::statement::MEMORY_VALUES_LIMBS;
use crate::cairo_air::statement::PUB_MEMORY_VALUE_LEN;
use crate::cairo_air::statement::PUBLIC_DATA_LEN;
use crate::circuits::context::{Context, TraceContext};
use crate::circuits::ops::Guess;
use crate::stark_verifier::proof::{Claim, Proof, ProofConfig};
use crate::stark_verifier::proof_from_stark_proof::{
    pack_component_log_sizes, pack_enable_bits, proof_from_stark_proof,
};
use crate::stark_verifier::verify::verify;
use cairo_air::CairoProof;
use cairo_air::flat_claims::FlatClaim;
use cairo_air::flat_claims::flatten_interaction_claim;
use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::{QM31, SecureField};
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;

/// Circuit Verifies a [CairoProof].
// TODO(Gali): Add test.
pub fn verify_cairo(proof: &CairoProof<Blake2sM31MerkleHasher>) -> Context<QM31> {
    let mut context = TraceContext::default();

    let output_len = 1;
    let program_len = 128;
    let flat_claim =
        vec![M31::zero(); PUBLIC_DATA_LEN + output_len * PUB_MEMORY_VALUE_LEN + program_len];
    let program = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; program_len];
    let outputs = vec![[M31::zero(); MEMORY_VALUES_LIMBS]; output_len];
    let statement = CairoStatement::<QM31>::new(&mut context, flat_claim, outputs, program);

    let (config, proof) = proof_from_cairo_proof(proof, &statement);

    let proof_vars = proof.guess(&mut context);

    verify(&mut context, &proof_vars, &config, &statement);

    context
}

/// Prepares the input for the circuit verifier by converting the [CairoProof] to a
/// [ProofConfig] and a [Proof].
pub fn proof_from_cairo_proof(
    proof: &CairoProof<Blake2sM31MerkleHasher>,
    statement: &CairoStatement<QM31>,
) -> (ProofConfig, Proof<SecureField>) {
    let CairoProof {
        claim,
        interaction_pow,
        interaction_claim,
        extended_stark_proof,
        channel_salt,
        preprocessed_trace_variant: _,
    } = proof;

    let FlatClaim { component_enable_bits, component_log_sizes, public_data: _ } =
        FlatClaim::from_cairo_claim(claim);

    // TODO(ilya): Create the statment based on the public_claim.

    let log_trace_size = component_log_sizes.iter().max().unwrap();
    let config = ProofConfig::from_statement(
        statement,
        *log_trace_size as usize,
        &proof.extended_stark_proof.proof.config,
        24,
    );

    let component_claimed_sums = flatten_interaction_claim(interaction_claim);
    let claim = Claim {
        packed_enable_bits: pack_enable_bits(&component_enable_bits),
        packed_component_log_sizes: pack_component_log_sizes(&component_log_sizes),
        claimed_sums: component_claimed_sums,
    };

    let proof = proof_from_stark_proof(
        extended_stark_proof,
        &config,
        claim,
        *interaction_pow,
        *channel_salt,
    );
    (config, proof)
}
