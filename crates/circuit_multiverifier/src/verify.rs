#![allow(unused)]
// Want:
// - two circuit proof test vectors.
// - define a single pcs config which is used by both circuit proofs.
// - both proofs are of the cairo verifier.

use circuit_verifier::{
    statement::{INTERACTION_POW_BITS, all_circuit_components},
    verify::{CircuitConfig, CircuitPublicData},
};
use circuits::{
    blake::{HashValue, blake, blake_qm31},
    context::Var,
    finalize_constants::finalize_constants,
    ivalue::IValue,
    ops::{Guess, output},
};
use circuits::{
    context::Context,
    eval,
    ivalue::qm31_from_u32s,
    ops::{eq, sub},
    wrappers::M31Wrapper,
};
use circuits_stark_verifier::{
    merkle::{AuthPath, verify_merkle_path},
    proof::{Proof, ProofConfig},
    verify::verify,
};
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::statement::SubCircuitStatement;

#[cfg(test)]
#[path = "verify_test.rs"]
mod verify_test;

#[cfg(test)]
#[path = "multi_fibonacci_test.rs"]
mod multi_fibonacci_test;

// TODO: remove this struct
pub struct Input<Value: IValue> {
    pub(crate) proof: Proof<Value>,
    pub(crate) circuit_public_data: CircuitPublicData,
    pub(crate) config: CircuitConfig,
    /// `true` if this proof is a proof of the multiverifier circuit itself
    /// (variant B in the recursion); `false` for a leaf circuit such as the
    /// Fibonacci/`circuit_verifier` proof (variant A). Encoded as the `bit`
    /// field in [`Metadata`] and used to:
    ///  - select the leaf position (and Merkle path direction) in the `metadata_root` commitment,
    ///  - select between `0` and `metadata_root` as the expected output value at slots 0,1 of the
    ///    proof being verified.
    pub(crate) is_multiverifier: bool,
    /// The other leaf in the 2-leaf metadata Merkle tree — the sibling
    /// against which this proof's metadata hash is paired to recompute the
    /// root. The prover provides it as a witness; the Merkle equation
    /// constrains its value.
    // TODO: remove from this struct, compute it on the fly.
    pub(crate) other_hash: HashValue<QM31>,
}

// TODO: find better name.
pub struct Metadata<T> {
    /// `0` for a leaf-circuit descriptor (variant A), `1` for the
    /// multiverifier descriptor (variant B). Hashed into the descriptor so
    /// that `bit` is bound to the leaf type via the Merkle path.
    pub(crate) bit: M31Wrapper<T>,
    pub(crate) n_blake_gates_pow_two: M31Wrapper<T>,
    pub(crate) output_addresses: Vec<M31Wrapper<T>>,
    pub(crate) preprocessed_root: HashValue<T>,
}

impl Metadata<QM31> {
    pub fn serialize_to_qm31(self) -> Vec<QM31> {
        let Metadata { bit, n_blake_gates_pow_two, output_addresses, preprocessed_root } = self;

        let mut res = vec![*bit.get(), *n_blake_gates_pow_two.get()];
        // Add domain separation for length.
        res.extend(output_addresses.iter().map(|x| *x.get()));
        res.extend([preprocessed_root.0, preprocessed_root.1]);
        res
    }
}

impl<Value: IValue> Metadata<Value> {
    pub fn from_config(config: &CircuitConfig, is_multiverifier: bool) -> Self {
        let bit = M31Wrapper::new_unsafe(Value::from_qm31(QM31::from(is_multiverifier as u32)));
        let n_blake_gates_pow_two = M31Wrapper::new_unsafe(Value::from_qm31(QM31::from(
            config.n_blake_gates.next_power_of_two(),
        )));

        let output_addresses = config
            .output_addresses
            .iter()
            .map(|x| M31Wrapper::new_unsafe(Value::from_qm31(QM31::from(*x))))
            .collect();

        Metadata {
            bit,
            n_blake_gates_pow_two,
            output_addresses,
            preprocessed_root: HashValue(
                Value::from_qm31(config.preprocessed_root.0),
                Value::from_qm31(config.preprocessed_root.1),
            ),
        }
    }
}

