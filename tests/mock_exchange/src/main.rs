use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::AddAssign;

#[allow(unused_imports)]
use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::ShardedTransferNotification;

#[init]
#[candid_method(init)]
async fn init(token_accepted: Principal) {
    STATE.with(|s| s.borrow_mut().token_accepted = token_accepted);
}

async fn register(token: Principal) {
    let response: Result<(Principal,), _> = ic_cdk::call(token, "register", (ic_cdk::id(),)).await;
    let assigned_shard = response.unwrap().0;
    STATE.with(|s| s.borrow_mut().assigned_shard = assigned_shard);
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}

#[derive(CandidType, Clone, Debug)]
struct State {
    token_accepted: Principal,
    assigned_shard: Principal,
    deposits: HashMap<Principal, Nat>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            token_accepted: Principal::anonymous(),
            assigned_shard: Principal::anonymous(),
            deposits: Default::default(),
        }
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

async fn assert_from_token_shard(shard_id: Principal) {
    assert_eq!(shard_id, STATE.with(|s| { s.borrow().assigned_shard }));
}

#[update(name = "initialize")]
#[candid_method(update)]
async fn initialize() {
    register(STATE.with(|s| s.borrow().token_accepted)).await;
}

#[query(name = "getDepositShardId")]
#[candid_method(query, rename = "getDepositShardId")]
fn get_deposit_shard_id() -> Principal {
    STATE.with(|s| s.borrow().assigned_shard)
}

#[update(name = "deposit")]
#[candid_method(update)]
async fn deposit(notification: ShardedTransferNotification) -> String {
    assert_from_token_shard(ic_cdk::caller()).await;
    assert_eq!(notification.to, ic_cdk::id());
    STATE.with(|s| {
        s.borrow_mut()
            .deposits
            .entry(notification.from)
            .or_default()
            .add_assign(notification.value);
    });
    "OK".to_string()
}

#[update(name = "withdrawAll")]
#[candid_method(update, rename = "withdrawAll")]
async fn withdraw_all(shard_id: Principal, to: Principal) {
    let amount = STATE
        .with(|s| s.borrow_mut().deposits.remove(&ic_cdk::caller()))
        .expect("no deposits found");
    let response: Result<(), _> = ic_cdk::call(
        STATE.with(|s| s.borrow().assigned_shard),
        "shardTransfer",
        (shard_id, to, amount),
    )
    .await;
    response.unwrap();
}

#[query(name = "balance")]
#[candid_method(query)]
fn balance() -> Nat {
    STATE.with(|s| {
        s.borrow()
            .deposits
            .get(&ic_cdk::caller())
            .cloned()
            .unwrap_or_default()
    })
}
