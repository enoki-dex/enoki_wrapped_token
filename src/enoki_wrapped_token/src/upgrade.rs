use candid::{CandidType, Deserialize};
use ic_cdk_macros::*;

use crate::{accounts, management, metadata, shards};
use crate::accounts::UserAccounts;
use crate::metadata::Metadata;
use crate::shards::Shards;
use crate::stable::StableManagementStats;

#[derive(Deserialize, CandidType)]
struct UpgradePayload {
    user_accounts: UserAccounts,
    management_stats: StableManagementStats,
    metadata: Metadata,
    shards: Shards,
}

#[pre_upgrade]
fn pre_upgrade() {
    let (user_accounts, ) = accounts::export_stable_storage();
    let (management_stats, ) = management::export_stable_storage();
    let (metadata, ) = metadata::export_stable_storage();
    let (shards, ) = shards::export_stable_storage();
    let payload = UpgradePayload {
        user_accounts,
        management_stats,
        metadata,
        shards,
    };
    ic_cdk::storage::stable_save((payload, )).expect("failed to save to stable storage");
}

#[post_upgrade]
fn post_upgrade() {
    let (payload, ): (UpgradePayload, ) =
        ic_cdk::storage::stable_restore().expect("failed to restore from stable storage");

    let UpgradePayload {
        user_accounts,
        management_stats,
        metadata,
        shards,
    } = payload;

    accounts::import_stable_storage(user_accounts);
    management::import_stable_storage(management_stats);
    metadata::import_stable_storage(metadata);
    shards::import_stable_storage(shards);
}
