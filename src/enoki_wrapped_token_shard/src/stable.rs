use std::collections::HashSet;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::balances::ShardBalances;
use crate::fees::FeeBalance;
use crate::ManagerContractData;

#[derive(CandidType, Clone, Deserialize, Serialize)]
pub struct StableShardBalances(Vec<(Principal, String)>);

#[derive(CandidType, Clone, Deserialize, Serialize)]
pub struct StableFeeBalance(String);

#[derive(CandidType, Clone, Deserialize, Serialize)]
pub struct StableManagerContractData {
    pub owner: Principal,
    pub manager_contract: Principal,
    pub fee: String,
    pub underlying_token: Principal,
    pub sibling_shards: HashSet<Principal>,
    pub deploy_time: u64,
}

impl From<StableShardBalances> for ShardBalances {
    fn from(balances: StableShardBalances) -> Self {
        balances
            .0
            .into_iter()
            .map(|(principal, balance)| (principal, balance.parse().unwrap()))
            .collect()
    }
}

impl From<ShardBalances> for StableShardBalances {
    fn from(balances: ShardBalances) -> Self {
        Self(
            balances
                .into_iter()
                .map(|(principal, balance)| (principal, balance.to_string()))
                .collect(),
        )
    }
}

impl From<StableFeeBalance> for FeeBalance {
    fn from(balance: StableFeeBalance) -> Self {
        Self(balance.0.parse().unwrap())
    }
}

impl From<FeeBalance> for StableFeeBalance {
    fn from(balance: FeeBalance) -> Self {
        Self(balance.0.to_string())
    }
}

impl From<StableManagerContractData> for ManagerContractData {
    fn from(data: StableManagerContractData) -> Self {
        Self {
            owner: data.owner,
            manager_contract: data.manager_contract,
            fee: data.fee.parse().unwrap(),
            underlying_token: data.underlying_token,
            sibling_shards: data.sibling_shards,
            deploy_time: data.deploy_time,
        }
    }
}

impl From<ManagerContractData> for StableManagerContractData {
    fn from(data: ManagerContractData) -> Self {
        Self {
            owner: data.owner,
            manager_contract: data.manager_contract,
            fee: data.fee.to_string(),
            underlying_token: data.underlying_token,
            sibling_shards: data.sibling_shards,
            deploy_time: data.deploy_time,
        }
    }
}
