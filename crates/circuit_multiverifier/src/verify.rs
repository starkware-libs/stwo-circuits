use circuit_common::N_RESERVED;
use circuit_verifier::statement::CircuitStatement;
use circuit_verifier::verify::CircuitConfig;
use circuits::context::{Context, FinalizedContext, Var};
use circuits::wrappers::U32Wrapper;
use circuits::{
    blake::{HashValue, blake2s_u32s, unpack_qm31s_to_u32_words},
    ivalue::IValue,
    ops::Guess,
};
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use circuits_stark_verifier::{
    proof::{Proof, ProofConfig},
    verify::verify,
};
use itertools::{Itertools, chain};
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

#[cfg(test)]
#[path = "verify_test.rs"]
mod verify_test;

/// A circuit proof together with metadata needed to build the [`CircuitStatement`] against which
/// the proof will be verified. The multiverifier expects that output values of an input circuit
/// consist of `N_RESERVED` QM31 (the unreduced output hash) + 1 QM31 equal to
/// [`circuits::context::U_VALUE`] coming from the constant finalization mechanism (see
/// [`circuits::finalize_constants`]).
pub struct MultiverifierInput<Value: IValue> {
    /// A circuit proof.
    pub proof: Proof<Value>,
    /// The preprocessed root of the circuit associated to `proof`.
    pub preprocessed_root: HashValue<QM31>,
    /// The output values of the circuit (excluding the value of the `u` wire at address
    /// [`circuits::context::U_VAR_IDX`]). The multiverifier only supports verification of circuits
    /// whose output is the unreduced Blake2s digest (`N_RESERVED` words).
    // TODO(ilya): consider changing this to `[u32; N_RESERVED]`.
    pub output_values: [QM31; N_RESERVED],
}

/// Configurations shared by the circuits being verified by the multiverifier and their proofs.
/// These values are static and fixed in advance.
pub struct SharedConfig {
    pub pcs_config: PcsConfig,
    pub proof_config: ProofConfig,
    pub preprocessed_column_log_sizes: OrderedHashMap<PreProcessedColumnId, u32>,
}

/// Verifies one child of a fanning node and returns that child's contribution to the fold-hash
/// preimage.
///
/// A fanning node verifies `k` child proofs in a single circuit and folds them by hashing each
/// child's `[preprocessed_root words, output words]` (children left-to-right) with `blake2s_u32s`.
/// The verification of a single child — building the child's statement, guessing its proof, running
/// the STARK verifier, and extracting the preprocessed-root + output words — is entirely
/// child-specific; only the fold-hash and its ordering are shared. This trait factors out that
/// per-child body so [`build_fanning_circuit`] can fold children of ANY statement type (the generic
/// [`CircuitStatement`] circuit proofs the multiverifier already handled, or an application AIR such
/// as gate_air) while keeping ONE shared, byte-identical fold-hash contract.
///
/// The returned `Vec` is the exact per-child preimage chunk that gets concatenated into the shared
/// hash, in the fixed order `[preprocessed_root (8 words), output words...]`. This ordering is the
/// single byte-identity contract with the out-of-circuit unpacker
/// (`recursive_aggregate::prove_root_verification`); every implementor MUST emit the eight
/// preprocessed-root words first, followed by the output words, so the two stay identical.
pub trait ChildVerifier<Value: IValue> {
    /// Per-child input: whatever the concrete verifier needs to reconstruct the child's statement
    /// and guess its proof.
    type Input;

    /// Verifies `input`'s proof against its reconstructed statement and returns the per-child
    /// preimage chunk `[preprocessed_root words, output words]`.
    fn verify_child(&self, context: &mut Context<Value>, input: Self::Input)
    -> Vec<U32Wrapper<Var>>;
}

/// The child verifier for generic [`CircuitStatement`] circuit proofs — the original multiverifier
/// per-child body. Each child is a [`MultiverifierInput`] verified against a [`CircuitStatement`]
/// reconstructed from the [`SharedConfig`].
pub struct CircuitChildVerifier<'a> {
    pub shared_config: &'a SharedConfig,
}

impl<'a, Value: IValue> ChildVerifier<Value> for CircuitChildVerifier<'a> {
    type Input = MultiverifierInput<Value>;

