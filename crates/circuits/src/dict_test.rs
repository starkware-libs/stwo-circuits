use num_traits::Zero;
use stwo::core::fields::qm31::QM31;

use crate::context::{Context, TraceContext, Var};
use crate::dict::{Dict, DictAccess};
use crate::ivalue::{IValue, NoValue, qm31_from_u32s};
use crate::ops::{Guess, output};
use crate::wrappers::M31Wrapper;

/// Guesses an `M31` input variable with the given value.
fn m31_var<Value: IValue>(context: &mut Context<Value>, value: u32) -> M31Wrapper<Var> {
    M31Wrapper::new_unsafe(Value::from_qm31(qm31_from_u32s(value, 0, 0, 0))).guess(context)
}

/// Marks all the variables of the squashed dict as outputs.
fn output_squashed<Value: IValue>(context: &mut Context<Value>, squashed: &[DictAccess]) {
    for access in squashed {
        output(context, *access.key.get());
        output(context, *access.prev_value.get());
        output(context, *access.new_value.get());
    }
}

#[test]
fn test_dict_single_key() {
    let mut context = TraceContext::default();
    context.enable_assert_eq_on_eval();
    let mut dict = Dict::new(QM31::zero());

    let key = m31_var(&mut context, 7);
    let five = m31_var(&mut context, 5);
    let nine = m31_var(&mut context, 9);

    dict.write(&mut context, key.clone(), five);
    let value = dict.read(&mut context, key.clone());
    assert_eq!(context.get(*value.get()), 5.into());
    dict.update(key, value, nine);

    let squashed = dict.squash(&mut context, 1);
    assert_eq!(squashed.len(), 1);
    assert_eq!(context.get(*squashed[0].key.get()), 7.into());
    assert_eq!(context.get(*squashed[0].prev_value.get()), QM31::zero());
    assert_eq!(context.get(*squashed[0].new_value.get()), 9.into());

    output_squashed(&mut context, &squashed);
    let context = context.finalize(true);
    context.validate_circuit();
    context.circuit().check_yields();
}

#[test]
fn test_dict_multiple_keys() {
    let mut context = TraceContext::default();
    context.enable_assert_eq_on_eval();
    let mut dict = Dict::new(QM31::zero());

    let key12 = m31_var(&mut context, 12);
    let key3 = m31_var(&mut context, 3);
    let key7 = m31_var(&mut context, 7);

    // Interleave accesses to different keys.
    let v100 = m31_var(&mut context, 100);
    let v200 = m31_var(&mut context, 200);
    let v300 = m31_var(&mut context, 300);
    dict.write(&mut context, key12.clone(), v100);
    dict.write(&mut context, key3.clone(), v200);
    let key12_value = dict.read(&mut context, key12);
    assert_eq!(context.get(*key12_value.get()), 100.into());
    dict.write(&mut context, key7.clone(), v300.clone());
    let key3_value = dict.read(&mut context, key3.clone());
    assert_eq!(context.get(*key3_value.get()), 200.into());
    dict.update(key3, key3_value, v300);
    let key7_value = dict.read(&mut context, key7);
    assert_eq!(context.get(*key7_value.get()), 300.into());

    let squashed = dict.squash(&mut context, 3);
    // The squashed dict is sorted by key and summarizes the first and last value of each key.
    let expected = [(3, 0, 300), (7, 0, 300), (12, 0, 100)];
    assert_eq!(squashed.len(), expected.len());
    for (access, (key, prev_value, new_value)) in squashed.iter().zip(expected) {
        assert_eq!(context.get(*access.key.get()), key.into());
        assert_eq!(context.get(*access.prev_value.get()), prev_value.into());
        assert_eq!(context.get(*access.new_value.get()), new_value.into());
    }

    output_squashed(&mut context, &squashed);
    let context = context.finalize(true);
    context.validate_circuit();
    context.circuit().check_yields();
}

