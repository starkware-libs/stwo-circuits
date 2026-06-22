//! Analyzes the Cairo-verifier [`circuits::circuit::Circuit`].

use circuits::context::Var;
use hashbrown::HashMap;

use circuits::circuit::Circuit;

/// Analyzes the circuit and writes the classified logup-sum summands to `summands.txt`.
pub fn analyze(_circuit: &Circuit, _debug_info: &HashMap<String, Var>) {
    // TODO(lior): complete the test.
}
