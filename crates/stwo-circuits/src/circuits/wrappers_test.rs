use expect_test::expect;

use crate::circuits::context::Context;
use crate::circuits::ivalue::NoValue;
use crate::circuits::ops::Guess;
use crate::circuits::wrappers::M31Wrapper;

#[test]
fn test_m31_wrapper_guess_circuit() {
    let mut context = Context::<NoValue>::default();
    let res = M31Wrapper::from(NoValue).guess(&mut context);
    expect!["M31([3])"].assert_eq(&format!("{res:?}"));
    expect![[r#"
        {
            (0 + 0i) + (0 + 0i)u: [0],
            (1 + 0i) + (0 + 0i)u: [1],
        }
    "#]]
    .assert_debug_eq(&context.constants());
    expect![[r#"
        [0] = [0] + [0]
        [1] = [1] + [0]
        [2] = [2] + [0]
        [3] = [2] x [1]

    "#]]
    .assert_debug_eq(&context.circuit);
}
