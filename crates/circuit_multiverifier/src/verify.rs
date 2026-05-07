// Want:
// - two circuit proof test vectors.
// - define a single pcs config which is used by both circuit proofs.
// - both proofs are of the cairo verifier.

use circuit_verifier::{
    statement::{INTERACTION_POW_BITS, all_circuit_components},
    verify::{CircuitConfig, CircuitPublicData},
};
use circuits::{
    blake::{HashValue, blake},
    context::Var,
    finalize_constants::finalize_constants,
    ivalue::{IValue, NoValue},
    ops::{Guess, mul, output},
};
use circuits::{context::Context, eval, ops::eq, wrappers::M31Wrapper};
use circuits_stark_verifier::{
    merkle::{AuthPath, verify_merkle_path},
    proof::{Proof, ProofConfig},
    verify::verify,
};
use itertools::{Itertools, chain};
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::statement::SubCircuitStatement;

#[cfg(test)]
#[path = "verify_test.rs"]
mod verify_test;

#[cfg(test)]
#[path = "multi_fibonacci_test.rs"]
mod multi_fibonacci_test;

#[cfg(test)]
#[path = "multi_cairo_test.rs"]
mod multi_cairo_test;

// TODO: maybe remove this struct?
pub struct SubCircuitInput<Value: IValue> {
    pub proof: Proof<Value>,
    pub circuit_public_data: CircuitPublicData,
    pub config: CircuitConfig,
    /// `true` if this proof is a proof of the multiverifier circuit itself
    /// (variant B in the recursion); `false` for a leaf circuit such as the
    /// Fibonacci/`circuit_verifier` proof (variant A).
    pub is_multiverifier: bool,
}

// TODO: find better name.
#[derive(Debug, Clone)]
pub struct Metadata<T> {
    pub n_blake_gates_pow_two: M31Wrapper<T>,
    pub output_addresses: Vec<M31Wrapper<T>>,
    pub preprocessed_root: HashValue<T>,
}

