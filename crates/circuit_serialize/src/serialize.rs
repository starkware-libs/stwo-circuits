use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use circuits::blake::HashValue;
use circuits::wrappers::M31Wrapper;
use circuits_stark_verifier::fri_proof::{FriCommitProof, FriProof, FriWitness};
use circuits_stark_verifier::merkle::{AuthPath, AuthPaths};
use circuits_stark_verifier::oods::EvalDomainSamples;
use circuits_stark_verifier::proof::{Claim, InteractionAtOods, Proof};
use circuits_stark_verifier::verify::LOG_SIZE_BITS;

pub trait CircuitSerialize {
    fn serialize(&self, output: &mut Vec<u8>);
}

impl CircuitSerialize for Proof<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self {
            channel_salt,
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
            pow_nonce,
            interaction_pow_nonce,
            fri,
        } = self;

        channel_salt.serialize(output);
        trace_root.serialize(output);
        interaction_root.serialize(output);
        composition_polynomial_root.serialize(output);
        claim.serialize(output);
        preprocessed_columns_at_oods.as_slice().serialize(output);
        trace_at_oods.as_slice().serialize(output);
        interaction_at_oods.as_slice().serialize(output);
        composition_eval_at_oods.serialize(output);
        eval_domain_samples.serialize(output);
        eval_domain_auth_paths.serialize(output);
        pow_nonce.serialize(output);
        interaction_pow_nonce.serialize(output);
        fri.serialize(output);
    }
}

impl CircuitSerialize for M31 {
    fn serialize(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.0.to_le_bytes());
    }
}

impl CircuitSerialize for QM31 {
    fn serialize(&self, output: &mut Vec<u8>) {
        for c in self.to_m31_array() {
            c.serialize(output);
        }
    }
}

impl<T: CircuitSerialize> CircuitSerialize for [T] {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.iter().for_each(|v| v.serialize(output));
    }
}

impl<T: CircuitSerialize, const N: usize> CircuitSerialize for [T; N] {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.as_slice().serialize(output);
    }
}

impl<T: CircuitSerialize> CircuitSerialize for Vec<T> {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.as_slice().serialize(output);
    }
}

impl CircuitSerialize for HashValue<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self(a, b) = self;
        a.serialize(output);
        b.serialize(output);
    }
}

impl CircuitSerialize for Claim<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self { packed_component_log_sizes, claimed_sums } = self;

        // Pack log sizes: 1 per u8 (requires `LOG_SIZE_BITS` <= 8).
        assert!(LOG_SIZE_BITS as usize <= 8);
        for qm31 in packed_component_log_sizes {
            for m31 in qm31.to_m31_array().into_iter() {
                output.push((m31.0 & 0xFF) as u8);
            }
        }

        claimed_sums.serialize(output);
    }
}

impl CircuitSerialize for InteractionAtOods<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self { at_oods, at_prev } = self;
        at_oods.serialize(output);
        if let Some(at_prev) = at_prev {
            at_prev.serialize(output);
        }
    }
}

impl CircuitSerialize for AuthPath<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self(path) = self;
        path.serialize(output);
    }
}

impl CircuitSerialize for AuthPaths<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self { data } = self;
        data.serialize(output);
    }
}

impl CircuitSerialize for M31Wrapper<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        // M31Wrapper wraps a value known to be in the base field M31.
        let m31: M31 = self.get().0.0;
        m31.serialize(output);
    }
}

impl CircuitSerialize for EvalDomainSamples<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let n_traces = self.n_traces();
        for trace_idx in 0..n_traces {
            let trace_data = self.data_for_trace(trace_idx);
            for column in trace_data {
                for cell in column {
                    cell.serialize(output);
                }
            }
        }
    }
}

impl CircuitSerialize for FriCommitProof<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self { layer_commitments, last_layer_coefs } = self;
        layer_commitments.serialize(output);
        last_layer_coefs.serialize(output);
    }
}

impl CircuitSerialize for FriWitness<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self(witness_per_query_per_tree) = self;
        witness_per_query_per_tree.serialize(output);
    }
}

impl CircuitSerialize for FriProof<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self { commit, auth_paths, witness } = self;
        commit.serialize(output);
        auth_paths.serialize(output);
        witness.serialize(output);
    }
}
