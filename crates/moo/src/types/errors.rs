use thiserror::Error;

#[derive(Error, Debug)]
pub enum MooError {
    #[error("Error parsing MOO file: {0}")]
    ParseError(String),
    #[error("Error writing MOO file: {0}")]
    WriteError(String),
    #[error("A compliant MOO file was not detected")]
    FileDetectionError,
    #[error("An unknown error occurred")]
    Unknown,
}
