use circuits::context::Context;
use circuits::ivalue::IValue;

/// Constructs all constants via arithmetic gates, replacing the old guess+hash approach.
/// Called during finalize, before padding.
pub fn finalize_constants(_context: &mut Context<impl IValue>) {
    todo!("implement constant construction")
}
