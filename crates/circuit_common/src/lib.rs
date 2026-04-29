pub mod finalize;
pub mod preprocessed;

// TODO(Anat): Take from somewhere stable.
pub const N_LANES: usize = 16;

#[derive(Debug, PartialEq)]
pub struct CircuitParams {
    pub trace_log_size: u32,
    pub first_permutation_row: usize,
    pub n_blake_gates: usize,
    /// Total number of blake compression blocks across all blake gates (unpadded). Equals
    /// `sum(gate.input.len())`.
    pub n_blake_compress: usize,
    pub output_addresses: Vec<usize>,
}

pub struct Qm31OpsTraceGenerator {
    pub first_permutation_row: usize,
}
