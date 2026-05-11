use circuit_verifier::{statement::all_circuit_components, verify::CircuitConfig};
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
use stwo::core::fields::qm31::QM31;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::statement::SubCircuitStatement;

#[cfg(test)]
#[path = "verify_test.rs"]
mod verify_test;

// #[cfg(test)]
// #[path = "multi_fibonacci_test.rs"]
// mod multi_fibonacci_test;

#[cfg(test)]
#[path = "multi_cairo_test.rs"]
mod multi_cairo_test;

/// All per-subproof data the multiverifier consumes.
///
/// Compared to the project-wide `CircuitConfig` / `CircuitPublicData`, this
/// only carries what `build_multiverifier_circuit` actually reads:
/// * `claimed_params` — the leaf-of-`metadata_tree` triple (`n_blake_gates_pow_two`,
///   `output_addresses`, `preprocessed_root`).
/// * `unconstrained_outputs` — the two `output_values[0..2]` slots that are guessed straight from
///   the public data (the other three slots are constrained or padding zeros, so they aren't
///   inputs).
///
/// Use `ClaimedParams::from_config` if you have a `CircuitConfig` lying around.
pub struct MultiverifierInput<Value: IValue> {
    pub proof: Proof<Value>,
    pub claimed_params: ClaimedParams<Value>,
    pub unconstrained_outputs: [QM31; 2],
    pub is_multiverifier: bool,
}

#[derive(Debug, Clone)]
pub struct ClaimedParams<T> {
    pub n_blake_gates_pow_two: M31Wrapper<T>,
    pub output_addresses: Vec<M31Wrapper<T>>,
    pub preprocessed_root: HashValue<T>,
}

impl<Value: IValue> ClaimedParams<Value> {
    pub fn from_config(config: &CircuitConfig) -> Self {
        let n_blake_gates_pow_two = M31Wrapper::new_unsafe(Value::from_qm31(QM31::from(
            config.n_blake_gates.next_power_of_two(),
        )));

        let output_addresses = config
            .output_addresses
            .iter()
            .map(|x| M31Wrapper::new_unsafe(Value::from_qm31(QM31::from(*x))))
            .collect();

        ClaimedParams {
            n_blake_gates_pow_two,
            output_addresses,
            preprocessed_root: HashValue(
                Value::from_qm31(config.preprocessed_root.0),
                Value::from_qm31(config.preprocessed_root.1),
            ),
        }
    }
}

impl<Value: IValue> Guess<Value> for ClaimedParams<Value> {
    type Target = ClaimedParams<Var>;
    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        ClaimedParams {
            n_blake_gates_pow_two: self.n_blake_gates_pow_two.guess(context),
            output_addresses: self.output_addresses.guess(context),
            preprocessed_root: self.preprocessed_root.guess(context),
        }
    }
}

impl<T: Copy> ClaimedParams<T> {
    fn flatten(&self) -> Vec<T> {
        let ClaimedParams { n_blake_gates_pow_two, output_addresses, preprocessed_root } = self;
        chain![
            [*n_blake_gates_pow_two.get()],
            output_addresses.iter().map(|x| x.get()).copied(),
            [preprocessed_root.0, preprocessed_root.1]
        ]
        .collect()
    }
}

impl ClaimedParams<Var> {
    fn hash<Value: IValue>(&self, context: &mut Context<Value>) -> HashValue<Var> {
        let flattened = self.flatten();
        blake(context, &flattened, 16 * flattened.len())
    }
}

impl<Value: IValue> ClaimedParams<Value> {
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
    pub fn commit(m1: ClaimedParams<Value>, m2: ClaimedParams<Value>) -> Self {
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

pub fn empty_claimed_params(n_outputs: usize) -> ClaimedParams<NoValue> {
    ClaimedParams {
        n_blake_gates_pow_two: M31Wrapper::from(NoValue {}),
        output_addresses: vec![M31Wrapper::from(NoValue {}); n_outputs],
        preprocessed_root: HashValue(NoValue {}, NoValue {}),
    }
}

pub struct CommonConfig {
    pub proof_config: ProofConfig,
    pub preprocessed_column_ids: Vec<PreProcessedColumnId>,
}

pub fn build_multiverifier_circuit<Value: IValue>(
    i1: MultiverifierInput<Value>,
    i2: MultiverifierInput<Value>,
    common_config: &CommonConfig,
    claimed_params_tree: MetadataTree<Value>,
    // TODO: return a result.
) -> Context<Value> {
    let mut context: Context<Value> = Context::default();
    let claimed_params_root_var = claimed_params_tree.root.guess(&mut context);

    let mut inner_outputs = vec![];
    // Verify each subcircuit proof.
    for multiverifier_input in [i1, i2] {
        let MultiverifierInput {
            proof,
            claimed_params,
            unconstrained_outputs,
            is_multiverifier,
        } = multiverifier_input;

        debug_assert_eq!(claimed_params.hash_value(), claimed_params_tree.leaves[is_multiverifier as usize],);
        // Verify the metadata
        let claimed_params = claimed_params.guess(&mut context);
        let hashed_claimed_params = claimed_params.hash(&mut context);
        let auth_path = claimed_params_tree.decommit(is_multiverifier as usize).guess(&mut context);
        let bit = Value::from_qm31(QM31::from(is_multiverifier as usize)).guess(&mut context);
        let bit_squared = eval!(&mut context, (bit) * (bit));
        eq(&mut context, bit_squared, bit);
        verify_merkle_path(&mut context, hashed_claimed_params, &[bit], claimed_params_root_var, &auth_path);

        // Build output_values of the circuit being verified.
        // First build the "unconstrained" outputs by taking them directly from the public data.
        let (output0, output1) = (
            Value::from_qm31(unconstrained_outputs[0]).guess(&mut context),
            Value::from_qm31(unconstrained_outputs[1]).guess(&mut context),
        );
        // Then build the constrained outputs.
        // If the circuit being verified is a multiverifier (bit = 1), we
        // require its first two outputs to equal *our own* metadata_root —
        // that's how `H` is propagated up the recursion. For a leaf circuit
        // (bit = 0) the corresponding outputs are padding zeros.
        let output2 = mul(&mut context, bit, claimed_params_root_var.0);
        let output3 = mul(&mut context, bit, claimed_params_root_var.1);
        let output4 = context.u();
        let output_values = vec![output0, output1, output2, output3, output4];
        // Add the unconstrained outputs to the inner outputs, which will need to be hashed at the
        // end.
        inner_outputs.extend([output0.clone(), output1.clone()]);

        let statement = SubCircuitStatement::<Value> {
            components: all_circuit_components(),
            output_addresses: claimed_params.output_addresses,
            output_values,
            n_blake_gates_pow_two: claimed_params.n_blake_gates_pow_two,
            preprocessed_column_ids: common_config.preprocessed_column_ids.clone(),
            preprocessed_root: claimed_params.preprocessed_root,
        };

        let proof_vars = proof.guess(&mut context);
        verify(&mut context, &proof_vars, &common_config.proof_config, &statement);
    }
    // Hash the unconstrained outputs of the inner proofs and output the hash.
    let HashValue(hash0, hash1) = blake(&mut context, &inner_outputs, 64);
    output(&mut context, hash0);
    output(&mut context, hash1);
    // Output the claimed_params_root guessed at the beginning.
    output(&mut context, claimed_params_root_var.0);
    output(&mut context, claimed_params_root_var.1);

    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context
}
