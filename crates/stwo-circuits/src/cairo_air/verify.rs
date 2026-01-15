use crate::cairo_air::statement::CairoStatement;
use crate::circuits::context::Context;
use crate::stark_verifier::proof::{Proof, ProofConfig};
use crate::stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use cairo_air::air::{CairoClaim, CairoInteractionClaim};
use cairo_air::blake::air::InteractionClaim as BlakeInteractionClaim;
use cairo_air::builtins_air::BuiltinsInteractionClaim;
use cairo_air::opcodes_air::OpcodeInteractionClaim;
use cairo_air::pedersen::air::InteractionClaim as PedersenInteractionClaim;
use cairo_air::poseidon::air::InteractionClaim as PoseidonInteractionClaim;
use stwo::core::fields::qm31::{QM31, SecureField};
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::vcs_lifted::MerkleHasherLifted;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;

/// [cairo_air::air::CairoProof] with [ExtendedStarkProof] instead of
/// [stwo::core::proof::StarkProof].
// TODO(Gali): Move to stwo_cairo.
pub struct ExtendedCairoProof<H: MerkleHasherLifted> {
    pub claim: CairoClaim,
    pub interaction_pow: u64,
    pub interaction_claim: CairoInteractionClaim,
    pub stark_proof: ExtendedStarkProof<H>,
    /// Optional salt used in the channel initialization.
    pub channel_salt: Option<u64>,
}

/// Circuit Verifies an [ExtendedCairoProof].
// TODO(Gali): Add test.
pub fn verify_cairo(_proof: &ExtendedCairoProof<Blake2sM31MerkleHasher>) -> Context<QM31> {
    unimplemented!()
}

/// Prepares the input for the circuit verifier by converting the [ExtendedCairoProof] to a
/// [ProofConfig] and a [Proof].
pub fn proof_from_cairo_proof(
    proof: &ExtendedCairoProof<Blake2sM31MerkleHasher>,
    statement: &CairoStatement<QM31>,
) -> (ProofConfig, Proof<SecureField>) {
    let ExtendedCairoProof {
        claim,
        // TODO(Gali): Add interaction pow to the config.
        interaction_pow: _,
        interaction_claim,
        stark_proof,
        // TODO(Gali): Add channel salt to the config.
        channel_salt: _,
    } = proof;

    let trace_log_sizes = &claim.log_sizes()[1];
    let log_trace_size = trace_log_sizes.iter().max().unwrap();
    let n_preprocessed_columns = proof.stark_proof.proof.queried_values[0].len();
    let config = ProofConfig::from_statement(
        statement,
        n_preprocessed_columns,
        *log_trace_size as usize,
        &proof.stark_proof.proof.config,
    );

    // TODO(Gali): Add public claim and segment ranges to the proof.
    let component_log_sizes =
        get_component_log_sizes(trace_log_sizes, &config.trace_columns_per_component);
    let claimed_sums = get_cairo_claimed_sums(interaction_claim);

    let proof = proof_from_stark_proof(stark_proof, &config, component_log_sizes, claimed_sums);
    (config, proof)
}

/// Extracts log size per component from the trace columns log sizes and the trace columns per
/// component.
fn get_component_log_sizes(
    trace_log_sizes: &[u32],
    trace_columns_per_component: &[usize],
) -> Vec<u32> {
    let mut component_log_sizes = Vec::new();
    let mut column_index = 0;

    for trace_columns in trace_columns_per_component {
        // All columns in a component share the same log size, so we take the first one.
        component_log_sizes.push(trace_log_sizes[column_index]);
        column_index += trace_columns;
    }

    component_log_sizes
}

