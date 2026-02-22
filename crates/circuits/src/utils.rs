use crate::{
    context::{Context, Var},
    eval,
    ivalue::IValue,
    ops::sub,
};

/// Implements a multiplexer.
/// Given a vector `values` and an index (represented in its bit decomposition `index_bits`)
/// returns a new variable equal to `values[index]`.
///
/// A multiplexer costs `3 * (n - 1) + log_2(n)` gates, where `n` is `values.len()`.
pub fn select_by_index<Value: IValue>(
    context: &mut Context<Value>,
    values: &[Var],
    index_bits: &[Var],
) -> Var {
    assert!(values.len().is_power_of_two());
    assert_eq!(values.len(), 1 << index_bits.len());

    let one = context.one();
    let mut layer = values.to_vec();
    let mut curr_layer_len = layer.len();

    for &bit in index_bits {
        let one_minus_bit = sub(context, one, bit);
        for i in (0..curr_layer_len).step_by(2) {
            let (left, right) = (layer[i], layer[i + 1]);
            layer[i >> 1] = eval!(context, ((one_minus_bit) * (left)) + ((bit) * (right)));
        }
        curr_layer_len >>= 1;
    }
    layer[0]
}
