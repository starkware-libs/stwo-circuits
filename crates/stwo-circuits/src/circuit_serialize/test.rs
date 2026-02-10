#[cfg(test)]
mod test {

    use stwo::core::fields::qm31::QM31;

    use crate::circuit_serialize::{
        deserialize::CircuitDeserialize, serialize::CircuitSerialize,
    };
    use crate::examples::simple_air::{LOG_SIZE_LONG, create_proof};
    use crate::examples::simple_statement::SimpleStatement;
    use crate::stark_verifier::proof::{Proof, ProofConfig};
    use crate::stark_verifier::proof_from_stark_proof::proof_from_stark_proof;

    #[test]
    fn test_serialize_deserialize() {
        let (_components, claim, pcs_config, proof, interaction_pow_nonce, channel_salt) =
            create_proof();

        let statement = &SimpleStatement::<QM31>::default();
        let config =
            ProofConfig::from_statement(statement, LOG_SIZE_LONG as usize, &pcs_config, 8);
        let proof =
            proof_from_stark_proof(&proof, &config, claim, interaction_pow_nonce, channel_salt);

        let mut serialized = Vec::new();
        proof.serialize(&mut serialized);
        let deserialized = Proof::deserialize(&mut serialized.iter());
        assert_eq!(proof, deserialized);
    }
}
