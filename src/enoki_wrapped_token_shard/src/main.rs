use candid::{candid_method, Func, Principal, types::number::Nat};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::Result;

use crate::management::ManagerContractData;

mod balances;
mod fees;
mod interfaces;
mod management;
mod mint;
mod upgrade;
mod stable;

#[init]
#[candid_method(init)]
fn init(owner: Principal, manager_contract: Principal, underlying_token: Principal, fee: Nat) {
    management::init_manager_data(ManagerContractData {
        owner,
        manager_contract,
        fee,
        underlying_token,
    });
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
