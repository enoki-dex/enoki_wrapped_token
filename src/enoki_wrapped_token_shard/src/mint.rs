use candid::{candid_method, CandidType, Deserialize, Func, Principal, types::number::Nat};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

use crate::balances::{decrease_balance, increase_balance};
use crate::fees::accept_fee;
use crate::interfaces::dip20::DIP20;
use crate::management;

#[update(name = "wrap")]
#[candid_method(update)]
async fn wrap(amount: Nat) -> Result<()> {
    let caller = ic_cdk::caller();
    let (token, underlying_fee) = get_underlying_token_and_fee().await;
    let amount_to_credit = deposit_token(caller, amount, token, underlying_fee).await?;
    increase_balance(caller, amount_to_credit);

    Ok(())
}

#[update(name = "unwrap")]
#[candid_method(update)]
async fn unwrap(amount: Nat, to: Principal) -> Result<()> {
    let caller = ic_cdk::caller();
    let fee = management::get_fee();
    if amount <= fee {
        return Err(TxError::InsufficientBalance);
    }

    decrease_balance(caller, amount.clone())?;
    accept_fee(fee.clone());
    let amount = amount - fee; // when reverting, do not refund fee

    let (token, underlying_fee) = get_underlying_token_and_fee().await;

    if let Err(_) = withdraw_token(amount.clone(), to, token, underlying_fee).await {
        increase_balance(caller, amount);
        return Err(TxError::UnderlyingTransferFailure);
    }

    Ok(())
}

async fn get_underlying_token_and_fee() -> (DIP20, Nat) {
    let token = DIP20::new(management::get_underlying());
    let dip_fee = token.get_metadata().await.fee;
    (token, dip_fee)
}

async fn deposit_token(caller: Principal, amount: Nat, token: DIP20, fee: Nat) -> Result<Nat> {
    let allowance = token.allowance(caller, ic_cdk::api::id()).await;
    if allowance < amount {
        return Err(TxError::InsufficientBalance);
    }
    let amount = amount - fee;

    token
        .transfer_from(caller, ic_cdk::api::id(), amount.clone())
        .await
        .map_err(|_| TxError::UnderlyingTransferFailure)?;

    Ok(amount)
}

async fn withdraw_token(amount: Nat, to: Principal, token: DIP20, fee: Nat) -> Result<()> {
    token
        .transfer(to, amount - fee)
        .await
        .map_err(|_| TxError::UnderlyingTransferFailure)?;

    Ok(())
}
