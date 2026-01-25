use crate::cairo_air::statement::CairoStatement;
use crate::cairo_air::statement::PUBLIC_DATA_LEN;
use crate::circuits::context::{Context, TraceContext};
use crate::circuits::ops::Guess;
use crate::stark_verifier::proof::{Claim, Proof, ProofConfig};
use crate::stark_verifier::proof_from_stark_proof::{
    pack_component_log_sizes, pack_enable_bits, proof_from_stark_proof,
};
use crate::stark_verifier::verify::verify;
use cairo_air::air::ExtendedCairoProof;
use cairo_air::flat_claims::{flatten_interaction_claim, FlatClaim};
use num_traits::Zero;
use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;
use stwo::core::fields::qm31::{SecureField, QM31};
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;

/// Circuit Verifies an [ExtendedCairoProof].
// TODO(Gali): Add test.
pub fn verify_cairo(proof: &ExtendedCairoProof<Blake2sM31MerkleHasher>) -> Context<QM31> {
    let mut context = TraceContext::default();
    let packed_claim =
        vec![QM31::zero(); PUBLIC_DATA_LEN.div_ceil(SECURE_EXTENSION_DEGREE)].guess(&mut context);
    let statement = CairoStatement::<QM31>::new(&mut context, packed_claim, PUBLIC_DATA_LEN);

    let (config, proof) = proof_from_cairo_proof(proof, &statement);

    let proof_vars = proof.guess(&mut context);

    verify(&mut context, &proof_vars, &config, &statement);

    context
}

/// Prepares the input for the circuit verifier by converting the [ExtendedCairoProof] to a
/// [ProofConfig] and a [Proof].
pub fn proof_from_cairo_proof(
    proof: &ExtendedCairoProof<Blake2sM31MerkleHasher>,
    statement: &CairoStatement<QM31>,
) -> (ProofConfig, Proof<SecureField>) {
    let ExtendedCairoProof {
        claim,
        interaction_pow,
        interaction_claim,
        extended_stark_proof,
        // TODO(Gali): Add channel salt to the config.
        channel_salt: _,
        preprocessed_trace_variant: _,
    } = proof;

    let FlatClaim { component_enable_bits, component_log_sizes, public_data:_ } =
        FlatClaim::from_cairo_claim(claim);
    let component_claimed_sums = flatten_interaction_claim(interaction_claim);

    let log_trace_size = component_log_sizes.iter().max().unwrap();
    let config = ProofConfig::from_statement(
        statement,
        *log_trace_size as usize,
        &proof.extended_stark_proof.proof.config,
    );

    let claim = Claim {
        packed_enable_bits: pack_enable_bits(&component_enable_bits),
        packed_component_log_sizes: pack_component_log_sizes(&component_log_sizes),
        claimed_sums: component_claimed_sums,
    };

    let proof = proof_from_stark_proof(extended_stark_proof, &config, claim, *interaction_pow);
    (config, proof)
}
