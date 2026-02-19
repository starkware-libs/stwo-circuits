use stwo::core::fields::qm31::QM31;

use crate::deserialize::deserialize_proof_with_config;
use crate::serialize::CircuitSerialize;
use circuits_stark_verifier::proof::ProofConfig;
use circuits_stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use circuits_stark_verifier_examples::simple_air::create_proof;
use circuits_stark_verifier_examples::simple_statement::SimpleStatement;

#[test]
fn test_serialize_deserialize() {
    let (_components, claim, pcs_config, proof, interaction_pow_nonce, channel_salt) =
        create_proof();

    let statement = &SimpleStatement::<QM31>::default();
    let config = ProofConfig::from_statement(statement, &pcs_config, 8);
    let proof = proof_from_stark_proof(&proof, &config, claim, interaction_pow_nonce, channel_salt);

    let mut serialized = Vec::new();
    proof.serialize(&mut serialized);
    let deserialized = deserialize_proof_with_config(&mut serialized.as_slice(), &config).unwrap();
    assert_eq!(proof, deserialized);
}
