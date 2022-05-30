use std::cell::RefCell;

use candid::{candid_method, CandidType, Deserialize, Func, Principal, types::number::Nat};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

#[update(name = "wrap")]
#[candid_method(update)]
fn wrap() {}

#[update(name = "unwrap")]
#[candid_method(update)]
fn unwrap() {}
