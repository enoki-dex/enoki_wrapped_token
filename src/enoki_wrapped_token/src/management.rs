use std::borrow::BorrowMut;
use std::cell::RefCell;

use candid::{candid_method, types::number::Nat, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

use crate::shards::total_supply;
use crate::types::ManagementStats;

thread_local! {
    static MANAGEMENT_STATS: RefCell<ManagementStats> = RefCell::new(ManagementStats::default());
}

#[derive(Deserialize, CandidType, Clone, Debug)]
pub struct Stats {
    pub total_supply: Nat,
    pub owner: Principal,
    pub fee: Nat,
    pub deploy_time: u64,
    pub cycles: u64,
}

impl From<ManagementStats> for Stats {
    fn from(m: ManagementStats) -> Self {
        Stats {
            total_supply: Default::default(),
            owner: m.owner,
            fee: m.fee,
            deploy_time: m.deploy_time,
            cycles: 0,
        }
    }
}

pub fn assert_is_owner() -> Result<()> {
    if MANAGEMENT_STATS.with(|s| s.borrow().owner) == ic_cdk::caller() {
        Ok(())
    } else {
        Err(TxError::Unauthorized)
    }
}

#[query(name = "stats")]
#[candid_method(query)]
async fn stats() -> Result<Stats> {
    let mut stats: Stats = MANAGEMENT_STATS.with(|s| s.borrow().clone()).into();
    stats.total_supply = total_supply().await?;
    stats.cycles = ic_cdk::api::canister_balance();
    Ok(stats)
}

#[query(name = "owner")]
#[candid_method(query)]
fn owner() -> Principal {
    MANAGEMENT_STATS.with(|s| s.borrow().owner)
}

#[update(name = "setFee")]
#[candid_method(update, rename = "setFee")]
fn set_fee(fee: Nat) -> Result<()> {
    assert_is_owner()?;
    MANAGEMENT_STATS.with(|s| s.borrow_mut().fee = fee);
    Ok(())
}

#[update(name = "setOwner")]
#[candid_method(update, rename = "setOwner")]
fn set_owner(owner: Principal) -> Result<()> {
    assert_is_owner()?;
    MANAGEMENT_STATS.with(|s| s.borrow_mut().owner = owner);
    Ok(())
}
