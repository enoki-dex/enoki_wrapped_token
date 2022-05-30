use std::collections::HashMap;
use std::iter::FromIterator;
use std::string::String;

use candid::{candid_method, CandidType, Deserialize, Func, Principal, types::number::Nat};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

mod upgrade;
mod balances;
mod management;
mod fees;
mod mint;

#[init]
#[candid_method(init)]
fn init(
    owner: Principal,
    manager_contract: Principal,
) {
    todo!()
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
