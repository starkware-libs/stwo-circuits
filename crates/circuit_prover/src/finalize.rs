use stwo::prover::backend::simd::m31::N_LANES;

use circuits::blake::{HashValue, blake};
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::output;

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

fn hash_constants(context: &mut Context<impl IValue>) -> HashValue<Var> {
    let constants: Vec<_> = context.constants().values().copied().collect();
    let n_bytes = constants.len() * 16;
    blake(context, &constants, n_bytes)
}

/// Finalizes the context by appending gates to the context for:
/// - Hashing the constants.
/// - Hashing the outputs.
/// - Padding the components to a power of two.
// TODO(Gali): Have it under a trait.
pub(crate) fn finalize_context(context: &mut Context<impl IValue>) {
    let HashValue(hash0, hash1) = hash_constants(context);
    // Add the hash of the constants to the outputs.
    // TODO(Leo): consider storing these values at a fixed address.
    output(context, hash0);
    output(context, hash1);
    // TODO(Gali): Hash the outputs (all variables that have no uses).

    // Padding the components to a power of two.
    pad_eq(context);
    pad_qm31_ops(context);
    // TODO(Gali): Pad blake gates.
}
