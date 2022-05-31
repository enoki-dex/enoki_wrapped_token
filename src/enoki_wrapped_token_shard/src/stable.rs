use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{AddAssign, Sub, SubAssign};

use candid::{candid_method, CandidType, Func, Principal, types::number::Nat};
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

use enoki_wrapped_token_shared::types::*;

use crate::balances::{EscrowBalances, ShardBalances};
use crate::fees::{accept_fee, FeeBalance};
use crate::management::{assert_is_manager_contract, get_fee};
use crate::ManagerContractData;

#[derive(CandidType, Clone, Deserialize, Serialize)]
pub struct StableShardBalances(Vec<(Principal, String)>);

#[derive(CandidType, Clone, Deserialize, Serialize)]
pub struct StableEscrowBalances {
    last_id: u64,
    deposits: Vec<(u64, String)>,
}

#[derive(CandidType, Clone, Deserialize, Serialize)]
pub struct StableFeeBalance(String);

#[derive(CandidType, Clone, Deserialize, Serialize)]
pub struct StableManagerContractData {
    pub owner: Principal,
    pub manager_contract: Principal,
    pub fee: String,
    pub underlying_token: Principal,
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

impl From<StableEscrowBalances> for EscrowBalances {
    fn from(balances: StableEscrowBalances) -> Self {
        Self {
            last_id: balances.last_id,
            deposits: balances
                .deposits
                .into_iter()
                .map(|(id, amount)| (id, amount.parse().unwrap()))
                .collect(),
        }
    }
}

impl From<EscrowBalances> for StableEscrowBalances {
    fn from(balances: EscrowBalances) -> Self {
        Self {
            last_id: balances.last_id,
            deposits: balances
                .deposits
                .into_iter()
                .map(|(id, amount)| (id, amount.to_string()))
                .collect(),
        }
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
        }
    }
}
