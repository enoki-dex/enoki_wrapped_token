extern crate core;

#[allow(unused_imports)]
use candid::{candid_method, Nat, Principal};
use ic_cdk_macros::*;

#[allow(unused_imports)]
use enoki_wrapped_token_shared::types::{Result, ShardedTransferNotification};

use crate::management::{assert_is_owner, ManagerContractData};

mod balances;
mod fees;
mod interfaces;
mod management;
mod mint;
mod stable;
mod upgrade;

#[init]
#[candid_method(init)]
fn init() {
    management::init_manager_data(ManagerContractData {
        owner: ic_cdk::caller(),
        manager_contract: Principal::anonymous(),
        fee: Default::default(),
        underlying_token: Principal::anonymous(),
        sibling_shards: Default::default(),
        deploy_time: ic_cdk::api::time(),
    });
}

#[update(name = "finishInit")]
#[candid_method(update, rename = "finishInit")]
fn finish_init(manager_contract: Principal, underlying_token: Principal) {
    assert_is_owner().unwrap();
    management::init_manager_and_token(manager_contract, underlying_token);
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
