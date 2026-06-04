use crate::N_LANES;
use circuits::blake::{blake_g_gate, m31_to_u32, triple_xor};
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{add_into, eq};
use rand_chacha::rand_core::{RngCore, SeedableRng};

fn pad_qm31_ops(context: &mut Context<impl IValue>) {
    let qm31_ops_n_rows = context.circuit.n_qm31_ops_rows();

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

fn pad_triple_xor(context: &mut Context<impl IValue>) {
    let n_rows = context.circuit.triple_xor.len();
    let padded = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
    let zero = context.zero();
    for _ in n_rows..padded {
        circuits::blake::triple_xor(context, zero, zero, zero);
    }
}

fn pad_m31_to_u32(context: &mut Context<impl IValue>) {
    let n_rows = context.circuit.m31_to_u32.len();
    let padded = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
    let zero = context.zero();
    for _ in n_rows..padded {
        circuits::blake::m31_to_u32(context, zero);
    }
}

fn pad_blake_g_gate(context: &mut Context<impl IValue>) {
    let n_rows = context.circuit.blake_g_gate.len();
    let padded = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
    let zero = context.zero();
    for _ in n_rows..padded {
        blake_g_gate(context, zero, zero, zero, zero, zero, zero);
    }
}

/// Pad the components to a power of two.
/// Appends gates to the context so that the resulting AIR components have length equal to a power
/// of two.
// TODO(Gali): Have it under a trait.
// TODO(Ilya): Make it pub(crate).
pub fn finalize_context(context: &mut Context<impl IValue>) {
    // Padding the components to a power of two.
    pad_eq(context);
    pad_qm31_ops(context);
    pad_triple_xor(context);
    pad_m31_to_u32(context);
    pad_blake_g_gate(context);
}

/// Adds random values to the qm31_ops component via trivial.
/// the padding is
/// x + 0 = x
/// 0 + y = y
/// z - 0 = z
fn qm31_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let zero = context.zero();
    let value_x = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
    let x = context.new_var(IValue::from_qm31(value_x));
    add_into(context, x, zero, x);
    let value_y = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
    let y = context.new_var(IValue::from_qm31(value_y));
    add_into(context, zero, y, y);

    let value_z = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
    let z = context.new_var(IValue::from_qm31(value_z));
    add_into(context, zero, z, z);
}

/// Adds a random value to the eq component via a trivial `x == x` equality, blinding the eq trace.
///
/// Note that we don't use the guess function here because we want to be able to run this after
/// finalize_guessed_vars.
fn eq_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let zero = context.zero();
    let value = qm31_from_u32s(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
    let x = context.new_var(IValue::from_qm31(value));
    // yield x.
    add_into(context, x, zero, x);
    eq(context, x, x);
}

/// Creates a fresh variable holding a random `u32` value, encoded as `(low_u16, high_u16, 0, 0)`,
/// and yields it via a trivial `x + 0 = x` add gate so it can be used as a gate input.
///
/// Note that we don't use the guess function here because we want to be able to run this after
/// finalize_guessed_vars.
fn random_u32_var(context: &mut Context<impl IValue>, rng: &mut impl RngCore) -> Var {
    let zero = context.zero();
    let x = context.new_var(IValue::pack_u32(rng.next_u32()));
    add_into(context, x, zero, x);
    x
}

/// Creates a fresh variable holding a random `M31` value, encoded as `(x, 0, 0, 0)`, and yields it
/// via a trivial `x + 0 = x` add gate so it can be used as a gate input.
///
/// Note that we don't use the guess function here because we want to be able to run this after
/// finalize_guessed_vars.
fn random_m31_var(context: &mut Context<impl IValue>, rng: &mut impl RngCore) -> Var {
    let zero = context.zero();
    let x = context.new_var(IValue::from_qm31(qm31_from_u32s(rng.next_u32(), 0, 0, 0)));
    add_into(context, x, zero, x);
    x
}

/// Adds a random row to the triple_xor component, blinding its trace.
fn triple_xor_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let a = random_u32_var(context, rng);
    let b = random_u32_var(context, rng);
    let c = random_u32_var(context, rng);
    triple_xor(context, a, b, c);
}

/// Adds a random row to the m31_to_u32 component, blinding its trace.
fn m31_to_u32_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let input = random_m31_var(context, rng);
    m31_to_u32(context, input);
}

/// Adds a random row to the blake_g_gate component, blinding its trace.
fn blake_g_gate_zk_blinding(context: &mut Context<impl IValue>, rng: &mut impl RngCore) {
    let a = random_u32_var(context, rng);
    let b = random_u32_var(context, rng);
    let c = random_u32_var(context, rng);
    let d = random_u32_var(context, rng);
    let f0 = random_u32_var(context, rng);
    let f1 = random_u32_var(context, rng);
    blake_g_gate(context, a, b, c, d, f0, f1);
}

/// Adds ZK blinding to the circuit by adding random rows to all the witness components: qm31_ops,
/// eq, triple_xor, m31_to_u32, and blake_g_gate. The fresh input variables are yielded through
/// trivial `x + 0 = x` add gates in the qm31_ops component.
pub fn add_zk_blinding(context: &mut Context<impl IValue>, seed_bytes: [u8; 32], n_padding: usize) {
    let mut rng = rand_chacha::ChaCha20Rng::from_seed(seed_bytes);
    for _ in 0..n_padding {
        qm31_zk_blinding(context, &mut rng);
        eq_zk_blinding(context, &mut rng);
        triple_xor_zk_blinding(context, &mut rng);
        m31_to_u32_zk_blinding(context, &mut rng);
        blake_g_gate_zk_blinding(context, &mut rng);
    }
}