impl<Value: IValue> Guess<Value> for Metadata<Value> {
    type Target = Metadata<Var>;
    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Metadata {
            bit: self.bit.guess(context),
            n_blake_gates_pow_two: self.n_blake_gates_pow_two.guess(context),
            output_addresses: self.output_addresses.guess(context),
            preprocessed_root: self.preprocessed_root.guess(context),
        }
    }
}

impl Metadata<Var> {
    fn serialize_to_qm31(&self) -> Vec<Var> {
        let Metadata { bit, n_blake_gates_pow_two, output_addresses, preprocessed_root } = self;

        let mut res = vec![*bit.get(), *n_blake_gates_pow_two.get()];
        // TODO: Add domain separation for length.
        res.extend(output_addresses.iter().map(|x| x.get()).copied());
        res.extend([preprocessed_root.0, preprocessed_root.1]);
        res
    }

    fn hash<Value: IValue>(&self, context: &mut Context<Value>) -> HashValue<Var> {
        let qm31s = self.serialize_to_qm31();
        blake(context, &qm31s, 16 * qm31s.len())
    }
}

/// Computes the metadata Merkle root over the two valid descriptors.
/// `m1` (the leaf at index 0) should be the leaf-circuit descriptor with
/// `bit = 0`; `m2` (the leaf at index 1) the multiverifier descriptor with
/// `bit = 1`.
pub fn merkleize_metadata(m1: Metadata<QM31>, m2: Metadata<QM31>) -> HashValue<QM31> {
    let m1_qm31s = m1.serialize_to_qm31();
    let m2_qm31s = m2.serialize_to_qm31();
    let hash_1 = blake_qm31(&m1_qm31s, 16 * m1_qm31s.len());
    let hash_2 = blake_qm31(&m2_qm31s, 16 * m2_qm31s.len());
    blake_qm31(&[hash_1.0, hash_1.1, hash_2.0, hash_2.1], 64)
}

const N_OUTPUTS: usize = 5;

/// Contains all the parameters that are fixed for the proofs being validated.
pub struct SubCircuitConfig {
    pub pcs_config: PcsConfig,
    pub n_outputs: usize,
    pub preprocessed_column_ids: Vec<PreProcessedColumnId>,
}

impl SubCircuitConfig {
    pub fn to_proof_config(&self) -> ProofConfig {
        let all_components = all_circuit_components::<QM31>();
        ProofConfig::from_components(
            &all_components,
            vec![true; all_components.len()],
            self.preprocessed_column_ids.len(),
            &self.pcs_config,
            INTERACTION_POW_BITS,
        )
    }
}

// pub fn circuit_verify_two_proofs(
//     p1: Input<QM31>,
//     p2: Input<QM31>,
//     subcircuit_config: SubCircuitConfig,
//     metadata_root: HashValue<QM31>,
// ) {
//     // build multiverifier config.
//     // validate the two configs against the multiverifier one.
//     for circuit_config in [p1.config, p2.config] {
//         //validate config against the multi verifier config.
//     }
//    // assert_eq!(p1.config.preprocessed_column_ids, p2.config.preprocessed_column_ids);
//    // assert_eq!(p1.config.output_addresses.len(), N_OUTPUTS);
// //    assert_eq!(p2.config.output_addresses.len(), N_OUTPUTS);

