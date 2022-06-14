use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Principal, types::number::Nat};
use candid::utils::{ArgumentDecoder, ArgumentEncoder};
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

use enoki_wrapped_token_shared::types::*;

use crate::accounts::{get_user_account, UserAccount};
use crate::management::{assert_is_owner, get_fee};
use crate::metadata::get_underlying_token;

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
pub struct Shard {
    id: Principal,
    num_accounts: u64,
}

pub type Shards = HashMap<Principal, Shard>;

thread_local! {
    static SHARDS: RefCell<Shards> = RefCell::new(Shards::default());
}

pub fn export_stable_storage() -> (Shards, ) {
    (SHARDS.with(|s| s.take()), )
}

pub fn import_stable_storage(shards: Shards) {
    SHARDS.with(|s| s.replace(shards));
}

#[query(name = "getShardIds")]
#[candid_method(query, rename = "getShardIds")]
fn get_shard_ids() -> Vec<Principal> {
    SHARDS.with(|s| s.borrow().keys().copied().collect())
}

#[query(name = "getShardsInfo")]
#[candid_method(query, rename = "getShardsInfo")]
fn get_shards_info() -> Vec<Shard> {
    SHARDS.with(|s| s.borrow().values().cloned().collect())
}

#[query(name = "totalSupply")]
#[candid_method(query, rename = "totalSupply")]
pub async fn total_supply() -> Nat {
    let values: Result<Vec<(Nat, )>> = foreach_shard("shardGetSupply", ()).await;
    values.map(|values| {
        values
            .into_iter()
            .fold(Nat::from(0), |sum, next| sum + next.0)
    }).unwrap()
}

#[query(name = "getAccruedFees")]
#[candid_method(query, rename = "getAccruedFees")]
async fn get_accrued_fees() -> Nat {
    let values: Result<Vec<(Nat, )>> = foreach_shard("getAccruedFees", ()).await;
    values.map(|values| {
        values
            .into_iter()
            .fold(Nat::from(0), |sum, next| sum + next.0)
    }).unwrap()
}

#[query(name = "balanceOf")]
#[candid_method(query, rename = "balanceOf")]
async fn balance_of(id: Principal) -> Nat {
    if let Some(UserAccount {
                    assigned_shard,
                }) = get_user_account(&id)
    {
        let balance: Result<(Nat, )> =
            ic_cdk::call(assigned_shard, "shardBalanceOf", (id, ))
                .await
                .map_err(|err| err.into());

        balance
            .map(|res| res.0)
            .unwrap_or_default()
    } else {
        Default::default()
    }
}

async fn foreach_shard<T: ArgumentEncoder + Clone, R: for<'a> ArgumentDecoder<'a>>(
    method: &str,
    args: T,
) -> Result<Vec<R>> {
    let shards = get_shard_ids();
    let responses: Vec<std::result::Result<R, _>> = futures::future::join_all(
        shards
            .into_iter()
            .map(|shard| ic_cdk::call(shard, method, args.clone())),
    )
        .await;
    responses
        .into_iter()
        .collect::<std::result::Result<Vec<R>, _>>()
        .map_err(|err| err.into())
}

#[update(name = "addShard")]
#[candid_method(update, rename = "addShard")]
async fn add_shard(id: Principal) -> Result<()> {
    assert_is_owner()?;
    let sibling_shards = get_shard_ids();

    let response: Result<()> = ic_cdk::call(
        id,
        "initShard",
        (get_underlying_token(), sibling_shards, get_fee()),
    )
        .await
        .map_err(|err| err.into());
    response?;

    foreach_shard::<(Principal, ), ()>("addSiblingShard", (id, )).await?;

    SHARDS.with(|s| {
        s.borrow_mut().insert(
            id,
            Shard {
                id,
                num_accounts: 0,
            },
        )
    });

    Ok(())
}

pub fn update_shard_accounts<TF: Fn(&mut u64)>(id: Principal, func: TF) {
    SHARDS.with(|s| {
        let mut shards = s.borrow_mut();
        let number = &mut shards.get_mut(&id).unwrap().num_accounts;
        func(number);
    })
}

pub fn get_lowest_utilization_shard() -> Principal {
    //TODO: choose by lowest transfer activity, not number of accounts
    SHARDS.with(|s| {
        s.borrow()
            .values()
            .min_by(|&a, &b| {
                let comp = a.num_accounts.cmp(&b.num_accounts);
                if let Ordering::Equal = comp {
                    a.id.to_string().cmp(&b.id.to_string())
                } else {
                    comp
                }
            })
            .map(|s| s.id)
            .expect("no shards exist")
    })
}

pub async fn update_fee(new_fee: Nat) -> Result<()> {
    let _ = foreach_shard::<(Nat, ), ()>("setFee", (new_fee, )).await?;

    Ok(())
}