#[test]
fn test_dict_read_default_value() {
    let mut context = TraceContext::default();
    context.enable_assert_eq_on_eval();
    let mut dict = Dict::new(QM31::from(42));

    let key = m31_var(&mut context, 5);
    let first_read = dict.read(&mut context, key.clone());
    assert_eq!(context.get(*first_read.get()), 42.into());
    let second_read = dict.read(&mut context, key);
    assert_eq!(context.get(*second_read.get()), 42.into());

    let squashed = dict.squash(&mut context, 1);
    // Reads only: the squashed entry has prev_value == new_value == default.
    assert_eq!(context.get(*squashed[0].prev_value.get()), 42.into());
    assert_eq!(context.get(*squashed[0].new_value.get()), 42.into());

    output_squashed(&mut context, &squashed);
    let context = context.finalize(true);
    context.validate_circuit();
}

#[test]
fn test_dict_wrong_update_prev_value() {
    let mut context = TraceContext::default();
    let mut dict = Dict::new(QM31::zero());

    let key = m31_var(&mut context, 7);
    let five = m31_var(&mut context, 5);
    let wrong_prev = m31_var(&mut context, 4);
    let six = m31_var(&mut context, 6);

    dict.write(&mut context, key.clone(), five);
    // The current value of the key is 5, not 4 — the chain condition must fail.
    dict.update(key, wrong_prev, six);
    dict.squash(&mut context, 1);

    assert!(!context.is_circuit_valid());
}

#[test]
#[should_panic(expected = "n_keys does not match the accesses")]
fn test_dict_wrong_n_keys() {
    let mut context = TraceContext::default();
    let mut dict = Dict::new(QM31::zero());

    let key1 = m31_var(&mut context, 1);
    let key2 = m31_var(&mut context, 2);
    let value = m31_var(&mut context, 5);
    dict.write(&mut context, key1, value.clone());
    dict.write(&mut context, key2, value);
    dict.squash(&mut context, 1);
}

#[test]
fn test_dict_tampered_witness() {
    let mut context = TraceContext::default();
    let mut dict = Dict::new(QM31::zero());

    let key1 = m31_var(&mut context, 1);
    let key2 = m31_var(&mut context, 2);
    let v5 = m31_var(&mut context, 5);
    let v6 = m31_var(&mut context, 6);
    dict.write(&mut context, key1.clone(), v5);
    dict.write(&mut context, key2, v6);
    let value = dict.read(&mut context, key1);
    dict.squash(&mut context, 2);
    context.validate_circuit();

    // Tampering with the value returned by the read must invalidate the circuit.
    let mut values = context.values().clone();
    values[value.get().idx] = 999.into();
    assert!(context.circuit.check(&values).is_err());
}

/// Builds the same dict circuit generically, to check that the topology does not depend on the
/// concrete values.
fn build_dict_context<Value: IValue>() -> Context<Value> {
    let mut context = Context::<Value>::default();
    let mut dict = Dict::new(Value::from_qm31(QM31::zero()));

    let key1 = m31_var(&mut context, 1);
    let key2 = m31_var(&mut context, 2);
    let v5 = m31_var(&mut context, 5);
    let v6 = m31_var(&mut context, 6);
    dict.write(&mut context, key1.clone(), v5);
    dict.write(&mut context, key2, v6.clone());
    let value = dict.read(&mut context, key1.clone());
    dict.update(key1, value, v6);

    let squashed = dict.squash(&mut context, 2);
    output_squashed(&mut context, &squashed);
    context
}

#[test]
fn test_dict_no_value_topology_matches() {
    let trace_context = build_dict_context::<QM31>();
    trace_context.validate_circuit();
    let trace_context = trace_context.finalize(true);
    trace_context.circuit().check_yields();

    let no_value_context = build_dict_context::<NoValue>().finalize(true);
    no_value_context.circuit().check_yields();

    assert!(trace_context.circuit() == no_value_context.circuit());
}
