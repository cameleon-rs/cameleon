pub mod memory;

use thiserror::Error;

#[derive(Debug, Error)]
enum EmulatorError {
    #[error("access to illegal or not existed memory location")]
    MemoryAccessViolation,
}

type Result<T> = std::result::Result<T, EmulatorError>;
