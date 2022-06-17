extern crate core;

#[allow(unused_imports)]
use candid::{candid_method, Nat, Principal};
use ic_cdk_macros::*;

#[allow(unused_imports)]
use enoki_wrapped_token_shared::types::{Result, ShardedTransferNotification};

use crate::management::ManagerContractData;

mod balances;
mod fees;
mod interfaces;
mod management;
mod mint;
mod stable;
mod upgrade;

#[init]
#[candid_method(init)]
fn init(manager_contract: Principal, underlying_token: Principal) {
    management::init_manager_data(ManagerContractData {
        owner: ic_cdk::caller(),
        manager_contract,
        fee: Default::default(),
        underlying_token,
        sibling_shards: Default::default(),
        deploy_time: ic_cdk::api::time(),
    });
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
