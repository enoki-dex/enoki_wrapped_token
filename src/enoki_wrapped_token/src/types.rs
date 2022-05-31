use candid::{CandidType, Deserialize, Principal, types::number::Nat};

#[derive(Deserialize, CandidType, Clone, Debug)]
pub struct ManagementStats {
    pub owner: Principal,
    pub fee: Nat,
    pub deploy_time: u64,
}

impl Default for ManagementStats {
    fn default() -> Self {
        ManagementStats {
            owner: Principal::anonymous(),
            fee: Nat::from(0),
            deploy_time: 0,
        }
    }
}
