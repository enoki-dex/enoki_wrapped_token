use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::{AddAssign, SubAssign};

use candid::{candid_method, Principal, types::number::Nat};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

use crate::fees::{accept_fee, get_accrued_fees};
use crate::management::{assert_is_manager_contract, assert_is_sibling, get_fee};
use crate::stable::StableShardBalances;

pub type ShardBalances = HashMap<Principal, Nat>;
pub type ShardSpenders = HashMap<Principal, HashSet<Principal>>;

#[derive(Default)]
pub struct ShardBalancesState {
    balances: ShardBalances,
    spenders: ShardSpenders,
}

thread_local! {
    static STATE: RefCell<ShardBalancesState> = RefCell::new(ShardBalancesState::default());
}

pub fn export_stable_storage() -> (StableShardBalances, ShardSpenders) {
    let ShardBalancesState { balances, spenders } = STATE.with(|b| b.take());
    (balances.into(), spenders)
}

pub fn import_stable_storage(balances: StableShardBalances, spenders: ShardSpenders) {
    STATE.with(|b| {
        b.replace(ShardBalancesState {
            balances: balances.into(),
            spenders,
        })
    });
}

pub fn assert_is_customer(user: &Principal) -> Result<()> {
    if STATE.with(|b| b.borrow().balances.contains_key(user)) {
        Ok(())
    } else {
        Err(TxError::AccountDoesNotExist {
            shard: ic_cdk::id().to_string(),
            user: user.to_string(),
        })
    }
}

#[update(name = "createAccount")]
#[candid_method(update, rename = "createAccount")]
pub fn create_account(account: Principal) {
    assert_is_manager_contract().unwrap();
    STATE.with(|b| {
        let mut balances = b.borrow_mut();
        if balances.balances.contains_key(&account) {
            panic!("{:?}", TxError::AccountAlreadyExists);
        }
        balances.balances.insert(account, Nat::from(0));
    })
}

pub fn increase_balance(account: Principal, amount: Nat) {
    STATE.with(|b| {
        let mut balances = b.borrow_mut();
        let balance = balances.balances.entry(account).or_default();
        balance.add_assign(amount);
    });
}

pub fn decrease_balance(account: Principal, amount: Nat) -> Result<()> {
    STATE.with(|b| {
        let mut balances = b.borrow_mut();
        let balance = balances.balances.entry(account).or_default();
        if *balance >= amount {
            balance.sub_assign(amount);
            Ok(())
        } else {
            Err(TxError::InsufficientBalance)
        }
    })
}

fn pre_transfer_check(
    from: Principal,
    shard_id: Principal,
    to: Principal,
    value: &Nat,
    fee: &Nat,
) -> Result<()> {
    let check_to = shard_id == ic_cdk::id();
    assert_is_customer(&from)?;
    if check_to {
        assert_is_customer(&to)?;
    }
    if value <= fee {
        return Err(TxError::TransferValueTooSmall);
    }

    STATE.with(|b| {
        if b.borrow().balances.get(&from).unwrap_or(&Nat::from(0)) < value {
            Err(TxError::InsufficientBalance)
        } else if check_to && !b.borrow().balances.contains_key(&to) {
            Err(TxError::AccountDoesNotExist {
                shard: ic_cdk::id().to_string(),
                user: to.to_string(),
            })
        } else {
            Ok(())
        }
    })
}

fn charge_fee(user: Principal, fee: Nat) -> Result<()> {
    STATE.with(|b| {
        let mut balances = b.borrow_mut();
        let balance = balances.balances.entry(user).or_default();
        if *balance < fee {
            return Err(TxError::InsufficientBalance);
        }
        balance.sub_assign(fee.clone());
        Ok(())
    })?;
    accept_fee(fee);
    Ok(())
}

async fn transfer_to_sibling_shard(shard_id: Principal, to: Principal, amount: Nat) -> Result<()> {
    assert_is_sibling(&shard_id)?;
    ic_cdk::call(shard_id, "shardReceiveTransfer", (to, amount))
        .await
        .map_err(|err| err.into())
}

async fn transfer_and_call_to_sibling_shard(
    shard_id: Principal,
    notification: ShardedTransferNotification,
    notify_principal: Principal,
    notify_method: String,
) -> Result<String> {
    assert_is_sibling(&shard_id)?;
    let result: Result<(String, )> = ic_cdk::call(
        shard_id,
        "shardReceiveTransferAndCall",
        (notification, notify_principal, notify_method),
    )
        .await
        .map_err(|err| err.into());
    result.map(|res| res.0)
}

#[update(name = "shardReceiveTransfer")]
#[candid_method(update, rename = "shardReceiveTransfer")]
async fn receive_transfer(to: Principal, value: Nat) {
    assert_is_sibling(&ic_cdk::caller()).unwrap();
    assert_is_customer(&to).unwrap();
    increase_balance(to, value);
}

#[update(name = "shardReceiveTransferAndCall")]
#[candid_method(update, rename = "shardReceiveTransferAndCall")]
async fn receive_transfer_and_call(
    notification: ShardedTransferNotification,
    notify_principal: Principal,
    notify_method: String,
) -> String {
    assert_is_sibling(&ic_cdk::caller()).unwrap();
    let to = notification.to;
    let value = notification.value.clone();
    assert_is_customer(&to).unwrap();

    // notify recipient
    let result: std::result::Result<(String, ), _> =
        ic_cdk::call(notify_principal, &notify_method, (notification, )).await;
    match result {
        Ok(response) => {
            // send funds to destination
            increase_balance(to, value);
            response.0
        }
        Err(error) => panic!("{:?}", error),
    }
}

