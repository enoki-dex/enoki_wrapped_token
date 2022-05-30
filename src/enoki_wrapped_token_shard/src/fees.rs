use std::cell::RefCell;
use std::ops::{AddAssign, SubAssign};

use candid::{candid_method, CandidType, Deserialize, Principal, types::number::Nat};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

use crate::management::assert_is_manager_contract;

thread_local! {
    static ACCRUED_FEES: RefCell<FeeBalance> = RefCell::new(FeeBalance::default());
}

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
struct FeeBalance(Nat);

pub fn accept_fee(value: Nat) {
    ACCRUED_FEES.with(|f| f.borrow_mut().0.add_assign(value));
}
