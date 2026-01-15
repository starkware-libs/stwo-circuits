use crate::stark_verifier::proof::{Proof, ProofConfig};
use crate::stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use cairo_air::air::{CairoClaim, CairoInteractionClaim, CairoProof};
use itertools::Itertools;
use stwo::core::fields::qm31::{SecureField, QM31};
use stwo::core::pcs::quotients::CommitmentSchemeProofAux;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;

/// Constructs [Proof] with the values from the given Cairo proof ([CairoProof]).
///
/// This function converts a `CairoProof` into the circuit verifier's `Proof` representation
/// by extracting the component log sizes and claimed sums from the Cairo claim and interaction
/// claim, then delegating to `proof_from_stark_proof`.
///
/// # Arguments
/// * `cairo_proof` - The Cairo proof to convert
/// * `aux` - The auxiliary proof data (query locations, Merkle decommitment aux, FRI aux) that is
///   required for constructing evaluation domain samples and authentication paths.
/// * `config` - The proof configuration describing the AIR structure
///
/// # Note
/// The auxiliary data (`CommitmentSchemeProofAux`) is generated during proving and contains:
/// - `unsorted_query_locations`: Query indices in sampling order
/// - `trace_decommitment`: Merkle decommitment auxiliary data for each trace
/// - `fri`: FRI auxiliary data for each layer
pub fn proof_from_cairo_proof(
    cairo_proof: &CairoProof<Blake2sM31MerkleHasher>,
    aux: &CommitmentSchemeProofAux<Blake2sM31MerkleHasher>,
    config: &ProofConfig,
) -> Proof<SecureField> {
    let component_log_sizes = get_component_log_sizes(&cairo_proof.claim);
    let claimed_sums = get_claimed_sums(&cairo_proof.interaction_claim);

    let extended_proof =
        ExtendedStarkProof { proof: cairo_proof.stark_proof.clone(), aux: aux.clone() };

    proof_from_stark_proof(&extended_proof, config, component_log_sizes, claimed_sums)
}

/// Extracts the component log sizes from a [CairoClaim].
///
/// Returns a flat vector of all component log sizes from all sub-claims.
fn get_component_log_sizes(claim: &CairoClaim) -> Vec<u32> {
    let log_sizes = claim.log_sizes();
    // Flatten the TreeVec<Vec<u32>> into a single Vec<u32>
    // Skip the preprocessed trace (index 0) as it's not part of component log sizes
    log_sizes.iter().skip(1).flatten().copied().collect_vec()
}

