use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::ManagementStats;

#[derive(CandidType, Clone, Deserialize, Serialize)]
pub struct StableManagementStats {
    pub owner: Principal,
    pub fee: String,
    pub deploy_time: u64,
}

impl From<StableManagementStats> for ManagementStats {
    fn from(s: StableManagementStats) -> Self {
        Self {
            owner: s.owner,
            fee: s.fee.parse().unwrap(),
            deploy_time: s.deploy_time,
        }
    }
}

impl From<ManagementStats> for StableManagementStats {
    fn from(s: ManagementStats) -> Self {
        Self {
            owner: s.owner,
            fee: s.fee.to_string(),
            deploy_time: s.deploy_time,
        }
    }
}
