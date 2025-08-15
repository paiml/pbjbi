pub mod deterministic;
pub mod probability;
pub mod quality;
pub mod statistics;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PbjbiError {
    #[error("Statistical error: {0}")]
    Statistical(String),

    #[error("Determinism violation: {0}")]
    DeterminismViolation(String),

    #[error("Quality gate failed: {0}")]
    QualityGateFailed(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Numerical error: {0}")]
    Numerical(String),
}

pub type Result<T> = std::result::Result<T, PbjbiError>;
