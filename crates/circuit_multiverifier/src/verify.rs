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
use circuits::{context::Context, eval, ivalue::qm31_from_u32s, ops::sub, wrappers::M31Wrapper};
use circuits_stark_verifier::{
    merkle::{AuthPath, verify_merkle_path},
    proof::{Proof, ProofConfig},
    verify::verify,
};
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};

use crate::statement::SubCircuitStatement;

#[cfg(test)]
#[path ="verify_test.rs"]
mod verify_test;

// TODO: compute the true root
const VALID_METADATA_ROOT: HashValue<QM31> =
    HashValue(QM31::from_u32_unchecked(0, 0, 0, 0), QM31::from_u32_unchecked(0, 0, 0, 0));

pub struct Input<Value: IValue> {
    proof: Proof<Value>,
    circuit_public_data: CircuitPublicData,
    config: CircuitConfig,
}
// Multiverifier config?

pub struct Metadata<T> {
    n_blake_gates_pow_two: M31Wrapper<T>,
    output_addresses: Vec<M31Wrapper<T>>,
    preprocessed_root: HashValue<T>,
}

impl Metadata<QM31> {
    pub fn serialize_to_qm31(self) -> Vec<QM31> {
        let Metadata { n_blake_gates_pow_two, output_addresses, preprocessed_root } = self;

        let mut res = vec![*n_blake_gates_pow_two.get()];
        // Add domain separation for length.
        res.extend(output_addresses.iter().map(|x| *x.get()));
        res.extend([preprocessed_root.0, preprocessed_root.1]);
        res
    }
}

impl<Value: IValue> Metadata<Value> {
    pub fn from_config(config: CircuitConfig) -> Self {
        let n_blake_gates_pow_two = M31Wrapper::new_unsafe(Value::from_qm31(QM31::from(
            config.n_blake_gates.next_power_of_two(),
        )));

        let output_addresses = config
            .output_addresses
            .iter()
            .map(|x| M31Wrapper::new_unsafe(Value::from_qm31(QM31::from(*x))))
            .collect();
        Metadata {
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
            n_blake_gates_pow_two: self.n_blake_gates_pow_two.guess(context),
            output_addresses: self.output_addresses.guess(context),
            preprocessed_root: self.preprocessed_root.guess(context),
        }
    }
}

impl Metadata<Var> {
    fn serialize_to_qm31(&self) -> Vec<Var> {
        let Metadata { n_blake_gates_pow_two, output_addresses, preprocessed_root } = self;

        let mut res = vec![*n_blake_gates_pow_two.get()];
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

fn merkleize_metadata(m1: Metadata<QM31>, m2: Metadata<QM31>) -> HashValue<QM31> {
    let m1_qm31s = m1.serialize_to_qm31();
    let m2_qm31s = m2.serialize_to_qm31();
    let hash_1 = blake_qm31(&m1_qm31s, 16 * m1_qm31s.len());
    let hash_2 = blake_qm31(&m2_qm31s, 16 * m2_qm31s.len());
    blake_qm31(&[hash_1.0, hash_1.1, hash_2.0, hash_2.1], 64)
}

pub fn build_multiverifier_circuit<Value: IValue>(
    p1: Input<Value>,
    p2: Input<Value>,
    // TODO: return a result.
) -> Context<Value> {
    assert_eq!(p1.config.preprocessed_column_ids, p2.config.preprocessed_column_ids);

    let mut context: Context<Value> = Context::default();
    let metadata_root =
        HashValue(Value::from_qm31(VALID_METADATA_ROOT.0), Value::from_qm31(VALID_METADATA_ROOT.1));
    let metadata_root_var = metadata_root.guess(&mut context);
    output(&mut context, metadata_root_var.0);
    output(&mut context, metadata_root_var.1);

    // Define the pcs config which MUST be the same for the two proofs.
    let mut pcs_config = PcsConfig::default();
    pcs_config.lifting_log_size = Some(23);

    let mut inner_outputs = vec![];
    for input in [p1, p2] {
        let Input { proof, circuit_public_data, config } = input;

        // Build metadata out of config.
        // Save log n blake gates for later.
        // TODO: the pp cols ids need to be unchanged. -> Move out of the loop
        let preprocessed_column_ids = config.preprocessed_column_ids.clone();
        let metadata_1 = Metadata::from_config(config).guess(&mut context);

        // Hash it.
        let hashed_metadata = metadata_1.hash(&mut context);
        // TODO: need to build it properly.
        let other_hash = HashValue::<Value>(Value::from_qm31(0.into()), Value::from_qm31(0.into()))
            .guess(&mut context);
        let bit = context.zero();
        // Verify merkle path.
        verify_merkle_path(
            &mut context,
            hashed_metadata,
            &[bit],
            metadata_root_var,
            &AuthPath(vec![other_hash]),
        );
        // Build statement (p1 and p2 must have the same # of outputs: here we assume 5).

        // Build the output values that need to be yielded.
        // Constrained outputs.
        let one = context.one();
        let one_minus_bit = sub(&mut context, one, bit);
        // If the circuit being verified now is a multiverifier, we need to verify its first two
        // outputs against our own metadata root. Otherwise, the outputs are junk (we verify that
        // they are just padding zeros).
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
            output_addresses: metadata_1.output_addresses,
            output_values,
            n_blake_gates_pow_two: metadata_1.n_blake_gates_pow_two,
            preprocessed_column_ids: preprocessed_column_ids.clone(),
            preprocessed_root: metadata_1.preprocessed_root,
        };

        let all_enable_bits = vec![true; statement.components.len()];
        // TODO: proof config should be indpendent of proofs. Move out of the loop
        let proof_config = ProofConfig::from_statement(
            &statement,
            all_enable_bits,
            &pcs_config,
            INTERACTION_POW_BITS,
        );
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
