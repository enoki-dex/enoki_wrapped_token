use std::collections::HashMap;

use candid::{candid_method, CandidType, Deserialize, Principal, types::number::Nat};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

struct Shard {
    id: Principal,
    num_accounts: u64,
}

type Shards = Vec<Principal>;
type UserAccounts = HashMap<Principal, UserAccount>; //TODO: convert to big-map (distributed among canisters)

#[query(name = "totalSupply")]
#[candid_method(query, rename = "totalSupply")]
pub async fn total_supply() -> Result<Nat> {
    todo!()
}

#[query(name = "balanceOf")]
#[candid_method(query, rename = "balanceOf")]
fn balance_of(id: Principal) -> Nat {
    todo!()
}
