use std::cell::RefCell;

use candid::{candid_method, CandidType, Deserialize, Principal, types::number::Nat};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

use crate::stable::StableManagerContractData;

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
pub struct ManagerContractData {
    pub owner: Principal,
    pub manager_contract: Principal,
    pub fee: Nat,
    pub underlying_token: Principal,
}

impl Default for ManagerContractData {
    fn default() -> Self {
        Self {
            owner: Principal::anonymous(),
            manager_contract: Principal::anonymous(),
            fee: Default::default(),
            underlying_token: Principal::anonymous(),
        }
    }
}

pub fn get_fee() -> Nat {
    MANAGER_CONTRACT_DATA.with(|d| d.borrow().fee.clone())
}

pub fn init_manager_data(data: ManagerContractData) {
    MANAGER_CONTRACT_DATA.with(|d| {
        *d.borrow_mut() = data;
    });
}

pub fn get_underlying() -> Principal {
    MANAGER_CONTRACT_DATA.with(|d| d.borrow().underlying_token)
}

thread_local! {
    static MANAGER_CONTRACT_DATA: RefCell<ManagerContractData> = RefCell::new(ManagerContractData::default());
}

#[query(name = "getOwner")]
#[candid_method(query, rename = "getOwner")]
fn get_owner() -> Principal {
    MANAGER_CONTRACT_DATA.with(|d| d.borrow().owner)
}

#[update(name = "setOwner")]
#[candid_method(update, rename = "setOwner")]
fn set_owner(new_owner: Principal) -> Result<()> {
    MANAGER_CONTRACT_DATA.with(|d| {
        let owner = &mut d.borrow_mut().owner;
        if ic_cdk::caller() == *owner {
            *owner = new_owner;
            Ok(())
        } else {
            Err(TxError::Unauthorized)
        }
    })
}

pub fn export_stable_storage() -> (StableManagerContractData, ) {
    let manager_data: StableManagerContractData = MANAGER_CONTRACT_DATA.with(|b| b.take()).into();
    (manager_data, )
}

pub fn import_stable_storage(manager_data: StableManagerContractData) {
    MANAGER_CONTRACT_DATA.with(|b| b.replace(manager_data.into()));
}
