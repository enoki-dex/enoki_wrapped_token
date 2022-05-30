use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, types::number::Nat, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use enoki_wrapped_token_shared::types::*;

use crate::management::assert_is_owner;

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
pub struct Metadata {
    pub logo: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

thread_local! {
    static METADATA: RefCell<Metadata> = RefCell::new(Metadata::default());
}

#[query(name = "getLogo")]
#[candid_method(query, rename = "getLogo")]
fn get_logo() -> String {
    METADATA.with(|d| d.borrow().logo.clone())
}

#[query(name = "name")]
#[candid_method(query)]
fn name() -> String {
    METADATA.with(|d| d.borrow().name.clone())
}

#[query(name = "symbol")]
#[candid_method(query)]
fn symbol() -> String {
    METADATA.with(|d| d.borrow().symbol.clone())
}

#[query(name = "decimals")]
#[candid_method(query)]
fn decimals() -> u8 {
    METADATA.with(|d| d.borrow().decimals)
}

#[query(name = "getMetadata")]
#[candid_method(query, rename = "getMetadata")]
fn get_metadata() -> Metadata {
    METADATA.with(|d| d.borrow().clone())
}

#[update(name = "setLogo")]
#[candid_method(update, rename = "setLogo")]
fn set_logo(logo: String) -> Result<()> {
    assert_is_owner()?;
    METADATA.with(|d| d.borrow_mut().logo = logo);
    Ok(())
}
