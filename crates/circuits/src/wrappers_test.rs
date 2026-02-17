use expect_test::expect;

use crate::context::Context;
use crate::ivalue::NoValue;
use crate::ops::Guess;
use crate::wrappers::M31Wrapper;

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
        [3] = [2] x [1]

    "#]]
    .assert_debug_eq(&context.circuit);
}
