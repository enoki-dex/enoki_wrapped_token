use std::cell::RefCell;
use std::collections::HashSet;

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

pub fn assert_is_sibling(id: &Principal) -> Result<()> {
    if MANAGER_CONTRACT_DATA.with(|s| s.borrow().sibling_shards.contains(id)) {
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
    pub sibling_shards: HashSet<Principal>,
    pub deploy_time: u64,
}

impl Default for ManagerContractData {
    fn default() -> Self {
        Self {
            owner: Principal::anonymous(),
            manager_contract: Principal::anonymous(),
            fee: Default::default(),
            underlying_token: Principal::anonymous(),
            sibling_shards: Default::default(),
            deploy_time: 0,
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

#[query(name = "whoami")]
#[candid_method(query, rename = "whoami")]
fn who_am_i() -> Principal {
    ic_cdk::caller()
}

#[query(name = "getManagementDetails")]
#[candid_method(query, rename = "getManagementDetails")]
fn get_management_details() -> ManagerContractData {
    MANAGER_CONTRACT_DATA.with(|d| d.borrow().clone())
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

#[update(name = "setFee")]
#[candid_method(update, rename = "setFee")]
fn set_fee(new_fee: Nat) -> Result<()> {
    MANAGER_CONTRACT_DATA.with(|d| {
        let mut data = d.borrow_mut();
        if ic_cdk::caller() == data.manager_contract {
            data.fee = new_fee;
            Ok(())
        } else {
            Err(TxError::Unauthorized)
        }
    })
}

#[update(name = "initShard")]
#[candid_method(update, rename = "initShard")]
fn init_shard(underlying_token: Principal, sibling_shards: Vec<Principal>, fee: Nat) -> Result<()> {
    MANAGER_CONTRACT_DATA.with(|d| {
        let mut data = d.borrow_mut();
        if ic_cdk::caller() == data.manager_contract {
            if data.underlying_token != underlying_token {
                return Err(TxError::Other("Incompatible shard".to_string()));
            }
            for shard in sibling_shards {
                data.sibling_shards.insert(shard);
            }
            data.fee = fee;
            Ok(())
        } else {
            Err(TxError::Unauthorized)
        }
    })
}

#[update(name = "addSiblingShard")]
#[candid_method(update, rename = "addSiblingShard")]
fn add_sibling_shard(new_shard: Principal) -> Result<()> {
    MANAGER_CONTRACT_DATA.with(|d| {
        let mut data = d.borrow_mut();
        if ic_cdk::caller() == data.manager_contract {
            data.sibling_shards.insert(new_shard);
            Ok(())
        } else {
            Err(TxError::Unauthorized)
        }
    })
}

#[update(name = "removeSiblingShard")]
#[candid_method(update, rename = "removeSiblingShard")]
fn remove_sibling_shard(shard: Principal) -> Result<()> {
    MANAGER_CONTRACT_DATA.with(|d| {
        let mut data = d.borrow_mut();
        if ic_cdk::caller() == data.manager_contract {
            data.sibling_shards.remove(&shard);
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