//     let all_components = all_circuit_components::<QM31>();
//     let proof_config = ProofConfig::from_components(
//         &all_components,
//         vec![true; all_components.len()],
//         subcircuit_config.preprocessed_column_ids.len(),
//         &subcircuit_config.pcs_config,
//         INTERACTION_POW_BITS,
//     );
// }
pub fn build_multiverifier_circuit<Value: IValue>(
    p1: Input<Value>,
    p2: Input<Value>,
    // proof config assumed to be the same for p1 and p2.
    subcircuit_config: SubCircuitConfig,
    metadata_root: HashValue<QM31>,
    // TODO: return a result.
) -> Context<Value> {
    // ProofConfig that is shared between the two proofs.
    let proof_config = subcircuit_config.to_proof_config();
    let mut context: Context<Value> = Context::default();
    let metadata_root =
        HashValue(Value::from_qm31(metadata_root.0), Value::from_qm31(metadata_root.1));
    let metadata_root_var = metadata_root.guess(&mut context);
    output(&mut context, metadata_root_var.0);
    output(&mut context, metadata_root_var.1);

    let mut inner_outputs = vec![];
    for input in [p1, p2] {
        let Input { proof, circuit_public_data, config, is_multiverifier, other_hash } = input;
        let metadata = Metadata::from_config(&config, is_multiverifier).guess(&mut context);

        // Hash it.
        let hashed_metadata = metadata.hash(&mut context);
        // The sibling hash in the 2-leaf metadata Merkle tree.
        let other_hash_var =
            HashValue::<Value>(Value::from_qm31(other_hash.0), Value::from_qm31(other_hash.1))
                .guess(&mut context);

        // The variant bit also serves as the Merkle leaf index. Constrain it
        // to be boolean so `cond_flip` (and the output-value selection below)
        // are sound — without `bit*bit == bit` an adversary could choose any
        // QM31 here.
        let bit = *metadata.bit.get();
        let bit_squared = eval!(&mut context, (bit) * (bit));
        eq(&mut context, bit_squared, bit);

        verify_merkle_path(
            &mut context,
            hashed_metadata,
            &[bit],
            metadata_root_var,
            &AuthPath(vec![other_hash_var]),
        );

        // Build statement (p1 and p2 must have the same # of outputs: here we assume 5).

        // Build the output values that need to be yielded.
        // Constrained outputs.
        let one = context.one();
        let one_minus_bit = sub(&mut context, one, bit);
        // If the circuit being verified is a multiverifier (bit = 1), we
        // require its first two outputs to equal *our own* metadata_root —
        // that's how `H` is propagated up the recursion. For a leaf circuit
        // (bit = 0) the corresponding outputs are padding zeros.
        // TODO: do the obvious optimization.
        let output0 = eval!(
            &mut context,
            ((bit) * (metadata_root_var.0)) + ((one_minus_bit) * (context.zero()))
        );
        let output1 = eval!(
            &mut context,
            ((bit) * (metadata_root_var.1)) + ((one_minus_bit) * (context.zero()))
        );
        let output4 = context.u();
        // Unconstrained outputs. (i.e. the hash-of-outputs of the circuit being verified now).
        let (output2, output3) = (
            Value::from_qm31(circuit_public_data.output_values[2]).guess(&mut context),
            Value::from_qm31(circuit_public_data.output_values[3]).guess(&mut context),
        );
        // Add the unconstrained outputs to the inner outputs.
        inner_outputs.extend([output2.clone(), output3.clone()]);

        let output_values = vec![output0, output1, output2, output3, output4];
        let statement = SubCircuitStatement::<Value> {
            components: all_circuit_components(),
            output_addresses: metadata.output_addresses,
            output_values,
            n_blake_gates_pow_two: metadata.n_blake_gates_pow_two,
            preprocessed_column_ids: subcircuit_config.preprocessed_column_ids.clone(),
            preprocessed_root: metadata.preprocessed_root,
        };

        let proof_vars = proof.guess(&mut context);
        verify(&mut context, &proof_vars, &proof_config, &statement);
    }
    // hash the hash-of-outputs
    let HashValue(hash0, hash1) = blake(&mut context, &inner_outputs, 64);
    output(&mut context, hash0);
    output(&mut context, hash1);

    // finalize consts
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context
}

// We will need
//
// pub fn prepare_circuit_proof_for_multicircuit_verifier(
//     circuit_proof: CircuitProof<Blake2sM31MerkleHasher>,
//     proof_config: &ProofConfig,
// ) -> (Proof<QM31>, CircuitPublicData)