/// Extracts the claimed sums from a [CairoInteractionClaim].
///
/// Returns a vector of all claimed sums for the logup argument, one per component.
/// The order must match the order of components as they appear in
/// [cairo_air::air::CairoComponents].
fn get_cairo_claimed_sums(interaction_claim: &CairoInteractionClaim) -> Vec<QM31> {
    let CairoInteractionClaim {
        opcodes,
        verify_instruction,
        blake_context,
        builtins,
        pedersen_context,
        poseidon_context,
        memory_address_to_id,
        memory_id_to_value,
        range_checks,
        verify_bitwise_xor_4,
        verify_bitwise_xor_7,
        verify_bitwise_xor_8,
        verify_bitwise_xor_9,
    } = interaction_claim;
    let mut claimed_sums = Vec::new();

    // Opcodes
    let OpcodeInteractionClaim {
        add,
        add_small,
        add_ap,
        assert_eq,
        assert_eq_imm,
        assert_eq_double_deref,
        blake,
        call,
        call_rel_imm,
        generic,
        jnz,
        jnz_taken,
        jump,
        jump_double_deref,
        jump_rel,
        jump_rel_imm,
        mul,
        mul_small,
        qm31,
        ret,
    } = opcodes;
    claimed_sums.extend(add.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(add_small.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(add_ap.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(assert_eq.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(assert_eq_imm.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(assert_eq_double_deref.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(blake.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(call.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(call_rel_imm.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(generic.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(jnz.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(jnz_taken.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(jump.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(jump_double_deref.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(jump_rel.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(jump_rel_imm.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(mul.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(mul_small.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(qm31.iter().map(|c| c.claimed_sum));
    claimed_sums.extend(ret.iter().map(|c| c.claimed_sum));

    // Verify instruction
    claimed_sums.push(verify_instruction.claimed_sum);

    // Blake context (if present)
    if let Some(BlakeInteractionClaim {
        blake_g,
        triple_xor_32,
        blake_sigma,
        blake_round,
        verify_bitwise_xor_12,
    }) = &blake_context.claim
    {
        claimed_sums.push(blake_g.claimed_sum);
        claimed_sums.push(triple_xor_32.claimed_sum);
        claimed_sums.push(blake_sigma.claimed_sum);
        claimed_sums.push(blake_round.claimed_sum);
        claimed_sums.push(verify_bitwise_xor_12.claimed_sum);
    }

    // Builtins
    let BuiltinsInteractionClaim {
        range_check_128_builtin,
        range_check_96_builtin,
        bitwise_builtin,
        add_mod_builtin,
        mul_mod_builtin,
        pedersen_builtin,
        poseidon_builtin,
    } = builtins;
    if let Some(claim) = &range_check_128_builtin {
        claimed_sums.push(claim.claimed_sum);
    }
    if let Some(claim) = &range_check_96_builtin {
        claimed_sums.push(claim.claimed_sum);
    }
    if let Some(claim) = &bitwise_builtin {
        claimed_sums.push(claim.claimed_sum);
    }
    if let Some(claim) = &add_mod_builtin {
        claimed_sums.push(claim.claimed_sum);
    }
    if let Some(claim) = &mul_mod_builtin {
        claimed_sums.push(claim.claimed_sum);
    }
    if let Some(claim) = &pedersen_builtin {
        claimed_sums.push(claim.claimed_sum);
    }
    if let Some(claim) = &poseidon_builtin {
        claimed_sums.push(claim.claimed_sum);
    }

    // Pedersen context (if present)
    if let Some(PedersenInteractionClaim {
        pedersen_aggregator,
        partial_ec_mul,
        pedersen_points_table,
    }) = &pedersen_context.claim
    {
        claimed_sums.push(pedersen_aggregator.claimed_sum);
        claimed_sums.push(partial_ec_mul.claimed_sum);
        claimed_sums.push(pedersen_points_table.claimed_sum);
    }

    // Poseidon context (if present)
    if let Some(PoseidonInteractionClaim {
        poseidon_aggregator,
        poseidon_3_partial_rounds_chain,
        poseidon_full_round_chain,
        cube_252,
        poseidon_round_keys,
        range_check_252_width_27,
    }) = &poseidon_context.claim
    {
        claimed_sums.push(poseidon_aggregator.claimed_sum);
        claimed_sums.push(poseidon_3_partial_rounds_chain.claimed_sum);
        claimed_sums.push(poseidon_full_round_chain.claimed_sum);
        claimed_sums.push(cube_252.claimed_sum);
        claimed_sums.push(poseidon_round_keys.claimed_sum);
        claimed_sums.push(range_check_252_width_27.claimed_sum);
    }

    // Memory address to id
    claimed_sums.push(memory_address_to_id.claimed_sum);

    // Memory id to value
    claimed_sums.extend(memory_id_to_value.big_claimed_sums.iter().copied());
    claimed_sums.push(memory_id_to_value.small_claimed_sum);

    // Range checks
    claimed_sums.push(range_checks.rc_6.claimed_sum);
    claimed_sums.push(range_checks.rc_8.claimed_sum);
    claimed_sums.push(range_checks.rc_11.claimed_sum);
    claimed_sums.push(range_checks.rc_12.claimed_sum);
    claimed_sums.push(range_checks.rc_18.claimed_sum);
    claimed_sums.push(range_checks.rc_20.claimed_sum);
    claimed_sums.push(range_checks.rc_4_3.claimed_sum);
    claimed_sums.push(range_checks.rc_4_4.claimed_sum);
    claimed_sums.push(range_checks.rc_9_9.claimed_sum);
    claimed_sums.push(range_checks.rc_7_2_5.claimed_sum);
    claimed_sums.push(range_checks.rc_3_6_6_3.claimed_sum);
    claimed_sums.push(range_checks.rc_4_4_4_4.claimed_sum);
    claimed_sums.push(range_checks.rc_3_3_3_3_3.claimed_sum);

    // Verify bitwise xor
    claimed_sums.push(verify_bitwise_xor_4.claimed_sum);
    claimed_sums.push(verify_bitwise_xor_7.claimed_sum);
    claimed_sums.push(verify_bitwise_xor_8.claimed_sum);
    claimed_sums.push(verify_bitwise_xor_9.claimed_sum);

    claimed_sums
}
