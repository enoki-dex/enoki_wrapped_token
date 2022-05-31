use std::string::String;

use candid::{candid_method, CandidType, Deserialize, Principal, types::number::Nat};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::Result;

use crate::management::Stats;
use crate::metadata::Metadata;

mod metadata;
mod shards;
mod management;
mod upgrade;
mod types;

#[init]
#[candid_method(init)]
fn init(
    logo: String,
    name: String,
    symbol: String,
    decimals: u8,
    owner: Principal,
    fee: Nat,
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

