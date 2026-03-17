#[derive(Debug, PartialEq, Eq)]
pub enum VerificationError {
    /// One or more proof fields have sizes inconsistent with the verification config.
    /// Covers OODS column counts, FRI layer/query structure, Merkle auth path lengths,
    /// claim field sizes, and eval domain sample dimensions.
    InvalidProofStructure,
    /// The proof is structurally valid but the circuit evaluation found it invalid.
    InvalidProof,
}

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProofStructure => write!(f, "Invalid proof structure"),
            Self::InvalidProof => write!(f, "Proof verification failed"),
        }
    }
}

impl std::error::Error for VerificationError {}