impl<Value: IValue> Metadata<Value> {
    pub fn from_config(config: &CircuitConfig) -> Self {
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

impl<T: Copy> Metadata<T> {
    fn flatten(&self) -> Vec<T> {
        let Metadata { n_blake_gates_pow_two, output_addresses, preprocessed_root } = self;
        chain![
            [*n_blake_gates_pow_two.get()],
            output_addresses.iter().map(|x| x.get()).copied(),
            [preprocessed_root.0, preprocessed_root.1]
        ]
        .collect()
    }
}

impl Metadata<Var> {
    fn hash<Value: IValue>(&self, context: &mut Context<Value>) -> HashValue<Var> {
        let flattened = self.flatten();
        blake(context, &flattened, 16 * flattened.len())
    }
}

impl<Value: IValue> Metadata<Value> {
    fn hash_value(&self) -> HashValue<Value> {
        let flattened = self.flatten();
        Value::blake(&flattened, 16 * flattened.len())
    }
}

pub struct MetadataTree<Value: IValue> {
    root: HashValue<Value>,
    leaves: Vec<HashValue<Value>>,
}

impl<Value: IValue> MetadataTree<Value> {
    pub fn commit(m1: Metadata<Value>, m2: Metadata<Value>) -> Self {
        let mut leaves = vec![];
        for metadata in [m1, m2] {
            let hashed_metadata = metadata.hash_value();
            leaves.push(hashed_metadata);
        }
        let root = Value::blake(&leaves.iter().flat_map(|&x| [x.0, x.1]).collect_vec(), 64);

        MetadataTree { root, leaves }
    }

    pub fn decommit(&self, bit: usize) -> AuthPath<Value> {
        assert!((0..=1).contains(&bit));
        AuthPath(vec![self.leaves[bit ^ 1]])
    }
}

pub fn empty_metadata(n_outputs: usize) -> Metadata<NoValue> {
    Metadata {
        n_blake_gates_pow_two: M31Wrapper::from(NoValue {}),
        output_addresses: vec![M31Wrapper::from(NoValue {}); n_outputs],
        preprocessed_root: HashValue(NoValue {}, NoValue {}),
    }
}

/// Contains all the parameters that are fixed for the proofs being validated.
pub struct SubCircuitConfig {
    pub pcs_config: PcsConfig,
    pub n_outputs: usize,
    pub preprocessed_column_ids: Vec<PreProcessedColumnId>,
}

impl SubCircuitConfig {
    pub fn to_proof_config(&self) -> ProofConfig {
        let all_components = all_circuit_components::<QM31>();
        ProofConfig::new(
            &all_components,
            vec![true; all_components.len()],
            self.preprocessed_column_ids.len(),
            &self.pcs_config,
            INTERACTION_POW_BITS,
        )
    }
}

// TODO: circuit config does not belong to input?
pub fn build_multiverifier_circuit<Value: IValue>(
    i1: SubCircuitInput<Value>,
    i2: SubCircuitInput<Value>,
    // Things that are constant
    subcircuit_config: SubCircuitConfig,
    // Things that are constant but we need to guess.
    metadata_tree: MetadataTree<Value>,
    // TODO: return a result.
) -> Context<Value> {
    // ProofConfig that is shared between the two proofs.
    let proof_config = subcircuit_config.to_proof_config();
    let mut context: Context<Value> = Context::default();
    let metadata_root_var = metadata_tree.root.guess(&mut context);

    let mut inner_outputs = vec![];
    // Verify each subcircuit proof.
    for subcircuit_input in [i1, i2] {
        let SubCircuitInput { proof, circuit_public_data, config, is_multiverifier } =
            subcircuit_input;
        // Verify the metadata
        let metadata = Metadata::from_config(&config).guess(&mut context);
        let hashed_metadata = metadata.hash(&mut context);
        let auth_path = metadata_tree.decommit(is_multiverifier as usize).guess(&mut context);
        let bit = Value::from_qm31(QM31::from(is_multiverifier as usize)).guess(&mut context);
        let bit_squared = eval!(&mut context, (bit) * (bit));
        eq(&mut context, bit_squared, bit);
        verify_merkle_path(&mut context, hashed_metadata, &[bit], metadata_root_var, &auth_path);

        // Build the values that are supposedly the output wires of the circuit being verified.
        // First build the "unconstrained" outputs by taking them directly from the public data.
        let (output0, output1) = (
            Value::from_qm31(circuit_public_data.output_values[0]).guess(&mut context),
            Value::from_qm31(circuit_public_data.output_values[1]).guess(&mut context),
        );
        // Then build the constrained outputs.
        // If the circuit being verified is a multiverifier (bit = 1), we
        // require its first two outputs to equal *our own* metadata_root —
        // that's how `H` is propagated up the recursion. For a leaf circuit
        // (bit = 0) the corresponding outputs are padding zeros.
        let output2 = mul(&mut context, bit, metadata_root_var.0);
        let output3 = mul(&mut context, bit, metadata_root_var.1);
        let output4 = context.u();
        let output_values = vec![output0, output1, output2, output3, output4];
        // Add the unconstrained outputs to the inner outputs, which will need to be hashed at the
        // end.
        inner_outputs.extend([output0.clone(), output1.clone()]);

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
    // Hash the unconstrained outputs of the inner proofs and output the hash.
    let HashValue(hash0, hash1) = blake(&mut context, &inner_outputs, 64);
    output(&mut context, hash0);
    output(&mut context, hash1);
    // Output the metadata root guessed at the beginning.
    output(&mut context, metadata_root_var.0);
    output(&mut context, metadata_root_var.1);

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

// To verify a circuit, you need a circuit statement and a proof config, so that you can call
// verify.

// How do they affect the topology of the resulting verifier circuit?
// 1. output addresses are added as consts: bad
// 1'. even if they were somehow guessed, their number affects the # of vars of the resulting
// circuit
// 2. preprocessed_cols: ordered set of pp cols affects the wiring of the resulting circuit when it
//    needs to compute the constraints at ood. -> requirement: given a pcs config, (the witnesses
//    of) circuit A and circuit B need to have the same ordered set of pp cols
// 3. pp root added as const: bad
// 4. n_blake_gates.next_power_of_two added as const: bad

// What is currently added as a const will be guessed and then verified by the next verifier.
