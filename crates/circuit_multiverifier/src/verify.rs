#![allow(unused)]
// Want:
// - two circuit proof test vectors.
// - define a single pcs config which is used by both circuit proofs.
// - both proofs are of the cairo verifier.

use circuit_verifier::{
    components::prelude::M31,
    verify::{CircuitConfig, CircuitPublicData, verify_circuit},
};
use circuits::context::Context;
use circuits::{
    blake::{HashValue, blake_qm31},
    context::Var,
    ivalue::IValue,
    ops::Guess,
};
use circuits_stark_verifier::proof::Proof;
use num_traits::Zero;
use stwo::core::fields::qm31::QM31;

// TODO: compute the true root
const VALID_METADATA_ROOT: HashValue<QM31> =
    HashValue(QM31::from_u32_unchecked(0, 0, 0, 0), QM31::from_u32_unchecked(0, 0, 0, 0));
enum ProofVariant {
    CairoVerifier,
    MultiVerifier,
}

struct Input<Value: IValue> {
    proof: Proof<Value>,
    circuit_public_data: CircuitPublicData,
    config: CircuitConfig,
}
// Multiverifier config?

pub struct Metadata<T> {
    log_n_blake_gates: T,
    output_addresses: Vec<T>,
    preprocessed_root: HashValue<T>,
}

impl Metadata<QM31> {
    pub fn serialize_to_qm31(self) -> Vec<QM31> {
        let Metadata { log_n_blake_gates, output_addresses, preprocessed_root } = self;

        let mut res = vec![log_n_blake_gates];
        res.extend(output_addresses.iter());
        res.extend([preprocessed_root.0, preprocessed_root.1]);
        res
    }

    pub fn from_config(config: CircuitConfig) -> Self {
        let log_n_blake_gates: QM31 = config.n_blake_gates.next_power_of_two().ilog2().into();
        let output_addresses: Vec<QM31> =
            config.output_addresses.iter().map(|x| QM31::from(*x)).collect();
        Metadata {
            log_n_blake_gates,
            output_addresses,
            preprocessed_root: config.preprocessed_root,
        }
    }
}

impl<Value: IValue> Guess<Value> for Metadata<Value> {
    type Target = Metadata<Var>;
    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        todo!()
    }
}

fn merkleize_metadata(m1: Metadata<QM31>, m2: Metadata<QM31>) -> HashValue<QM31> {
    let m1_qm31s = m1.serialize_to_qm31();
    let m2_qm31s = m2.serialize_to_qm31();
    let hash_1 = blake_qm31(&m1_qm31s, 16 * m1_qm31s.len());
    let hash_2 = blake_qm31(&m2_qm31s, 16 * m2_qm31s.len());
    blake_qm31(&[hash_1.0, hash_1.1, hash_1.0, hash_2.1], 64)
}

fn build_multiverifier_circuit<Value: IValue>(
    p1: Input<Value>,
    p2: Input<Value>,
) -> Context<Value> {
    let Input { proof: proof_1, circuit_public_data: pub_data_1, config: circuit_config_1 } = p1;

    todo!()
}

// We will need
//
// pub fn prepare_circuit_proof_for_multicircuit_verifier(
//     circuit_proof: CircuitProof<Blake2sM31MerkleHasher>,
//     proof_config: &ProofConfig,
// ) -> (Proof<QM31>, CircuitPublicData)
