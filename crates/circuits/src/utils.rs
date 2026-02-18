use crate::{
    context::{Context, Var},
    eval,
    ivalue::IValue,
    ops::sub,
};

/// Implements a multiplexer.
/// Given a vector `values` and an index (represented in its bit decomposition `index_bits`)
/// returns a new variable equal to `values[index]`. It adds `3 * (n - 1) + log_2(n)` gates,
/// where `n` is `values.len()`.
pub fn select_by_index<Value: IValue>(
    context: &mut Context<Value>,
    values: &[Var],
    index_bits: &[Var],
) -> Var {
    assert!(values.len().is_power_of_two());
    assert_eq!(values.len(), 1 << index_bits.len());

    let one = context.one();
    let mut layer = values.to_vec();

    for &bit in index_bits {
        let one_minus_bit = sub(context, one, bit);
        let mut next = Vec::with_capacity(layer.len() / 2);
        for pair in layer.chunks_exact(2) {
            let [left, right] = pair.try_into().unwrap();
            next.push(eval!(context, ((one_minus_bit) * (left)) + ((bit) * (right))));
        }
        layer = next;
    }
    layer[0]
}
