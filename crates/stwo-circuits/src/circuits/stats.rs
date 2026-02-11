#[derive(Debug, Default, PartialEq)]
pub struct Stats {
    pub equals: usize,
    pub add: usize,
    pub sub: usize,
    pub mul: usize,
    /// The number of divisions. Note that each division also increments [Self::mul],
    /// [Self::guess] and [Self::equals] by 1.
    pub div: usize,
    pub pointwise_mul: usize,
    pub guess: usize,
    pub blake_updates: usize,
    pub outputs: usize,
}
