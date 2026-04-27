//! Top-level Cairo verifier input.
//!
//! Mirrors the Cairo verifier's `main(proof: CircuitProof, config: CircuitVerifierConfig)`
//! signature. Field order MUST match the Cairo `CircuitProof` struct in
//! `stwo-cairo/stwo_cairo_verifier/crates/circuit_air/src/lib.cairo`.
//!
//! The serializer is asymmetric on `queried_values`: the prover stores them in tree-major
//! order, the Cairo verifier reads them sorted by column size and transposed (see
//! `cairo_air::utils::sort_and_transpose_queried_values`). The conversion in
//! [`prepare_cairo_verifier_input`] applies the sort before the derive emits felts.

use cairo_air::utils::sort_and_transpose_queried_values;
use circuit_verifier::circuit_proof::CircuitProof as CircuitProverOutput;
use starknet_ff::FieldElement;
use stwo::core::ColumnVec;
use stwo::core::pcs::quotients::CommitmentSchemeProof;
use stwo::core::proof::StarkProof;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::vcs_lifted::merkle_hasher::MerkleHasherLifted;
use stwo_cairo_serialize::{CairoDeserialize, CairoSerialize};

use crate::claim::{CairoCircuitClaim, CairoCircuitInteractionClaim};

/// Owned mirror of the Cairo `CircuitProof` struct, with `queried_values` already sorted
/// and transposed into the layout the Cairo verifier expects.
///
/// Symmetric `CairoSerialize`/`CairoDeserialize` derive — both directions read fields in
/// declaration order, so this round-trips cleanly.
// Note: cannot derive `PartialEq`/`Eq` because `FriProof`/`MerkleDecommitmentLifted` do
// not implement them. Roundtrip tests compare serialized felts instead.
#[derive(Clone, Debug, CairoSerialize, CairoDeserialize)]
pub struct CairoCircuitProof<H: MerkleHasherLifted<Hash = Blake2sHash>> {
    pub claim: CairoCircuitClaim,
    pub interaction_pow: u64,
    pub interaction_claim: CairoCircuitInteractionClaim,
    pub stark_proof: CairoStarkProofForCircuit<H>,
    pub channel_salt: u32,
}

/// Owned counterpart of `CommitmentSchemeProof` with `queried_values` already in the
/// 2D sorted-and-transposed layout (one `Vec<BaseField>` per tree, concatenated across
/// queries) that the Cairo verifier deserializes.
#[derive(Clone, Debug, CairoSerialize, CairoDeserialize)]
pub struct CairoStarkProofForCircuit<H: MerkleHasherLifted<Hash = Blake2sHash>> {
    pub config: stwo::core::pcs::PcsConfig,
    pub commitments: Vec<Blake2sHash>,
    pub sampled_values: Vec<ColumnVec<Vec<stwo::core::fields::qm31::QM31>>>,
    pub decommitments: Vec<stwo::core::vcs_lifted::verifier::MerkleDecommitmentLifted<H>>,
    /// Sorted+transposed queried values (per tree).
    pub queried_values: Vec<Vec<stwo::core::fields::m31::M31>>,
    pub proof_of_work: u64,
    pub fri_proof: stwo::core::fri::FriProof<H>,
}

impl<H: MerkleHasherLifted<Hash = Blake2sHash>> CairoStarkProofForCircuit<H> {
    /// Builds the Cairo-ready stark proof from a Rust `StarkProof` plus per-tree column
    /// log sizes for the [trace, interaction] trees (used to sort `queried_values`).
    pub fn from_stark_proof(
        proof: &StarkProof<H>,
        trace_and_interaction_trace_log_sizes: &[&[u32]; 2],
    ) -> Self {
        let CommitmentSchemeProof {
            config,
            commitments,
            sampled_values,
            decommitments,
            queried_values,
            proof_of_work,
            fri_proof,
        } = &proof.0;

        let sorted = sort_and_transpose_queried_values(
            queried_values,
            trace_and_interaction_trace_log_sizes.to_vec(),
        );

        Self {
            config: *config,
            commitments: (**commitments).clone(),
            sampled_values: (**sampled_values).clone(),
            decommitments: (**decommitments).clone(),
            queried_values: (*sorted).clone(),
            proof_of_work: *proof_of_work,
            fri_proof: fri_proof.clone(),
        }
    }
}

impl<H: MerkleHasherLifted<Hash = Blake2sHash>> CairoCircuitProof<H> {
    /// Builds from the live circuit prover output. Errors if the prover failed.
    pub fn from_prover_output(
        prover_output: &CircuitProverOutput<H>,
    ) -> Result<Self, &'static str> {
        let stark_proof = &prover_output.stark_proof.proof;

        let log_sizes = prover_output.claim.column_log_sizes_per_tree();
        let stark_proof = CairoStarkProofForCircuit::<H>::from_stark_proof(
            stark_proof,
            &[log_sizes[0].as_slice(), log_sizes[1].as_slice()],
        );

        Ok(Self {
            claim: CairoCircuitClaim::from(&prover_output.claim),
            interaction_pow: prover_output.interaction_pow_nonce,
            interaction_claim: CairoCircuitInteractionClaim::from(&prover_output.interaction_claim),
            stark_proof,
            channel_salt: prover_output.channel_salt,
        })
    }
}

/// Builds the felt252 input stream for the Cairo circuit verifier from a live prover
/// output. Only the proof is serialized — verifier-config constants (output addresses,
/// `n_blake_gates`, preprocessed root, `lifting_log_size`, preprocessed column log sizes)
/// are hardcoded inside the Cairo verifier binary for a specific circuit topology.
pub fn prepare_cairo_verifier_input<H: MerkleHasherLifted<Hash = Blake2sHash>>(
    prover_output: &CircuitProverOutput<H>,
) -> Result<Vec<FieldElement>, &'static str> {
    let proof = CairoCircuitProof::<H>::from_prover_output(prover_output)?;
    let mut felts = Vec::new();
    CairoSerialize::serialize(&proof, &mut felts);
    Ok(felts)
}
