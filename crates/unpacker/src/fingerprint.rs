//! Structural fingerprint of a built circuit.
//!
//! A circuit's topology must be a function of its shape parameters alone: the same
//! parameters must always produce the same gates, in the same order, whatever the witness
//! values are and whatever process builds it. Two failure modes break that, and both are
//! invisible in a passing functional test:
//!
//! * **witness-dependent topology** — the verifier then checks a different statement than the one
//!   the prover's circuit expresses;
//! * **per-process nondeterminism** — iterating a `HashMap`/`HashSet` while emitting gates picks up
//!   std's per-process random seed, so the gate order changes between runs.
//!
//! [`circuit_fingerprint`] hashes the structure — gate counts per kind, then every gate's
//! `uses`/`yields` in `all_gates` order — so tests can pin it. A pinned fingerprint also
//! documents every intentional topology change: it must be updated deliberately, in the
//! commit that changes the circuit.
//!
//! Values are deliberately *not* hashed: two witnesses over the same shape must share a
//! fingerprint.

#[cfg(test)]
#[path = "fingerprint_test.rs"]
mod test;

use circuits::circuit::Circuit;
use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasher};

/// Hashes a circuit's topology (not its witness values). See the module docs.
pub fn circuit_fingerprint(circuit: &Circuit) -> Blake2sHash {
    let mut hasher = Blake2sHasher::new();
    hasher.update(&(circuit.n_vars as u64).to_le_bytes());
    for (index, gate) in circuit.all_gates().enumerate() {
        // The index pins gate ORDER, not just the multiset of gates.
        hasher.update(&(index as u64).to_le_bytes());
        hash_vars(&mut hasher, &gate.uses());
        hash_vars(&mut hasher, &gate.yields());
    }
    hasher.finalize()
}

fn hash_vars(hasher: &mut Blake2sHasher, vars: &[usize]) {
    hasher.update(&(vars.len() as u64).to_le_bytes());
    for var in vars {
        hasher.update(&(*var as u64).to_le_bytes());
    }
}
