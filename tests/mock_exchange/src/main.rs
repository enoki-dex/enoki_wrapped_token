use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::AddAssign;

#[allow(unused_imports)]
use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

#[init]
#[candid_method(init)]
async fn init(token_accepted: Principal) {
    STATE.with(|s| s.borrow_mut().token_accepted = token_accepted);
}

async fn register(token: Principal) {
    let response: Result<(enoki_wrapped_token_shared::types::Result<Principal>,), _> =
        ic_cdk::call(token, "startRegistration", ()).await;
    let assigned_shard = response.unwrap().0.unwrap();
    STATE.with(|s| s.borrow_mut().assigned_shard = assigned_shard);
    let me: Result<(Principal,), _> = ic_cdk::call(assigned_shard, "whoAmI", ()).await;
    let response: Result<(enoki_wrapped_token_shared::types::Result<()>,), _> =
        ic_cdk::call(token, "completeRegistration", (me.unwrap().0,)).await;
    response.unwrap().0.unwrap();

    let response: Result<(Principal,), _> = ic_cdk::call(assigned_shard, "whoAmI", ()).await;
    STATE.with(|s| s.borrow_mut().my_id_in_assigned_shard = response.unwrap().0);
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
    my_id_in_assigned_shard: Principal,
    deposits: HashMap<Principal, Nat>,
    pending_deposits: HashMap<u64, Principal>,
    last_id: u64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            token_accepted: Principal::anonymous(),
            assigned_shard: Principal::anonymous(),
            my_id_in_assigned_shard: Principal::anonymous(),
            deposits: Default::default(),
            pending_deposits: Default::default(),
            last_id: 0,
        }
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

async fn assert_from_token_shard(shard_id: Principal, destination: Principal) {
    assert_eq!(
        (shard_id, destination),
        STATE.with(|s| {
            let state = s.borrow();
            (state.assigned_shard, state.my_id_in_assigned_shard)
        })
    );
}

#[update(name = "initialize")]
#[candid_method(update)]
async fn initialize() {
    register(STATE.with(|s| s.borrow().token_accepted)).await;
}

#[update(name = "startDeposit")]
#[candid_method(update, rename = "startDeposit")]
fn start_deposit() -> (u64, Principal, Principal) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let shard_id = state.assigned_shard;
        let deposit_address = state.my_id_in_assigned_shard;
        let id = state.last_id;
        state.last_id += 1;
        state.pending_deposits.insert(id, ic_cdk::caller());
        (id, shard_id, deposit_address)
    })
}

#[update(name = "completeDeposit")]
#[candid_method(update, rename = "completeDeposit")]
async fn complete_deposit(deposit_id: u64, destination_address: Principal, value: Nat) {
    assert_from_token_shard(ic_cdk::caller(), destination_address).await;
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let original_caller = state
            .pending_deposits
            .remove(&deposit_id)
            .expect("deposit_id not found");
        state
            .deposits
            .entry(original_caller)
            .or_default()
            .add_assign(value);
    });
}

#[update(name = "withdrawAll")]
#[candid_method(update, rename = "withdrawAll")]
async fn withdraw_all(shard_id: Principal, to: Principal) {
    let amount = STATE
        .with(|s| s.borrow_mut().deposits.remove(&ic_cdk::caller()))
        .expect("no deposits found");
    let response: Result<(enoki_wrapped_token_shared::types::Result<()>,), _> = ic_cdk::call(
        STATE.with(|s| s.borrow().assigned_shard),
        "shardTransfer",
        (shard_id, to, amount),
    )
    .await;
    response.unwrap().0.unwrap();
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
