use circuits::{
    blake::HashValue,
    context::{Context, Var},
    ivalue::IValue,
    ops::output,
};

/// Creates a fresh [`Context`] for a verifier circuit with two reserved variables at the front
/// (wires 3 and 4).
///
/// These two slots are placeholders for the output gates, which are filled in later (in all current
/// cases, the two slots will hold the two limbs of a `HashValue`). Reserving them up front
/// guarantees that the output wires are in fixed positions regardless of how the rest of the
/// circuit is built.
pub fn new_verifier_context<Value: IValue>() -> Context<Value> {
    let mut context = Context::default();
    context.reserve();
    context.reserve();
    context
}

/// Copies the two limbs of `hash` into the two reserved variables that had been reserved by
/// [`new_verifier_context`], and marks them as outputs.
///
/// Expects the context to have exactly two outstanding reservations. The two limbs of `hash` are
/// copied into those reserved variables via [`Context::copy_into_reserved`] (which adds `reserved =
/// limb + 0` gates to constrain them), and each reserved variable is then emitted as output.
///
/// Panics if the number of outstanding reservations is not exactly two.
pub fn copy_hash_into_reserved<Value: IValue>(context: &mut Context<Value>, hash: HashValue<Var>) {
    let [reserved0, reserved1] = context
        .reserved()
        .try_into()
        .expect("The verifier circuit should have exactly 2 reserved vars.");
    context.copy_into_reserved(reserved0, hash.0);
    context.copy_into_reserved(reserved1, hash.1);
    output(context, reserved0);
    output(context, reserved1);
}
