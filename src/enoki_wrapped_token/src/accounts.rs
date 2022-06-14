use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

use enoki_wrapped_token_shared::types::*;

use crate::shards::{get_lowest_utilization_shard, update_shard_accounts};

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
pub struct UserAccount {
    pub assigned_shard: Principal,
}

pub type UserAccounts = HashMap<Principal, UserAccount>; //TODO: convert to big-map (distributed among canisters)

thread_local! {
    static USER_ACCOUNTS: RefCell<UserAccounts> = RefCell::new(UserAccounts::default());
}

pub fn export_stable_storage() -> (UserAccounts,) {
    (USER_ACCOUNTS.with(|a| a.take()),)
}

pub fn import_stable_storage(user_accounts: UserAccounts) {
    USER_ACCOUNTS.with(|a| a.replace(user_accounts));
}

pub fn get_user_account(user: &Principal) -> Option<UserAccount> {
    USER_ACCOUNTS.with(|a| a.borrow().get(user).cloned())
}

#[update(name = "register")]
#[candid_method(update)]
async fn register(address: Principal) -> Principal {
    if let Some(existing) =
        USER_ACCOUNTS.with(|a| a.borrow().get(&address).map(|a| a.assigned_shard))
    {
        return existing;
    }
    let assigned_shard = get_lowest_utilization_shard();
    let new_user = UserAccount { assigned_shard };
    USER_ACCOUNTS.with(|a| a.borrow_mut().insert(address, new_user));
    update_shard_accounts(assigned_shard, |count| *count += 1);

    let response: Result<()> = ic_cdk::call(assigned_shard, "createAccount", (address,))
        .await
        .map_err(|err| err.into());

    match response {
        Ok(_) | Err(TxError::AccountAlreadyExists) => assigned_shard,
        Err(err) => panic!("{:?}", err),
    }
}

#[query(name = "getAssignedShardId")]
#[candid_method(query, rename = "getAssignedShardId")]
fn get_assigned_shard_id(address: Principal) -> Principal {
    USER_ACCOUNTS
        .with(|a| {
            if let Some(UserAccount { assigned_shard, .. }) = a.borrow().get(&address) {
                Ok(*assigned_shard)
            } else {
                Err(TxError::AccountDoesNotExist {
                    shard: format!("main contract {}", ic_cdk::id()),
                    user: address.to_string(),
                })
            }
        })
        .unwrap()
}

#[update(name = "transfer")]
#[candid_method(update)]
async fn transfer(to: Principal, amount: Nat) {
    let from = ic_cdk::caller();
    let from_shard = register(from).await;
    let to_shard = register(to).await;
    let response: Result<()> = ic_cdk::call(
        from_shard,
        "transferFromManager",
        (from, to_shard, to, amount),
    )
    .await
    .map_err(|err| err.into());
    response.unwrap()
}
