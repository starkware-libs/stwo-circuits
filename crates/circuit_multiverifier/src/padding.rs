use circuits::blake::{blake_g_gate, m31_to_u32, triple_xor};
use circuits::context::Context;
use circuits::ivalue::IValue;
use circuits::ops::{add, eq};

use crate::test_utils::ComponentLogSizes;

/// Pads the components of `context` so that each one reaches the corresponding target log size.
pub fn pad_components_to_target_log_sizes<Value: IValue>(
    context: &mut Context<Value>,
    ComponentLogSizes {
        eq: target_eq_log,
        qm31_ops: target_qm31_ops_log,
        m31_to_u32: target_m31_to_u32_log,
        triple_xor: target_triple_xor_log,
        blake_g_gate: target_blake_g_gate_log,
    }: ComponentLogSizes,
) {
    let zero = context.zero();

    let current_eq = context.circuit.eq.len();
    let target_eq = 1usize << target_eq_log;
    assert!(target_eq >= current_eq);
    for _ in current_eq..target_eq {
        eq(context, zero, zero);
    }

    let current_qm31_ops = context.circuit.add.len()
        + context.circuit.sub.len()
        + context.circuit.mul.len()
        + context.circuit.pointwise_mul.len()
        + context
            .circuit
            .permutation
            .iter()
            .map(|p| p.inputs.len() + p.outputs.len())
            .sum::<usize>();
    let target_qm31_ops = 1 << target_qm31_ops_log;
    assert!(target_qm31_ops >= current_qm31_ops);
    for _ in current_qm31_ops..target_qm31_ops {
        add(context, zero, zero);
    }

    let current_triple_xor = context.circuit.triple_xor.len();
    let target_triple_xor = 1 << target_triple_xor_log;
    assert!(target_triple_xor >= current_triple_xor);
    for _ in current_triple_xor..target_triple_xor {
        triple_xor(context, zero, zero, zero);
    }

    let current_m31_to_u32 = context.circuit.m31_to_u32.len();
    let target_m31_to_u32 = 1 << target_m31_to_u32_log;
    assert!(target_m31_to_u32 >= current_m31_to_u32);
    for _ in current_m31_to_u32..target_m31_to_u32 {
        m31_to_u32(context, zero);
    }

    let current_blake_g_gate = context.circuit.blake_g_gate.len();
    let target_blake_g_gate = 1 << target_blake_g_gate_log;
    assert!(target_blake_g_gate >= current_blake_g_gate);
    for _ in current_blake_g_gate..target_blake_g_gate {
        blake_g_gate(context, zero, zero, zero, zero, zero, zero);
    }
}
