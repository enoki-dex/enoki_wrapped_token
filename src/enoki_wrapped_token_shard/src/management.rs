use std::cell::RefCell;

use candid::{candid_method, types::number::Nat, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

pub fn assert_is_manager_contract() -> Result<()> {
    if MANAGER_CONTRACT_DATA.with(|s| s.borrow().manager_contract) == ic_cdk::caller() {
        Ok(())
    } else {
        Err(TxError::Unauthorized)
    }
}

pub fn assert_is_owner() -> Result<()> {
    if MANAGER_CONTRACT_DATA.with(|s| s.borrow().owner) == ic_cdk::caller() {
        Ok(())
    } else {
        Err(TxError::Unauthorized)
    }
}

#[derive(Deserialize, CandidType, Clone, Debug)]
struct ManagerContractData {
    pub owner: Principal,
    pub manager_contract: Principal,
    pub fee: Nat,
}

impl Default for ManagerContractData {
    fn default() -> Self {
        Self {
            owner: Principal::anonymous(),
            manager_contract: Principal::anonymous(),
            fee: Default::default(),
        }
    }
}

pub fn get_fee() -> Nat {
    MANAGER_CONTRACT_DATA.with(|d| d.borrow().fee.clone())
}

thread_local! {
    static MANAGER_CONTRACT_DATA: RefCell<ManagerContractData> = RefCell::new(ManagerContractData::default());
}