/// Extracts the claimed sums from a [CairoInteractionClaim].
///
/// Returns a vector of all claimed sums for the logup argument, one per component.
/// The order must match the order of components as they appear in `CairoComponents`.
fn get_claimed_sums(interaction_claim: &CairoInteractionClaim) -> Vec<QM31> {
    let mut claimed_sums = Vec::new();

    // Opcodes - collect from all opcode types' vectors
    claimed_sums.extend(interaction_claim.opcodes.add.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.add_small.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.add_ap.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.assert_eq.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.assert_eq_imm.iter().map(|c| c.claimed_sum));
    claimed_sums
        .extend(interaction_claim.opcodes.assert_eq_double_deref.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.blake.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.call.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.call_rel_imm.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.generic.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.jnz.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.jnz_taken.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.jump.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.jump_double_deref.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.jump_rel.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.jump_rel_imm.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.mul.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.mul_small.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.qm31.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(interaction_claim.opcodes.ret.iter().map(|c| c.claimed_sum));

    // Verify instruction
    claimed_sums.push(interaction_claim.verify_instruction.claimed_sum);

    // Blake context (if present)
    if let Some(claim) = &interaction_claim.blake_context.claim {
        claimed_sums.push(claim.blake_g.claimed_sum);
        claimed_sums.push(claim.triple_xor_32.claimed_sum);
    }

    // Builtins
    if let Some(claim) = &interaction_claim.builtins.range_check_128_builtin {
        claimed_sums.push(claim.claimed_sum);
    }
    if let Some(claim) = &interaction_claim.builtins.range_check_96_builtin {
        claimed_sums.push(claim.claimed_sum);
    }
    if let Some(claim) = &interaction_claim.builtins.bitwise_builtin {
        claimed_sums.push(claim.claimed_sum);
    }
    if let Some(claim) = &interaction_claim.builtins.add_mod_builtin {
        claimed_sums.push(claim.claimed_sum);
    }
    if let Some(claim) = &interaction_claim.builtins.mul_mod_builtin {
        claimed_sums.push(claim.claimed_sum);
    }
    if let Some(claim) = &interaction_claim.builtins.pedersen_builtin {
        claimed_sums.push(claim.claimed_sum);
    }
    if let Some(claim) = &interaction_claim.builtins.poseidon_builtin {
        claimed_sums.push(claim.claimed_sum);
    }

    // Pedersen context (if present)
    if let Some(claim) = &interaction_claim.pedersen_context.claim {
        claimed_sums.push(claim.pedersen_points_table.claimed_sum);
    }

    // Poseidon context (if present)
    if let Some(claim) = &interaction_claim.poseidon_context.claim {
        claimed_sums.push(claim.poseidon_aggregator.claimed_sum);
        claimed_sums.push(claim.poseidon_3_partial_rounds_chain.claimed_sum);
        claimed_sums.push(claim.poseidon_full_round_chain.claimed_sum);
        claimed_sums.push(claim.cube_252.claimed_sum);
        claimed_sums.push(claim.poseidon_round_keys.claimed_sum);
        claimed_sums.push(claim.range_check_252_width_27.claimed_sum);
    }

    // Memory address to id
    claimed_sums.push(interaction_claim.memory_address_to_id.claimed_sum);

    // Memory id to value (big + small)
    claimed_sums.extend(interaction_claim.memory_id_to_value.big_claimed_sums.iter().copied());
    claimed_sums.push(interaction_claim.memory_id_to_value.small_claimed_sum);

    // Range checks
    claimed_sums.push(interaction_claim.range_checks.rc_6.claimed_sum);
    claimed_sums.push(interaction_claim.range_checks.rc_8.claimed_sum);
    claimed_sums.push(interaction_claim.range_checks.rc_11.claimed_sum);
    claimed_sums.push(interaction_claim.range_checks.rc_12.claimed_sum);
    claimed_sums.push(interaction_claim.range_checks.rc_18.claimed_sum);
    claimed_sums.push(interaction_claim.range_checks.rc_20.claimed_sum);
    claimed_sums.push(interaction_claim.range_checks.rc_4_3.claimed_sum);
    claimed_sums.push(interaction_claim.range_checks.rc_4_4.claimed_sum);
    claimed_sums.push(interaction_claim.range_checks.rc_9_9.claimed_sum);
    claimed_sums.push(interaction_claim.range_checks.rc_7_2_5.claimed_sum);
    claimed_sums.push(interaction_claim.range_checks.rc_3_6_6_3.claimed_sum);
    claimed_sums.push(interaction_claim.range_checks.rc_4_4_4_4.claimed_sum);
    claimed_sums.push(interaction_claim.range_checks.rc_3_3_3_3_3.claimed_sum);

    // Verify bitwise xor components
    claimed_sums.push(interaction_claim.verify_bitwise_xor_4.claimed_sum);
    claimed_sums.push(interaction_claim.verify_bitwise_xor_7.claimed_sum);
    claimed_sums.push(interaction_claim.verify_bitwise_xor_8.claimed_sum);
    claimed_sums.push(interaction_claim.verify_bitwise_xor_9.claimed_sum);

    claimed_sums
}

// TODO(Gali): Add test.
pub fn verify_cairo(
    cairo_proof: &CairoProof<Blake2sM31MerkleHasher>,
    aux: &CommitmentSchemeProofAux<Blake2sM31MerkleHasher>,
    config: &ProofConfig,
) {
    let _proof = proof_from_cairo_proof(cairo_proof, aux, config);
    // TODO(Gali): Implement verification logic using the converted proof.
}
