use std::string::String;

use candid::{candid_method, types::number::Nat, Principal};
use ic_cdk_macros::*;

#[allow(unused_imports)]
use enoki_wrapped_token_shared::types::Result;

use crate::management::{assert_is_owner, init_fee};
#[allow(unused_imports)]
use crate::management::{init_management_data, Stats};
use crate::metadata::{init_metadata, Metadata};
#[allow(unused_imports)]
use crate::shards::Shard;
use crate::types::ManagementStats;

mod accounts;
mod management;
mod metadata;
mod shards;
mod stable;
mod types;
mod upgrade;

#[init]
#[candid_method(init)]
fn init() {
    init_management_data(ManagementStats {
        owner: ic_cdk::caller(),
        fee: Default::default(),
        deploy_time: ic_cdk::api::time(),
    });
}

#[update(name = "finishInit")]
#[candid_method(update, rename = "finishInit")]
fn finish_init(
    underlying_token: Principal,
    logo: String,
    name: String,
    symbol: String,
    decimals: u8,
    fee: Nat,
) {
    assert_is_owner().unwrap();
    init_metadata(Metadata {
        logo,
        name,
        symbol,
        decimals,
        underlying_token,
    });
    init_fee(fee);
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
