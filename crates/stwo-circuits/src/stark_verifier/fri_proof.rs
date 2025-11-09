/// Represents the structure of a FRI proof.
pub struct FriConfig {
    pub log_trace_size: usize,
    pub log_evaluation_domain_size: usize,
    pub n_queries: usize,
    pub log_n_last_layer_coefs: usize,
}
