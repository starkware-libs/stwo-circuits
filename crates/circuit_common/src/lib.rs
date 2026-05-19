pub mod component_utils;
pub mod finalize;
pub mod order_hash_map;
pub mod preprocessed;

// TODO(Anat): Take from somewhere stable.
pub const N_LANES: usize = 16;
/// The number of reserved variables in a verifier circuit.
pub const N_RESERVED: usize = 2;

#[derive(Debug, PartialEq)]
pub struct CircuitParams {
    pub trace_log_size: u32,
    pub first_permutation_row: usize,
    pub output_addresses: Vec<usize>,
}

pub struct Qm31OpsTraceGenerator {
    pub first_permutation_row: usize,
}
