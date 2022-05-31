use std::cell::RefCell;

use candid::{candid_method, CandidType, Principal, types::number::Nat};
use candid::utils::{ArgumentDecoder, ArgumentEncoder};
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

use enoki_wrapped_token_shared::types::*;

use crate::accounts::{get_user_account, UserAccount};
use crate::management::assert_is_owner;
use crate::metadata::get_underlying_token;

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
pub struct Shard {
    id: Principal,
    num_accounts: u64,
}

pub type Shards = Vec<Shard>;

thread_local! {
    static SHARDS: RefCell<Shards> = RefCell::new(Shards::default());
}

pub fn export_stable_storage() -> (Shards, ) {
    (SHARDS.with(|s| s.take()), )
}

pub fn import_stable_storage(shards: Shards) {
    SHARDS.with(|s| s.replace(shards));
}

fn get_shard_ids() -> Vec<Principal> {
    SHARDS.with(|s| s.borrow().iter().map(|s| s.id).collect())
}

#[query(name = "totalSupply")]
#[candid_method(query, rename = "totalSupply")]
pub async fn total_supply() -> Result<Nat> {
    let values: Result<Vec<(Nat, )>> = foreach_shard("shardGetSupply", ()).await;

    Ok(values?
        .into_iter()
        .fold(Nat::from(0), |sum, next| sum + next.0))
}

#[query(name = "balanceOf")]
#[candid_method(query, rename = "balanceOf")]
async fn balance_of(id: Principal) -> Nat {
    if let Some(UserAccount {
                    shard_account: Some(shard_account),
                    shard_id,
                }) = get_user_account(&id)
    {
        let balance: Result<(Result<Nat>, )> =
            ic_cdk::call(shard_id, "shardBalanceOf", (shard_account, ))
                .await
                .map_err(|err| err.into());

        balance
            .map(|res| res.0.unwrap_or_default())
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
    let sibling_shards: Vec<_> = SHARDS.with(|s| s.borrow().iter().map(|shard| shard.id).collect());

    let response: Result<()> =
        ic_cdk::call(id, "initShard", (get_underlying_token(), sibling_shards))
            .await
            .map_err(|err| err.into());
    response?;

    foreach_shard::<(Principal, ), ()>("addSiblingShard", (id, )).await?;

    SHARDS.with(|s| {
        s.borrow_mut().push(Shard {
            id,
            num_accounts: 0,
        })
    });

    Ok(())
}

pub fn get_lowest_utilization_shard() -> Result<Principal> {
    //TODO: choose by lowest transfer activity, not number of accounts
    SHARDS.with(|s| {
        s.borrow()
            .iter()
            .min_by(|&a, &b| a.num_accounts.cmp(&b.num_accounts))
            .map(|s| s.id)
            .ok_or(TxError::ShardDoesNotExist)
    })
}

pub async fn update_fee(new_fee: Nat) -> Result<()> {
    let _ = foreach_shard::<(Nat, ), ()>("setFee", (new_fee, )).await?;

    Ok(())
}
