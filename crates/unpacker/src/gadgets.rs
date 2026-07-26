//! Shared in-circuit gadgets over guessed u32 words and [`HashValue`]s, used by the unpacker and
//! Patricia circuits.

use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{Guess, add, eq, guess, mul};
use circuits::wrappers::U32Wrapper;
use stwo::core::fields::qm31::QM31;

/// Selects between two hash values word-by-word:
/// `select(selector, if_zero, if_one) = if_zero + selector * (if_one - if_zero)`.
///
/// Assumes `selector` is `0` or `1`.
pub fn select_hash<Value: IValue>(
    context: &mut Context<Value>,
    selector: Var,
    if_zero: &HashValue<Var>,
    if_one: &HashValue<Var>,
) -> HashValue<Var> {
    HashValue(std::array::from_fn(|i| {
        let diff = eval!(context, (*if_one[i].get()) - (*if_zero[i].get()));
        let result = eval!(context, (*if_zero[i].get()) + ((selector) * (diff)));
        U32Wrapper::new_unsafe(result)
    }))
}

/// Reads the concrete value of a [`HashValue<Var>`] back out of the context.
pub fn hash_value_of<Value: IValue>(
    context: &Context<Value>,
    hash: &HashValue<Var>,
) -> HashValue<Value> {
    HashValue(std::array::from_fn(|i| U32Wrapper::new_unsafe(context.get(*hash[i].get()))))
}

/// Guesses a fresh all-zero hash as witness — every word is a freshly guessed variable holding
/// zero (range-constrained to a valid `u32` like any other [`HashValue`] guess).
///
/// Crucially these are *distinct* variables, not the canonical zero constant (`Var { idx: 0 }`).
/// [`add`]/[`mul`] constant-fold away any operand that *is* the canonical zero, so building
/// padding from the zero constant would emit fewer gates than a real (non-zero) slot and make the
/// gate count depend on the witness. Guessing zeros instead keeps padding slots structurally
/// identical to real slots, so a fixed-topology circuit stays fixed.
pub fn guess_zero_hash<Value: IValue>(context: &mut Context<Value>) -> HashValue<Var> {
    let zero: HashValue<Value> =
        HashValue(std::array::from_fn(|_| U32Wrapper::new_unsafe(Value::pack_u32(0))));
    zero.guess(context)
}

/// Guesses a concrete [`HashValue<QM31>`] as witness in the `Value` context, lifting each word's
/// `QM31` value through [`IValue::from_qm31`] (a no-op for a `QM31` context; discarded for a value-
/// less topology build). Each word is range-constrained like any other [`HashValue`] guess.
pub fn guess_hash_value<Value: IValue>(
    context: &mut Context<Value>,
    hash: &HashValue<QM31>,
) -> HashValue<Var> {
    let value: HashValue<Value> = HashValue(std::array::from_fn(|i| {
        U32Wrapper::new_unsafe(Value::from_qm31(*hash[i].get()))
    }));
    value.guess(context)
}

/// Returns a `0/1` selector variable that is `1` iff every word of `words` is zero.
///
/// The words are summed into a single field element `acc`. Because every word of a guessed
/// [`HashValue`] is range-constrained to a valid `u32` packing `(low_u16, high_u16, 0, 0)` (see
/// [`HashValue::guess`]), each coordinate of the sum stays below `words.len() · 2^16`, which must
/// stay below `M31::P` for the sum not to wrap — hence the `words.len() < 2^15` assertion. Then
/// `acc == 0` (in `QM31`) iff every word is zero.
///
/// `is_zero` is then pinned to `[acc == 0]` by the standard is-zero gadget with witness
/// `inv_or_zero`:
/// * `acc * is_zero == 0` forces `is_zero` to be false (`0`) whenever `acc != 0`;
/// * `acc * inv_or_zero + is_zero == 1` forces `is_zero` to be true (`1`) whenever `acc == 0`, and
///   otherwise requires `inv_or_zero = 1/acc`.
///
/// Together these uniquely determine `is_zero ∈ {0, 1}` as a deterministic function of `words`, so
/// selection logic built on it cannot be steered by a malicious prover.
pub fn is_zero_words<Value: IValue>(
    context: &mut Context<Value>,
    words: &[U32Wrapper<Var>],
) -> Var {
    assert!(!words.is_empty(), "is_zero_words requires at least one word");
    assert!(words.len() < 1 << 15, "word sum could wrap M31::P for {} words", words.len());
    let zero = context.zero();
    let one = context.one();

    let acc = words.iter().skip(1).fold(*words[0].get(), |acc, w| add(context, acc, *w.get()));

    // Witness values for the is-zero gadget.
    let acc_val = context.get(acc);
    let is_zero = acc_val == Value::from_qm31(qm31_from_u32s(0, 0, 0, 0));
    let (is_zero_val, inv_or_zero_val) = if is_zero {
        (Value::from_qm31(qm31_from_u32s(1, 0, 0, 0)), Value::from_qm31(qm31_from_u32s(0, 0, 0, 0)))
    } else {
        (
            Value::from_qm31(qm31_from_u32s(0, 0, 0, 0)),
            Value::from_qm31(qm31_from_u32s(1, 0, 0, 0)) / acc_val,
        )
    };
    let is_zero = guess(context, is_zero_val);
    let inv_or_zero = guess(context, inv_or_zero_val);

    // acc * is_zero == 0
    let acc_is_zero = mul(context, acc, is_zero);
    eq(context, acc_is_zero, zero);
    // acc * inv_or_zero + is_zero == 1
    let acc_inv_or_zero = mul(context, acc, inv_or_zero);
    let acc_inv_or_zero_plus_is_zero = add(context, acc_inv_or_zero, is_zero);
    eq(context, acc_inv_or_zero_plus_is_zero, one);

    is_zero
}
