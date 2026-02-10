use std::array;

use starknet_ff::FieldElement;
use stwo::core::fields::m31::BaseField;
use stwo::core::fields::qm31::SecureField;

use crate::circuits::blake::HashValue;
use crate::circuits::wrappers::M31Wrapper;
use crate::stark_verifier::fri_proof::{FriCommitProof, FriConfig, FriProof};
use crate::stark_verifier::merkle::{AuthPath, AuthPaths};
use crate::stark_verifier::oods::{EvalDomainSamples, N_COMPOSITION_COLUMNS};
use crate::stark_verifier::proof::{Claim, InteractionAtOods, Proof};

/// Deserializes types from a format serialized by corresponding `CircuitSerialize` implementations.
pub trait CircuitDeserialize: Sized {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self;
}

impl CircuitDeserialize for Proof<SecureField> {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        let channel_salt = SecureField::deserialize(data);
        let preprocessed_root = HashValue::<SecureField>::deserialize(data);
        let trace_root = HashValue::<SecureField>::deserialize(data);
        let interaction_root = HashValue::<SecureField>::deserialize(data);
        let composition_polynomial_root = HashValue::<SecureField>::deserialize(data);
        let preprocessed_columns_at_oods = Vec::<SecureField>::deserialize(data);
        let trace_at_oods = Vec::<SecureField>::deserialize(data);
        let composition_eval_at_oods = <[SecureField; N_COMPOSITION_COLUMNS]>::deserialize(data);
        let claim = Claim::<SecureField>::deserialize(data);
        let interaction_at_oods = Vec::<InteractionAtOods<SecureField>>::deserialize(data);
        let eval_domain_samples = EvalDomainSamples::<SecureField>::deserialize(data);
        let eval_domain_auth_paths = AuthPaths::<SecureField>::deserialize(data);
        let proof_of_work_nonce = SecureField::deserialize(data);
        let interaction_pow_nonce = SecureField::deserialize(data);
        let fri = FriProof::<SecureField>::deserialize(data);
        Proof {
            channel_salt,
            preprocessed_root,
            trace_root,
            interaction_root,
            composition_polynomial_root,
            preprocessed_columns_at_oods,
            trace_at_oods,
            composition_eval_at_oods,
            claim,
            interaction_at_oods,
            eval_domain_samples,
            eval_domain_auth_paths,
            proof_of_work_nonce,
            interaction_pow_nonce,
            fri,
        }
    }
}

impl CircuitDeserialize for u32 {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        let field_elem = data.next().unwrap();
        let bytes = field_elem.to_bytes_be();
        if bytes[0..28] != [0; 28] {
            panic!("Invalid value for u32");
        }
        u32::from_be_bytes(bytes[28..32].try_into().unwrap())
    }
}

impl CircuitDeserialize for u64 {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        let field_elem = data.next().unwrap();
        let bytes = field_elem.to_bytes_be();
        if bytes[0..24] != [0; 24] {
            panic!("Invalid value for u64");
        }
        u64::from_be_bytes(bytes[24..32].try_into().unwrap())
    }
}

impl CircuitDeserialize for usize {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        <u64 as CircuitDeserialize>::deserialize(data) as usize
    }
}

impl CircuitDeserialize for BaseField {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        BaseField::from(u32::deserialize(data))
    }
}

impl CircuitDeserialize for SecureField {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        let mut m31_values = [BaseField::from(0); 4];
        #[allow(clippy::needless_range_loop)]
        for i in 0..4 {
            m31_values[i] = BaseField::deserialize(data);
        }

        SecureField::from_m31_array(m31_values)
    }
}

impl CircuitDeserialize for FieldElement {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        data.next().copied().unwrap()
    }
}

impl CircuitDeserialize for FriConfig {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        let log_trace_size = usize::deserialize(data);
        let log_blowup_factor = usize::deserialize(data);
        let n_queries = usize::deserialize(data);
        let log_n_last_layer_coefs = usize::deserialize(data);
        FriConfig { log_trace_size, log_blowup_factor, n_queries, log_n_last_layer_coefs }
    }
}

impl<T: CircuitDeserialize> CircuitDeserialize for Option<T> {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        let discriminant = data.next().unwrap();
        if *discriminant == FieldElement::ZERO {
            let value = T::deserialize(data);
            Some(value)
        } else if *discriminant == FieldElement::ONE {
            None
        } else {
            panic!("Invalid discriminant for Option<T>");
        }
    }
}

impl<T: CircuitDeserialize, const N: usize> CircuitDeserialize for [T; N] {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        array::from_fn(|_| T::deserialize(data))
    }
}

impl<T: CircuitDeserialize> CircuitDeserialize for Vec<T> {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        let len: usize = usize::deserialize(data);

        (0..len).map(|_| T::deserialize(data)).collect()
    }
}

impl CircuitDeserialize for HashValue<SecureField> {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        HashValue(SecureField::deserialize(data), SecureField::deserialize(data))
    }
}

impl CircuitDeserialize for Claim<SecureField> {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        Claim {
            packed_enable_bits: Vec::deserialize(data),
            packed_component_log_sizes: Vec::deserialize(data),
            claimed_sums: Vec::deserialize(data),
        }
    }
}

impl CircuitDeserialize for InteractionAtOods<SecureField> {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        InteractionAtOods {
            at_oods: SecureField::deserialize(data),
            at_prev: Option::deserialize(data),
        }
    }
}

impl CircuitDeserialize for AuthPath<SecureField> {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        AuthPath(Vec::deserialize(data))
    }
}

impl CircuitDeserialize for AuthPaths<SecureField> {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        AuthPaths {
            data: Vec::deserialize(data),
        }
    }
}

impl CircuitDeserialize for M31Wrapper<SecureField> {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        let m31 = BaseField::deserialize(data);
        M31Wrapper::from(m31)
    }
}

impl CircuitDeserialize for EvalDomainSamples<SecureField> {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        let n_traces = usize::deserialize(data);
        let mut data_vec = Vec::with_capacity(n_traces);
        for _ in 0..n_traces {
            let n_columns = usize::deserialize(data);
            let mut trace_data = Vec::with_capacity(n_columns);
            for _ in 0..n_columns {
                let column_len = usize::deserialize(data);
                trace_data.push(
                    (0..column_len)
                        .map(|_| BaseField::deserialize(data))
                        .collect(),
                );
            }
            data_vec.push(trace_data);
        }
        EvalDomainSamples::from_m31s(data_vec)
    }
}

impl CircuitDeserialize for FriCommitProof<SecureField> {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        FriCommitProof {
            layer_commitments: Vec::deserialize(data),
            last_layer_coefs: Vec::deserialize(data),
        }
    }
}

impl CircuitDeserialize for FriProof<SecureField> {
    fn deserialize<'a>(data: &mut impl Iterator<Item = &'a FieldElement>) -> Self {
        FriProof {
            commit: FriCommitProof::deserialize(data),
            auth_paths: AuthPaths::deserialize(data),
            fri_siblings: Vec::deserialize(data),
        }
    }
}
