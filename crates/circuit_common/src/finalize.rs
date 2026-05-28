use crate::N_LANES;
use circuits::blake::blake_g_gate;
use circuits::circuit::Circuit;
use circuits::context::Context;
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::eq;
use rand_chacha::rand_core::{RngCore, SeedableRng};

fn pad_qm31_ops(context: &mut Context<impl IValue>, target: usize) {
    let qm31_ops_n_rows = qm31_ops_n_rows(&context.circuit);
    assert!(qm31_ops_n_rows <= target);
    let zero = context.zero();
    for _ in qm31_ops_n_rows..target {
        eval!(context, (zero) + (zero));
    }
}

fn pad_eq(context: &mut Context<impl IValue>, target: usize) {
    let eq_n_rows = context.circuit.eq.len();
    assert!(eq_n_rows <= target);
    let zero = context.zero();
    for _ in eq_n_rows..target {
        circuits::ops::eq(context, zero, zero);
    }
}

fn pad_triple_xor(context: &mut Context<impl IValue>, target: usize) {
    let n_rows = context.circuit.triple_xor.len();
    assert!(n_rows <= target);
    let zero = context.zero();
    for _ in n_rows..target {
        circuits::blake::triple_xor(context, zero, zero, zero);
    }
}

fn pad_m31_to_u32(context: &mut Context<impl IValue>, target: usize) {
    let n_rows = context.circuit.m31_to_u32.len();
    assert!(n_rows <= target);
    let zero = context.zero();
    for _ in n_rows..target {
        circuits::blake::m31_to_u32(context, zero);
    }
}

fn pad_blake_g_gate(context: &mut Context<impl IValue>, target: usize) {
    let n_rows = context.circuit.blake_g_gate.len();
    assert!(n_rows <= target);
    let zero = context.zero();
    for _ in n_rows..target {
        blake_g_gate(context, zero, zero, zero, zero, zero, zero);
    }
}

/// Pads the components to the next power of two of each component.
/// Appends gates to the context so that the resulting AIR components have length equal to a power
/// of two.
// TODO(Gali): Have it under a trait.
// TODO(Ilya): Make it pub(crate).
pub fn finalize_context(context: &mut Context<impl IValue>) {
    // Padding the components to a power of two.
    let padded_sizes = compute_padded_sizes(context);
    pad_to_targets(context, padded_sizes);
}

/// Pads each component to its target size by appending trivial gates. The target sizes are passed
/// in parameter `targets`.
pub fn pad_to_targets(context: &mut Context<impl IValue>, targets: ComponentSizes) {
    pad_eq(context, targets.eq);
    pad_qm31_ops(context, targets.qm31_ops);
    pad_triple_xor(context, targets.triple_xor);
    pad_m31_to_u32(context, targets.m31_to_u32);
    pad_blake_g_gate(context, targets.blake_g_gate);
}

/// Adds ZK blinding to the circuit by adding random values to the qm31_ops and eq components.
pub fn add_zk_blinding(context: &mut Context<impl IValue>, seed_bytes: [u8; 32], n_padding: usize) {
    let mut rng = rand_chacha::ChaCha20Rng::from_seed(seed_bytes);
    for _ in 0..n_padding {
        // Note that we don't use the guess function here because we want to be able to run this
        // function after finalize_guessed_vars.
        let value1 = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
        let var1 = context.new_var(IValue::from_qm31(value1));
        context.circuit.add.push(circuits::circuit::Add {
            in0: var1.idx,
            in1: context.zero().idx,
            out: var1.idx,
        });
        let value2 = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
        let var2 = context.new_var(IValue::from_qm31(value2));
        context.circuit.add.push(circuits::circuit::Add {
            in0: context.zero().idx,
            in1: var2.idx,
            out: var2.idx,
        });
        eval!(context, (var1) + (var2));

        let value3 = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
        let var3 = context.new_var(IValue::from_qm31(value3));
        context.circuit.add.push(circuits::circuit::Add {
            in0: var3.idx,
            in1: context.zero().idx,
            out: var3.idx,
        });
        eq(context, var3, var3);
    }
}

/// Helper struct containing the sizes of the AIR components of a circuit.
#[derive(Clone)]
pub struct ComponentSizes {
    pub eq: usize,
    pub qm31_ops: usize,
    pub m31_to_u32: usize,
    pub triple_xor: usize,
    pub blake_g_gate: usize,
}

// Prints the sizes and the log base 2 rounded up.
impl std::fmt::Display for ComponentSizes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "eq={:>2} (log: {}),  qm31_ops={:>2} (log: {}), m31_to_u32={:>2} (log: {}), triple_xor={:>2} (log: {}), blake_g_gate={:>2} (log: {})",
            self.eq, self.eq.next_power_of_two().ilog2(),
            self.qm31_ops, self.qm31_ops.next_power_of_two().ilog2(),
            self.m31_to_u32, self.m31_to_u32.next_power_of_two().ilog2(),
            self.triple_xor, self.triple_xor.next_power_of_two().ilog2(),
            self.blake_g_gate, self.blake_g_gate.next_power_of_two().ilog2(),
        ))
    }
}

pub fn compute_padded_sizes(context: &Context<impl IValue>) -> ComponentSizes {
    let circuit = &context.circuit;
    let qm31_ops_n_rows = qm31_ops_n_rows(&context.circuit);
    ComponentSizes {
        eq: padded_size(circuit.eq.len()),
        qm31_ops: padded_size(qm31_ops_n_rows),
        m31_to_u32: padded_size(circuit.m31_to_u32.len()),
        triple_xor: padded_size(circuit.triple_xor.len()),
        blake_g_gate: padded_size(circuit.blake_g_gate.len()),
    }
}

/// Computes the number of rows in the qm31_ops AIR component.
fn qm31_ops_n_rows(circuit: &Circuit) -> usize {
    circuit.add.len()
        + circuit.sub.len()
        + circuit.mul.len()
        + circuit.pointwise_mul.len()
        + circuit.permutation.iter().map(|p| p.inputs.len() + p.outputs.len()).sum::<usize>()
}

fn padded_size(n_rows: usize) -> usize {
    std::cmp::max(n_rows.next_power_of_two(), N_LANES)
}
