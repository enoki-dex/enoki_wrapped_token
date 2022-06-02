use std::cell::RefCell;
use std::ops::AddAssign;

use candid::{candid_method, CandidType, Deserialize, types::number::Nat};
use ic_cdk_macros::*;

use crate::stable::StableFeeBalance;

thread_local! {
    static ACCRUED_FEES: RefCell<FeeBalance> = RefCell::new(FeeBalance::default());
}

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
pub struct FeeBalance(pub Nat);

pub fn accept_fee(value: Nat) {
    ACCRUED_FEES.with(|f| f.borrow_mut().0.add_assign(value));
}

pub fn export_stable_storage() -> (StableFeeBalance, ) {
    let fee_balance: StableFeeBalance = ACCRUED_FEES.with(|b| b.take()).into();
    (fee_balance, )
}

pub fn import_stable_storage(fee_balance: StableFeeBalance) {
    ACCRUED_FEES.with(|b| b.replace(fee_balance.into()));
}

#[query(name = "getAccruedFees")]
#[candid_method(query, rename = "getAccruedFees")]
pub fn get_accrued_fees() -> Nat {
    ACCRUED_FEES.with(|d| d.borrow().0.clone())
}