use std::cell::RefCell;

use candid::{candid_method, types::number::Nat, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

use crate::shards::{total_supply, update_fee};
use crate::stable::StableManagementStats;
use crate::types::ManagementStats;

thread_local! {
    static MANAGEMENT_STATS: RefCell<ManagementStats> = RefCell::new(ManagementStats::default());
}

pub fn export_stable_storage() -> (StableManagementStats,) {
    let management_stats = MANAGEMENT_STATS.with(|s| s.take()).into();
    (management_stats,)
}

pub fn import_stable_storage(management_stats: StableManagementStats) {
    MANAGEMENT_STATS.with(|s| s.replace(management_stats.into()));
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

pub fn init_management_data(stats: ManagementStats) {
    MANAGEMENT_STATS.with(|s| s.replace(stats));
}

#[query(name = "stats")]
#[candid_method(query)]
async fn stats() -> Stats {
    let mut stats: Stats = MANAGEMENT_STATS.with(|s| s.borrow().clone()).into();
    stats.total_supply = total_supply().await;
    stats.cycles = ic_cdk::api::canister_balance();
    stats
}

#[query(name = "owner")]
#[candid_method(query)]
fn owner() -> Principal {
    MANAGEMENT_STATS.with(|s| s.borrow().owner)
}

#[update(name = "setFee")]
#[candid_method(update, rename = "setFee")]
async fn set_fee(fee: Nat) -> Result<()> {
    assert_is_owner()?;
    MANAGEMENT_STATS.with(|s| s.borrow_mut().fee = fee.clone());
    update_fee(fee).await?;
    Ok(())
}

#[query(name = "getFee")]
#[candid_method(query, rename = "getFee")]
pub fn get_fee() -> Nat {
    MANAGEMENT_STATS.with(|s| s.borrow().fee.clone())
}

#[update(name = "setOwner")]
#[candid_method(update, rename = "setOwner")]
fn set_owner(owner: Principal) -> Result<()> {
    assert_is_owner()?;
    MANAGEMENT_STATS.with(|s| s.borrow_mut().owner = owner);
    Ok(())
}
