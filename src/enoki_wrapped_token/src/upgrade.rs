use std::collections::HashMap;

use candid::{candid_method, CandidType, Deserialize, Principal, types::number::Nat};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

#[derive(Deserialize, CandidType)]
struct UpgradePayload {}

#[pre_upgrade]
fn pre_upgrade() {}

#[post_upgrade]
fn post_upgrade() {}
