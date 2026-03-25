use crate::N_LANES;
use circuits::circuit::{BlakeGGate, M31ToU32Gate, TripleXorGate};
use circuits::context::Context;
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::eq;
use rand_chacha::rand_core::{RngCore, SeedableRng};

fn pad_qm31_ops(context: &mut Context<impl IValue>) {
    let qm31_ops_n_rows = context.circuit.add.len()
        + context.circuit.sub.len()
        + context.circuit.mul.len()
        + context.circuit.pointwise_mul.len()
        + context
            .circuit
            .permutation
            .iter()
            .map(|p| p.inputs.len() + p.outputs.len())
            .sum::<usize>();

    let qm31_padding =
        std::cmp::max(qm31_ops_n_rows.next_power_of_two(), N_LANES) - qm31_ops_n_rows;
    let zero = context.zero();
    for _ in 0..qm31_padding {
        eval!(context, (zero) + (zero));
    }
}

fn pad_eq(context: &mut Context<impl IValue>) {
    let eq_n_rows = context.circuit.eq.len();
    let eq_padding = std::cmp::max(eq_n_rows.next_power_of_two(), N_LANES) - eq_n_rows;
    let zero = context.zero();
    for _ in 0..eq_padding {
        circuits::ops::eq(context, zero, zero);
    }
}

fn pad_blake_g(context: &mut Context<impl IValue>) {
    let n_rows = context.circuit.blake_g.len();
    let padded = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
    let zero = context.zero();
    for _ in n_rows..padded {
        let out_a = context.new_var(IValue::from_qm31(0.into()));
        let out_b = context.new_var(IValue::from_qm31(0.into()));
        let out_c = context.new_var(IValue::from_qm31(0.into()));
        let out_d = context.new_var(IValue::from_qm31(0.into()));
        context.circuit.blake_g.push(BlakeGGate {
            a: zero.idx,
            b: zero.idx,
            c: zero.idx,
            d: zero.idx,
            m0: zero.idx,
            m1: zero.idx,
            out_a: out_a.idx,
            out_b: out_b.idx,
            out_c: out_c.idx,
            out_d: out_d.idx,
        });
    }
}

fn pad_m31_to_u32(context: &mut Context<impl IValue>) {
    let n_rows = context.circuit.m31_to_u32.len();
    let padded = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
    let zero = context.zero();
    for _ in n_rows..padded {
        // Pad with gates that split the zero constant (address 0) into (0, 0).
        let out = context.new_var(IValue::from_qm31(0.into()));
        context.circuit.m31_to_u32.push(M31ToU32Gate { input: zero.idx, out: out.idx });
    }
}

fn pad_triple_xor(context: &mut Context<impl IValue>) {
    let n_rows = context.circuit.triple_xor.len();
    let padded = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
    let zero = context.zero();
    for _ in n_rows..padded {
        // Pad with gates that XOR zeros: 0 ^ 0 ^ 0 = 0.
        let out = context.new_var(IValue::from_qm31(0.into()));
        context.circuit.triple_xor.push(TripleXorGate {
            a: zero.idx,
            b: zero.idx,
            c: zero.idx,
            out: out.idx,
        });
    }
}

/// Finalizes the context by padding the components to a power of two.
// TODO(Gali): Have it under a trait.
// TODO(Ilya): Make it pub(crate).
pub fn finalize_context(context: &mut Context<impl IValue>) {
    pad_eq(context);
    pad_qm31_ops(context);
    pad_blake_g(context);
    pad_m31_to_u32(context);
    pad_triple_xor(context);
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