async fn transfer_internal(
    from: Principal,
    to_shard: Principal,
    to: Principal,
    value: Nat,
) -> Result<()> {
    let fee = get_fee();
    pre_transfer_check(from, to_shard, to, &value, &fee)?;
    charge_fee(from, fee.clone())?;
    let value = value - fee;

    decrease_balance(from, value.clone())?;

    if to_shard == ic_cdk::id() {
        increase_balance(to, value.clone());
    } else {
        if let Err(error) = transfer_to_sibling_shard(to_shard, to, value.clone()).await {
            increase_balance(from, value);
            return Err(error.into());
        }
    }

    Ok(())
}

#[update(name = "shardTransfer")]
#[candid_method(update, rename = "shardTransfer")]
async fn transfer(to_shard: Principal, to: Principal, value: Nat) {
    transfer_internal(ic_cdk::caller(), to_shard, to, value)
        .await
        .unwrap();
}

#[update(name = "transferFromManager")]
#[candid_method(update, rename = "transferFromManager")]
async fn transfer_from_manager(from: Principal, to_shard: Principal, to: Principal, value: Nat) {
    assert_is_manager_contract().unwrap();
    transfer_internal(from, to_shard, to, value).await.unwrap();
}

fn assert_is_spender(of_account: Principal) -> Result<()> {
    if STATE.with(|s| {
        let s = s.borrow();
        if let Some(spenders) = s.spenders.get(&of_account) {
            if spenders.contains(&ic_cdk::caller()) {
                return true;
            }
        }
        false
    }) {
        Ok(())
    } else {
        Err(TxError::Unauthorized)
    }
}

// This account is authorized to drain all your tokens
#[update(name = "addSpender")]
#[candid_method(update, rename = "addSpender")]
async fn add_spender(account: Principal) {
    STATE.with(|s| {
        s.borrow_mut()
            .spenders
            .entry(ic_cdk::caller())
            .or_default()
            .insert(account)
    });
}

#[update(name = "removeSpender")]
#[candid_method(update, rename = "removeSpender")]
async fn remove_spender(account: Principal) {
    STATE.with(|s| {
        s.borrow_mut()
            .spenders
            .entry(ic_cdk::caller())
            .or_default()
            .remove(&account)
    });
}

#[update(name = "shardSpend")]
#[candid_method(update, rename = "shardSpend")]
async fn spend(from: Principal, to_shard: Principal, to: Principal, value: Nat) {
    assert_is_spender(from).unwrap();
    transfer_internal(from, to_shard, to, value).await.unwrap();
}

async fn transfer_and_call_internal(
    from: Principal,
    shard_id: Principal,
    to: Principal,
    value: Nat,
    notify_principal: Principal,
    notify_method: String,
    data: String,
) -> Result<String> {
    let fee = get_fee();
    pre_transfer_check(from, shard_id, to, &value, &fee)?;
    charge_fee(from, fee.clone())?;
    let value = value - fee.clone();

    decrease_balance(from, value.clone())?;

    let notification = ShardedTransferNotification {
        from,
        from_shard: ic_cdk::id(),
        to,
        fee_charged: fee,
        value: value.clone(),
        data,
    };
    let result = if shard_id == ic_cdk::id() {
        let result: Result<(String, )> =
            ic_cdk::call(notify_principal, &notify_method, (notification, ))
                .await
                .map_err(|err| err.into());
        result.map(|res| {
            // send funds to destination
            increase_balance(to, value.clone());
            res.0
        })
    } else {
        transfer_and_call_to_sibling_shard(shard_id, notification, notify_principal, notify_method)
            .await
    };

    result
}

#[update(name = "shardTransferAndCall")]
#[candid_method(update, rename = "shardTransferAndCall")]
async fn transfer_and_call(
    shard_id: Principal,
    to: Principal,
    value: Nat,
    notify_principal: Principal,
    notify_method: String,
    data: String,
) -> String {
    let from = ic_cdk::caller();
    transfer_and_call_internal(
        from,
        shard_id,
        to,
        value,
        notify_principal,
        notify_method,
        data,
    )
        .await
        .unwrap()
}

#[update(name = "shardSpendAndCall")]
#[candid_method(update, rename = "shardSpendAndCall")]
async fn spend_and_call(
    from: Principal,
    shard_id: Principal,
    to: Principal,
    value: Nat,
    notify_principal: Principal,
    notify_method: String,
    data: String,
) -> String {
    assert_is_spender(from).unwrap();
    transfer_and_call_internal(
        from,
        shard_id,
        to,
        value,
        notify_principal,
        notify_method,
        data,
    )
        .await
        .unwrap()
}

#[query(name = "shardGetSupply")]
#[candid_method(query, rename = "shardGetSupply")]
fn shard_get_supply() -> Nat {
    STATE.with(|b| {
        b.borrow()
            .balances
            .values()
            .cloned()
            .fold(Nat::from(0), |sum, next| sum + next)
    }) + get_accrued_fees()
}

#[query(name = "shardBalanceOf")]
#[candid_method(query, rename = "shardBalanceOf")]
fn balance_of(account: Principal) -> Nat {
    STATE
        .with(|b| b.borrow().balances.get(&account).cloned())
        .ok_or(TxError::AccountDoesNotExist {
            shard: ic_cdk::id().to_string(),
            user: account.to_string(),
        })
        .unwrap()
}
