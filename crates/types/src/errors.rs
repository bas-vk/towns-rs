use alloy_primitives::FixedBytes;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TownsError {
    #[error("invalid {0}")]
    InvalidArgument(&'static str),
    #[error("invalid {0} value {1}")]
    InvalidArgumentWithValue(&'static str, String),
    #[error("invalid stream updated event {0} tx={1} log_idx={2}")]
    InvalidStreamUpdatedEvent(String, FixedBytes<32>, u64),
    #[error("invalid previous miniblock hash exp{0} got{1}")]
    InvalidPreviousMiniblockHash(FixedBytes<32>, FixedBytes<32>),
    #[error("invalid previous miniblock num exp{0} got{1}")]
    InvalidPreviousMiniblockNum(u64, u64),
    #[error("not found")]
    NotFound,
    #[error("contract call failed")]
    ContractCallFailed(alloy_contract::Error),
}