    fn verify_child(
        &self,
        context: &mut Context<Value>,
        input: Self::Input,
    ) -> Vec<U32Wrapper<Var>> {
        let MultiverifierInput { proof, preprocessed_root, output_values } = input;

        let circuit_config = CircuitConfig {
            config: self.shared_config.pcs_config,
            n_outputs: N_RESERVED,
            preprocessed_column_log_sizes: self.shared_config.preprocessed_column_log_sizes.clone(),
            preprocessed_root,
        };
        let statement = CircuitStatement::new(context, &circuit_config, &output_values);
        let proof_vars = proof.guess(context);

        verify(context, &proof_vars, &self.shared_config.proof_config, &statement);
        let preprocessed_root = statement.preprocessed_root.clone();
        let output_words =
            unpack_qm31s_to_u32_words(context, statement.get_output_values().iter().copied());
        chain!(preprocessed_root.into_iter(), output_words).collect()
    }
}

/// Builds a circuit that verifies `k` circuit proofs (a `k`-to-1 fold node).
///
/// For each [`MultiverifierInput`], the function reconstructs the inner
/// [`CircuitStatement`] from `shared_config` and the per-proof
/// `preprocessed_root` and `outputs`, and runs the STARK verifier.
///
/// After all `k` proofs are verified, the preprocessed roots and the inner
/// circuits' output values are concatenated (in the fixed left-to-right order of `inputs`) and
/// hashed. The resulting unreduced Blake2s digest is written into the `N_RESERVED` reserved output
/// variables of the outer circuit. The circuit is then finalized.
///
/// All proofs must have been produced with the same [`SharedConfig`]. `inputs` must be non-empty;
/// its length is the fold arity `k` (see `FOLD_ARITY` in `recursive_aggregate`).
///
/// The child ordering of the hash preimage is the single byte-identity contract shared with the
/// out-of-circuit unpacker (`recursive_aggregate::prove_root_verification`): both concatenate the
/// children left-to-right, and per child emit `[preprocessed_root words, output words]`. Any
/// arity change must keep this ordering identical in both places.
///
/// This is a thin wrapper over the generic [`build_fanning_circuit`] with the
/// [`CircuitChildVerifier`]; both share the one fold-hash implementation.
pub fn build_multiverifier_circuit<Value: IValue>(
    inputs: Vec<MultiverifierInput<Value>>,
    shared_config: &SharedConfig,
) -> FinalizedContext<Value> {
    build_fanning_circuit(inputs, &CircuitChildVerifier { shared_config })
}

/// Builds a `k`-to-1 fold node that verifies `k` children of ANY [`ChildVerifier`] type and folds
/// them with the shared Blake2s fold-hash.
///
/// This is the DRY core of the multiverifier: `verifier.verify_child` supplies the (child-specific)
/// verify + per-child preimage chunk, and everything below — the left-to-right concatenation, the
/// `blake2s_u32s`, writing the eight-word digest into the reserved outputs, and finalization — is
/// SHARED and byte-identical across every child type. Passing [`CircuitChildVerifier`] reproduces
/// the original [`build_multiverifier_circuit`] byte-for-byte.
///
/// `inputs` must be non-empty; its length is the fold arity `k`. The per-child ordering
/// (`[preprocessed_root words, output words]`, children left-to-right) is the byte-identity contract
/// with the out-of-circuit unpacker; see [`ChildVerifier`].
pub fn build_fanning_circuit<Value: IValue, V: ChildVerifier<Value>>(
    inputs: Vec<V::Input>,
    verifier: &V,
) -> FinalizedContext<Value> {
    assert!(!inputs.is_empty(), "fanning node needs at least one input");
    let mut context = Context::new(N_RESERVED);

    let mut outer_verifier_output_preimage = vec![];
    // Verify the `k` proofs sequentially, in the fixed left-to-right order of `inputs`.
    for input in inputs {
        outer_verifier_output_preimage.extend(verifier.verify_child(&mut context, input));
    }
    // The payload to be hashed is, for each of the `k` verified circuits (left to right), the eight
    // 32-bit words of its preprocessed root followed by its output values unpacked into u32 words:
    // [
    //      preprocessed_root_0 (8 words), outputs_0 words...,
    //      preprocessed_root_1 (8 words), outputs_1 words...,
    //      ... (k children total, in the order of `inputs`)
    // ].
    let n_bytes = 4 * outer_verifier_output_preimage.len();
    let output_hash = blake2s_u32s(&mut context, outer_verifier_output_preimage, n_bytes);
    // Copy the unreduced digest words into the reserved variables.
    context.set_outputs(&output_hash.iter().map(|word| *word.get()).collect_vec());

    let context = context.finalize(false);
    #[cfg(test)]
    context.circuit().check_yields();
    context
}
