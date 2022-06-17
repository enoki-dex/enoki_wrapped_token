use candid::{CandidType, Deserialize};
use ic_cdk_macros::*;

use crate::{balances, fees, management};
use crate::balances::ShardSpenders;
use crate::stable::{StableFeeBalance, StableManagerContractData, StableShardBalances};

#[derive(Deserialize, CandidType)]
struct UpgradePayload {
    shard_balances: StableShardBalances,
    shard_spenders: ShardSpenders,
    fee_balance: StableFeeBalance,
    manager_data: StableManagerContractData,
}

#[pre_upgrade]
fn pre_upgrade() {
    let (shard_balances, shard_spenders) = balances::export_stable_storage();
    let (fee_balance, ) = fees::export_stable_storage();
    let (manager_data, ) = management::export_stable_storage();
    let payload = UpgradePayload {
        shard_balances,
        shard_spenders,
        fee_balance,
        manager_data,
    };
    ic_cdk::storage::stable_save((payload, )).expect("failed to save to stable storage");
}

#[post_upgrade]
fn post_upgrade() {
    let (payload, ): (UpgradePayload, ) =
        ic_cdk::storage::stable_restore().expect("failed to restore from stable storage");

    let UpgradePayload {
        shard_balances,
        shard_spenders,
        fee_balance,
        manager_data,
    } = payload;

    balances::import_stable_storage(shard_balances, shard_spenders);
    fees::import_stable_storage(fee_balance);
    management::import_stable_storage(manager_data);
}
