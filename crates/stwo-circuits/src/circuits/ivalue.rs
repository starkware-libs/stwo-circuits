use stwo::core::fields::cm31::CM31;
use stwo::core::fields::qm31::QM31;

pub fn qm31_from_u32s(a: u32, b: u32, c: u32, d: u32) -> QM31 {
    QM31(CM31(a.into(), b.into()), CM31(c.into(), d.into()))
}

/// Represents a value that can be used in a [Circuit].
///
/// We use [QM31] for a circuit with concrete values and [NoValue] for a circuit with no
/// concrete values.
pub trait IValue:
    Sized
    + Copy
    + Clone
    + std::fmt::Debug
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
{
    fn from_qm31(value: QM31) -> Self;
}

impl IValue for QM31 {
    /// Constructs an [IValue] from the given [QM31].
    fn from_qm31(value: QM31) -> Self {
        value
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoValue;

impl IValue for NoValue {
    fn from_qm31(_: QM31) -> Self {
        Self
    }
}

impl std::ops::Add for NoValue {
    type Output = NoValue;
    fn add(self, _: NoValue) -> NoValue {
        Self
    }
}
impl std::ops::Sub for NoValue {
    type Output = NoValue;
    fn sub(self, _: NoValue) -> NoValue {
        Self
    }
}
impl std::ops::Mul for NoValue {
    type Output = NoValue;
    fn mul(self, _: NoValue) -> NoValue {
        Self
    }
}
impl std::ops::Div for NoValue {
    type Output = NoValue;
    fn div(self, _: NoValue) -> NoValue {
        Self
    }
}
