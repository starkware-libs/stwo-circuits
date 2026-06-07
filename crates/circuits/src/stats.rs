#[derive(Debug, Default, PartialEq)]
pub struct Stats {
    pub equals: usize,
    pub add: usize,
    pub sub: usize,
    pub mul: usize,
    /// The number of inversions. Note that each inversion also increments [Self::mul],
    /// [Self::guess] and [Self::equals] by 1.
    pub inv: usize,
    pub pointwise_mul: usize,
    pub guess: usize,
    pub blake_updates: usize,
    /// The number of inputs to permutation gates.
    pub permutation_inputs: usize,
    pub outputs: usize,
    pub triple_xor: usize,
    pub m31_to_u32: usize,
}
