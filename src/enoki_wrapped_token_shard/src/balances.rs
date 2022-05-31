use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{AddAssign, Sub, SubAssign};

use candid::{candid_method, CandidType, Deserialize, Func, Principal, types::number::Nat};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

use crate::fees::accept_fee;
use crate::management::{assert_is_manager_contract, get_fee};
use crate::stable::{StableEscrowBalances, StableShardBalances};

pub type ShardBalances = HashMap<Principal, Nat>;

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
pub struct EscrowBalances {
    pub last_id: u64,
    pub deposits: HashMap<u64, Nat>,
}

impl EscrowBalances {
    pub(crate) fn deposit(&mut self, amount: Nat) -> u64 {
        let id = self.last_id;
        self.last_id += 1;
        self.deposits.insert(id, amount);
        id
    }
    pub(crate) fn withdraw(&mut self, id: u64) -> Result<Nat> {
        self.deposits.remove(&id).ok_or(TxError::Other(
            "Error with EscrowBalances: cannot find id".to_string(),
        ))
    }
}

thread_local! {
    static SHARD_BALANCES: RefCell<ShardBalances> = RefCell::new(ShardBalances::default());
    static FUNDS_IN_ESCROW: RefCell<EscrowBalances> = RefCell::new(EscrowBalances::default());
}

pub fn export_stable_storage() -> (StableShardBalances, StableEscrowBalances) {
    let shard_balances: StableShardBalances = SHARD_BALANCES.with(|b| b.take()).into();
    let escrow_balances: StableEscrowBalances = FUNDS_IN_ESCROW.with(|b| b.take()).into();
    (shard_balances, escrow_balances)
}

pub fn import_stable_storage(
    shard_balances: StableShardBalances,
    escrow_balances: StableEscrowBalances,
) {
    SHARD_BALANCES.with(|b| b.replace(shard_balances.into()));
    FUNDS_IN_ESCROW.with(|b| b.replace(escrow_balances.into()));
}

pub fn assert_is_customer(user: &Principal) -> Result<()> {
    if SHARD_BALANCES.with(|b| b.borrow().contains_key(user)) {
        Ok(())
    } else {
        Err(TxError::AccountDoesNotExist)
    }
}

#[update(name = "createAccount")]
#[candid_method(update)]
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

fn pre_transfer_check(from: Principal, to: Principal, value: &Nat, fee: &Nat) -> Result<()> {
    assert_is_customer(&from)?;
    assert_is_customer(&to)?;
    if value <= fee {
        return Err(TxError::TransferValueTooSmall);
    }

    SHARD_BALANCES.with(|b| {
        if b.borrow().get(&from).unwrap_or(&Nat::from(0)) < value {
            Err(TxError::InsufficientBalance)
        } else if !b.borrow().contains_key(&to) {
            Err(TxError::AccountDoesNotExist)
        } else {
            Ok(())
        }
    })
}

fn charge_fee(user: Principal, fee: Nat) -> Result<()> {
    SHARD_BALANCES.with(|b| {
        let mut balances = b.borrow_mut();
        let mut balance = balances.entry(user).or_default();
        if *balance < fee {
            return Err(TxError::InsufficientBalance);
        }
        balance.sub_assign(fee.clone());
        Ok(())
    })?;
    accept_fee(fee);
    Ok(())
}

#[update(name = "shardTransfer")]
#[candid_method(update, rename = "shardTransfer")]
fn transfer(to: Principal, value: Nat) -> Result<()> {
    let from = ic_cdk::caller();
    let fee = get_fee();
    pre_transfer_check(from, to, &value, &fee)?;
    charge_fee(from, fee.clone())?;
    let value = value.sub(fee);

    SHARD_BALANCES.with(|b| {
        let mut balances = b.borrow_mut();
        let mut from_balance = balances.entry(from).or_default();
        from_balance.sub_assign(value.clone());
        let mut to_balance = balances.entry(from).or_default();
        to_balance.add_assign(value.clone());
    });
    Ok(())
}

#[update(name = "shardTransferAndCall")]
#[candid_method(update, rename = "shardTransferAndCall")]
async fn transfer_and_call(to: Principal, value: Nat, notify: Func) -> Result<()> {
    let from = ic_cdk::caller();
    let fee = get_fee();
    pre_transfer_check(from, to, &value, &fee)?;
    charge_fee(from, fee.clone())?;
    let value = value.sub(fee);

    // put the funds in escrow
    let escrow_id = SHARD_BALANCES.with(|b| {
        let mut balances = b.borrow_mut();
        let mut from_balance = balances.entry(from).or_default();
        from_balance.sub_assign(value.clone());
        FUNDS_IN_ESCROW.with(|f| f.borrow_mut().deposit(value.clone()))
    });

    let send_funds_in_escrow = |destination: Principal| -> Result<()> {
        FUNDS_IN_ESCROW.with(|f| {
            f.borrow_mut().withdraw(escrow_id).map(|funds| {
                SHARD_BALANCES.with(|b| {
                    let mut balances = b.borrow_mut();
                    let mut dest_balance = balances.entry(destination).or_default();
                    dest_balance.add_assign(funds);
                })
            })
        })
    };

    // notify recipient
    // problem with this approach: the destination smart contract can only acknowledge receipt
    // but CANNOT immediately transfer the received funds because they are still in escrow.
    // This is not a problem for a DEX, but it might be a problem for other applications.
    let result: std::result::Result<((), ), _> =
        ic_cdk::call(notify.principal, &notify.method, (from, value)).await;
    match result {
        Ok(_) => {
            // send funds in escrow to destination
            send_funds_in_escrow(to)?;
            Ok(())
        }
        Err((rejection_code, details)) => {
            // revert transaction
            send_funds_in_escrow(from)?;
            Err((rejection_code, details).into())
        }
    }
}
