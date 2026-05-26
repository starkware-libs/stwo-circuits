use circuit_common::N_RESERVED;
use circuit_verifier::statement::CircuitStatement;
use circuit_verifier::verify::CircuitConfig;
use circuits::context::{Context, U_VALUE};
use circuits::{
    blake::{HashValue, blake},
    finalize_constants::finalize_constants,
    ivalue::IValue,
    ops::Guess,
};
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use circuits_stark_verifier::{
    proof::{Proof, ProofConfig},
    verify::verify,
};
use itertools::chain;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

/// A circuit proof together with metadata needed to build the [`CircuitStatement`] against which
/// the proof will be verified. The multiverifier expects that output values of an input circuit
/// consist of 2 QM31 (usually representing an hash) + 1 QM31 equal to
/// [`circuits::context::U_VALUE`] coming from the constant finalization mechanism (see
/// [`circuits::finalize_constants`]).
pub struct MultiverifierInput<Value: IValue> {
    /// A circuit proof.
    pub proof: Proof<Value>,
    /// The preprocessed root of the circuit associated to `proof`.
    pub preprocessed_root: HashValue<QM31>,
    /// The values of the first two output wires of the circuit associated to `proof`.
    pub hashed_outputs: [QM31; 2],
}

/// Configurations shared by the circuits being verified by the multiverifier and their proofs.
/// These values are static and fixed in advance.
pub struct SharedConfig {
    pub pcs_config: PcsConfig,
    pub proof_config: ProofConfig,
    pub preprocessed_column_log_sizes: OrderedHashMap<PreProcessedColumnId, u32>,
    pub output_addresses: Vec<usize>,
}

/// Builds a circuit that verifies two circuit proofs.
///
/// For each [`MultiverifierInput`], the function reconstructs the inner
/// [`CircuitStatement`] from `shared_config` and the per-proof
/// `preprocessed_root` and `hashed_outputs`, and runs the STARK verifier.
///
/// After both proofs are verified, the preprocessed roots and the inner
/// circuits' output values are concatenated and hashed. the
/// resulting hash is written into the two reserved output variables
/// of the outer circuit. The circuit is then finalized.
///
/// Both proofs must have been produced with the same [`SharedConfig`].
pub fn build_multiverifier_circuit<Value: IValue>(
    input0: MultiverifierInput<Value>,
    input1: MultiverifierInput<Value>,
    shared_config: &SharedConfig,
) -> Context<Value> {
    let mut context = Context::new(N_RESERVED);

    let mut outer_verifier_output_preimage = vec![];
    // Verify sequentially the two proofs.
    for multiverifier_input in [input0, input1] {
        let MultiverifierInput { proof, preprocessed_root, hashed_outputs } = multiverifier_input;

        let circuit_config = CircuitConfig {
            config: shared_config.pcs_config,
            output_addresses: shared_config.output_addresses.clone(),
            preprocessed_column_log_sizes: shared_config.preprocessed_column_log_sizes.clone(),
            preprocessed_root,
        };

        let mut inner_circuit_output_values = hashed_outputs.to_vec();
        inner_circuit_output_values.push(U_VALUE);
        let statement =
            CircuitStatement::new(&mut context, &circuit_config, &inner_circuit_output_values);

        let proof_vars = proof.guess(&mut context);
        verify(&mut context, &proof_vars, &shared_config.proof_config, &statement);
        let (_, inner_circuit_output_vars) = statement.get_output_values().split_last().unwrap();
        outer_verifier_output_preimage.extend(chain!(
            [statement.preprocessed_root.0, statement.preprocessed_root.1],
            inner_circuit_output_vars.iter().copied(),
        ));
    }
    // The payload to be hashed is equal to:
    // [
    //      preprocessed_rootA.0, preprocessed_rootA.1, hashed_outputsA.0, hashed_outputsA.1
    //      preprocessed_rootB.0, preprocessed_rootB.1, hashed_outputsB.0, hashed_outputsB.1
    // ]
    // where A, B are the two circuits being verified.
    let output_hash = blake(
        &mut context,
        &outer_verifier_output_preimage,
        16 * outer_verifier_output_preimage.len(),
    );
    // Copy the resulting hash into the reserved variables
    context.output_into_reserved(&[output_hash.0, output_hash.1]);

    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context
}
