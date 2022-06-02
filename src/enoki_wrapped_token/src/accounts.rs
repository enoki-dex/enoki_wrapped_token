use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

use enoki_wrapped_token_shared::types::*;

use crate::shards::{get_lowest_utilization_shard, update_shard_accounts};

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
pub struct UserAccount {
    pub shard_account: Option<Principal>,
    pub shard_id: Principal,
}

pub type UserAccounts = HashMap<Principal, UserAccount>; //TODO: convert to big-map (distributed among canisters)

thread_local! {
    static USER_ACCOUNTS: RefCell<UserAccounts> = RefCell::new(UserAccounts::default());
}

pub fn export_stable_storage() -> (UserAccounts, ) {
    (USER_ACCOUNTS.with(|a| a.take()), )
}

pub fn import_stable_storage(user_accounts: UserAccounts) {
    USER_ACCOUNTS.with(|a| a.replace(user_accounts));
}

pub fn get_user_account(user: &Principal) -> Option<UserAccount> {
    USER_ACCOUNTS.with(|a| a.borrow().get(user).cloned())
}

#[update(name = "startRegistration")]
#[candid_method(update, rename = "startRegistration")]
async fn start_registration() -> Result<Principal> {
    let caller = ic_cdk::caller();
    if USER_ACCOUNTS.with(|a| a.borrow().contains_key(&caller)) {
        return Err(TxError::AccountAlreadyExists);
    }
    let assigned_shard = get_lowest_utilization_shard()?;
    let new_user = UserAccount {
        shard_account: None,
        shard_id: assigned_shard,
    };
    USER_ACCOUNTS.with(|a| a.borrow_mut().insert(caller, new_user));
    update_shard_accounts(assigned_shard, |count| *count += 1);

    Ok(assigned_shard)
}

#[update(name = "completeRegistration")]
#[candid_method(update, rename = "completeRegistration")]
async fn complete_registration(shard_account: Principal) -> Result<()> {
    let caller = ic_cdk::caller();
    let shard_id = USER_ACCOUNTS.with(|a| {
        if let Some(UserAccount {
                        shard_account: existing,
                        shard_id,
                    }) = a.borrow().get(&caller)
        {
            if existing.is_some() {
                Err(TxError::Other("registration already completed".to_string()))
            } else {
                Ok(*shard_id)
            }
        } else {
            Err(TxError::Other(
                "you must startRegistration first".to_string(),
            ))
        }
    })?;

    let response: Result<()> = ic_cdk::call(shard_id, "createAccount", (shard_account, ))
        .await
        .map_err(|err| err.into());
    response?;

    USER_ACCOUNTS
        .with(|a| a.borrow_mut().get_mut(&caller).unwrap().shard_account = Some(shard_account));

    Ok(())
}

#[query(name = "getAssignedShardId")]
#[candid_method(query, rename = "getAssignedShardId")]
fn get_assigned_shard_id() -> Result<Principal> {
    USER_ACCOUNTS.with(|a| {
        if let Some(UserAccount {
                        shard_account: Some(account),
                        ..
                    }) = a.borrow().get(&ic_cdk::caller())
        {
            Ok(*account)
        } else {
            Err(TxError::AccountDoesNotExist)
        }
    })
}
