use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use circuits::blake::HashValue;
use circuits::wrappers::M31Wrapper;
use circuits_stark_verifier::fri_proof::{FriCommitProof, FriProof};
use circuits_stark_verifier::merkle::{AuthPath, AuthPaths};
use circuits_stark_verifier::oods::EvalDomainSamples;
use circuits_stark_verifier::proof::{Claim, InteractionAtOods, Proof};

pub trait CircuitSerialize {
    fn serialize(&self, output: &mut Vec<u8>);
}

impl CircuitSerialize for Proof<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self {
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
        } = self;

        CircuitSerialize::serialize(channel_salt, output);
        CircuitSerialize::serialize(preprocessed_root, output);
        CircuitSerialize::serialize(trace_root, output);
        CircuitSerialize::serialize(interaction_root, output);
        CircuitSerialize::serialize(composition_polynomial_root, output);
        CircuitSerialize::serialize(preprocessed_columns_at_oods.as_slice(), output);
        CircuitSerialize::serialize(trace_at_oods.as_slice(), output);
        CircuitSerialize::serialize(composition_eval_at_oods, output);
        CircuitSerialize::serialize(claim, output);
        CircuitSerialize::serialize(interaction_at_oods.as_slice(), output);
        CircuitSerialize::serialize(eval_domain_samples, output);
        CircuitSerialize::serialize(eval_domain_auth_paths, output);
        CircuitSerialize::serialize(proof_of_work_nonce, output);
        CircuitSerialize::serialize(interaction_pow_nonce, output);
        CircuitSerialize::serialize(fri, output);
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
        let Self { packed_enable_bits, packed_component_log_sizes, claimed_sums } = self;
        packed_enable_bits.serialize(output);
        packed_component_log_sizes.serialize(output);
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

impl CircuitSerialize for FriProof<QM31> {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Self { commit, auth_paths, witness_per_query_per_tree } =
            self;
        commit.serialize(output);
        auth_paths.serialize(output);
        witness_per_query_per_tree.serialize(output);
    }
}
