use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{AddAssign, SubAssign};

use candid::{candid_method, types::number::Nat, Principal};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

use crate::fees::{accept_fee, get_accrued_fees};
use crate::management::{assert_is_manager_contract, assert_is_sibling, get_fee};
use crate::stable::StableShardBalances;

pub type ShardBalances = HashMap<Principal, Nat>;

thread_local! {
    static SHARD_BALANCES: RefCell<ShardBalances> = RefCell::new(ShardBalances::default());
}

pub fn export_stable_storage() -> (StableShardBalances,) {
    let shard_balances: StableShardBalances = SHARD_BALANCES.with(|b| b.take()).into();
    (shard_balances,)
}

pub fn import_stable_storage(shard_balances: StableShardBalances) {
    SHARD_BALANCES.with(|b| b.replace(shard_balances.into()));
}

pub fn assert_is_customer(user: &Principal) -> Result<()> {
    if SHARD_BALANCES.with(|b| b.borrow().contains_key(user)) {
        Ok(())
    } else {
        Err(TxError::AccountDoesNotExist)
    }
}

#[update(name = "createAccount")]
#[candid_method(update, rename = "createAccount")]
pub fn create_account(account: Principal) -> Result<()> {
    assert_is_manager_contract()?;
    SHARD_BALANCES.with(|b| {
        let mut balances = b.borrow_mut();
        if balances.contains_key(&account) {
            return Err(TxError::AccountAlreadyExists);
        }
        balances.insert(account, Nat::from(0));
        Ok(())
    })
}

pub fn increase_balance(account: Principal, amount: Nat) {
    SHARD_BALANCES.with(|b| {
        let mut balances = b.borrow_mut();
        let balance = balances.entry(account).or_default();
        balance.add_assign(amount);
    });
}

pub fn decrease_balance(account: Principal, amount: Nat) -> Result<()> {
    SHARD_BALANCES.with(|b| {
        let mut balances = b.borrow_mut();
        let balance = balances.entry(account).or_default();
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

    SHARD_BALANCES.with(|b| {
        if b.borrow().get(&from).unwrap_or(&Nat::from(0)) < value {
            Err(TxError::InsufficientBalance)
        } else if check_to && !b.borrow().contains_key(&to) {
            Err(TxError::AccountDoesNotExist)
        } else {
            Ok(())
        }
    })
}

fn charge_fee(user: Principal, fee: Nat) -> Result<()> {
    SHARD_BALANCES.with(|b| {
        let mut balances = b.borrow_mut();
        let balance = balances.entry(user).or_default();
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
    from: Principal,
    shard_id: Principal,
    to: Principal,
    amount: Nat,
    notify_principal: Principal,
    notify_method: String,
) -> Result<()> {
    assert_is_sibling(&shard_id)?;
    ic_cdk::call(
        shard_id,
        "shardReceiveTransferAndCall",
        (from, to, amount, notify_principal, notify_method),
    )
    .await
    .map_err(|err| err.into())
}

#[update(name = "shardReceiveTransfer")]
#[candid_method(update, rename = "shardReceiveTransfer")]
async fn receive_transfer(to: Principal, value: Nat) -> Result<()> {
    assert_is_sibling(&ic_cdk::caller())?;
    assert_is_customer(&to)?;
    increase_balance(to, value);

    Ok(())
}

#[update(name = "shardReceiveTransferAndCall")]
#[candid_method(update, rename = "shardReceiveTransferAndCall")]
async fn receive_transfer_and_call(
    from: Principal,
    to: Principal,
    value: Nat,
    notify_principal: Principal,
    notify_method: String,
) -> Result<()> {
    assert_is_sibling(&ic_cdk::caller())?;
    assert_is_customer(&to)?;

    // notify recipient
    let result: std::result::Result<((),), _> =
        ic_cdk::call(notify_principal, &notify_method, (from, to, value.clone())).await;
    match result {
        Ok(_) => {
            // send funds to destination
            increase_balance(to, value);
            Ok(())
        }
        Err(error) => Err(error.into()),
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
async fn transfer(to_shard: Principal, to: Principal, value: Nat) -> Result<()> {
    transfer_internal(ic_cdk::caller(), to_shard, to, value).await
}

#[update(name = "transferFromManager")]
#[candid_method(update, rename = "transferFromManager")]
async fn transfer_from_manager(
    from: Principal,
    to_shard: Principal,
    to: Principal,
    value: Nat,
) -> Result<()> {
    assert_is_manager_contract()?;
    transfer_internal(from, to_shard, to, value).await
}

#[update(name = "shardTransferAndCall")]
#[candid_method(update, rename = "shardTransferAndCall")]
async fn transfer_and_call(
    shard_id: Principal,
    to: Principal,
    value: Nat,
    notify_principal: Principal,
    notify_method: String,
) -> Result<()> {
    let from = ic_cdk::caller();
    let fee = get_fee();
    pre_transfer_check(from, shard_id, to, &value, &fee)?;
    charge_fee(from, fee.clone())?;
    let value = value - fee;

    decrease_balance(from, value.clone())?;

    let result = if shard_id == ic_cdk::id() {
        let result: Result<()> =
            ic_cdk::call(notify_principal, &notify_method, (from, to, value.clone()))
                .await
                .map_err(|err| err.into());
        result.map(|_| {
            // send funds to destination
            increase_balance(to, value.clone());
        })
    } else {
        transfer_and_call_to_sibling_shard(
            from,
            shard_id,
            to,
            value.clone(),
            notify_principal,
            notify_method,
        )
        .await
    };

    if result.is_err() {
        // revert transaction
        increase_balance(from, value);
        return result;
    }

    Ok(())
}

#[query(name = "shardGetSupply")]
#[candid_method(query, rename = "shardGetSupply")]
fn shard_get_supply() -> Nat {
    SHARD_BALANCES.with(|b| {
        b.borrow()
            .values()
            .cloned()
            .fold(Nat::from(0), |sum, next| sum + next)
    }) + get_accrued_fees()
}

#[query(name = "shardBalanceOf")]
#[candid_method(query, rename = "shardBalanceOf")]
fn balance_of(account: Principal) -> Result<Nat> {
    SHARD_BALANCES
        .with(|b| b.borrow().get(&account).cloned())
        .ok_or(TxError::AccountDoesNotExist)
}
