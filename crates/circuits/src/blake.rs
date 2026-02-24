use blake2::{Blake2s256, Digest};
use itertools::Itertools;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::{fields::qm31::QM31, vcs::blake2_hash::reduce_to_m31};

use crate::circuit::Blake;
use crate::context::{Context, Var};
use crate::ivalue::{IValue, qm31_from_u32s};
use crate::ops::Guess;
use crate::stats::StatEntryKind;

#[cfg(test)]
#[path = "blake_test.rs"]
pub mod test;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HashValue<T>(pub T, pub T);

impl<Value: IValue> Guess<Value> for HashValue<Value> {
    type Target = HashValue<Var>;
    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        HashValue(self.0.guess(context), self.1.guess(context))
    }
}

impl From<Blake2sHash> for HashValue<QM31> {
    fn from(value: Blake2sHash) -> Self {
        HashValue(
            qm31_from_bytes(&value.0[0..16].try_into().unwrap()),
            qm31_from_bytes(&value.0[16..32].try_into().unwrap()),
        )
    }
}

/// Convert QM31 to 16 bytes (4 u32s)
fn to_bytes(value: QM31) -> [u8; 16] {
    let mut bytes = [0u8; 16];
    bytes[0..4].copy_from_slice(&value.0.0.0.to_le_bytes());
    bytes[4..8].copy_from_slice(&value.0.1.0.to_le_bytes());
    bytes[8..12].copy_from_slice(&value.1.0.0.to_le_bytes());
    bytes[12..16].copy_from_slice(&value.1.1.0.to_le_bytes());
    bytes
}

/// Create QM31 from 16 bytes (4 u32s)
pub fn qm31_from_bytes(bytes: &[u8; 16]) -> QM31 {
    let a = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let b = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let c = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let d = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    qm31_from_u32s(a, b, c, d)
}

/// Blake2s hash function implementation for QM31.
/// Takes [QM31] values as input and returns 2 [QM31] values as output.
pub fn blake_qm31(input: &[QM31], n_bytes: usize) -> HashValue<QM31> {
    // Sanity check: check the number of bytes is consistent with the number of [QM31] values.
    assert_eq!(input.len(), n_bytes.div_ceil(16));

    // Convert [QM31] inputs to bytes.
    let mut input_bytes: Vec<u8> = vec![];
    for x in input {
        input_bytes.extend_from_slice(&to_bytes(*x));
    }

    let mut hasher = Blake2s256::new();
    hasher.update(&input_bytes[0..n_bytes]);
    let hash: [u8; 32] = reduce_to_m31(hasher.finalize().into());

    let res0 = qm31_from_bytes(hash[0..16].try_into().unwrap());
    let res1 = qm31_from_bytes(hash[16..32].try_into().unwrap());

    HashValue(res0, res1)
}

/// Adds a blake hash gate to the circuit, and returns the two output variables as [HashValue].
///
/// NOTE: If the number of bytes is not a multiple of 16, the caller must make sure that the
/// remaining bytes are zero.
/// For example, if `n_bytes` is 4, only the first coordinate of the [QM31] may be non-zero.
/// If `n_bytes` is 1, that coordinate must be < 256.
pub fn blake<Value: IValue>(
    context: &mut Context<Value>,
    input: &[Var],
    n_bytes: usize,
) -> HashValue<Var> {
    // Sanity check: check the number of bytes is consistent with the number of [QM31] values.
    assert_eq!(input.len(), n_bytes.div_ceil(16));

    // Compute the hash.
    let out = Value::blake(&input.iter().map(|v| context.get(*v)).collect::<Vec<_>>(), n_bytes);

    // Pad input with zeros and split into chunks of 4 [QM31] values.
    let zero_idx = context.zero().idx;
    let chunks = input
        .iter()
        .chunks(4)
        .into_iter()
        .map(|chunk| {
            let mut res = [zero_idx; 4];
            for (i, v) in chunk.enumerate() {
                res[i] = v.idx;
            }
            res
        })
        .collect_vec();
    context.stats.blake_updates += chunks.len();
    // context.stats.register(StatEntryKind::Blake { updates: chunks.len() });
    let out_var0 = context.new_var(out.0);
    let out_var1 = context.new_var(out.1);

    context.circuit.blake.push(Blake {
        input: chunks,
        n_bytes,
        out0: out_var0.idx,
        out1: out_var1.idx,
    });

    HashValue(out_var0, out_var1)
}
