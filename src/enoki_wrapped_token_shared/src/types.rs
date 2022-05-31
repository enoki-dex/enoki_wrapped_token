use std::string::String;

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::call::RejectionCode;

#[derive(CandidType, Debug, Deserialize)]
pub enum TxError {
    InsufficientBalance,
    Unauthorized,
    AccountDoesNotExist,
    AccountAlreadyExists,
    TransferValueTooSmall,
    TransferCallbackError(String),
    UnderlyingTransferFailure,
    Other(String),
}

impl From<(RejectionCode, String)> for TxError {
    fn from(err: (RejectionCode, String)) -> Self {
        Self::TransferCallbackError(format!("Error in callback (code {:?}): {}", err.0, err.1))
    }
}

pub type Result<T> = std::result::Result<T, TxError>;

pub struct UserAccount {
    pub main_account: Principal,
    pub shard_account: Option<Principal>,
}
