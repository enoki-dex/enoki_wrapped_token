use std::string::String;

use candid::{candid_method, Principal, types::number::Nat};
use ic_cdk_macros::*;

#[allow(unused_imports)]
use enoki_wrapped_token_shared::types::Result;

#[allow(unused_imports)]
use crate::management::{init_management_data, Stats};
use crate::metadata::{init_metadata, Metadata};
#[allow(unused_imports)]
use crate::shards::Shard;
use crate::types::ManagementStats;

mod metadata;
mod shards;
mod management;
mod upgrade;
mod types;
mod accounts;
mod stable;

#[init]
#[candid_method(init)]
fn init(
    underlying_token: Principal,
    logo: String,
    name: String,
    symbol: String,
    decimals: u8,
    owner: Principal,
    fee: Nat,
) {
    init_metadata(Metadata {
        logo,
        name,
        symbol,
        decimals,
        underlying_token,
    });
    init_management_data(ManagementStats {
        owner,
        fee,
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

