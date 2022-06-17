use std::cell::RefCell;

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

use crate::management::assert_is_owner;

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
pub struct Metadata {
    pub logo: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub underlying_token: Principal,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            logo: "".to_string(),
            name: "".to_string(),
            symbol: "".to_string(),
            decimals: 0,
            underlying_token: Principal::anonymous(),
        }
    }
}

thread_local! {
    static METADATA: RefCell<Metadata> = RefCell::new(Metadata::default());
}

pub fn export_stable_storage() -> (Metadata, ) {
    (METADATA.with(|d| d.take()), )
}

pub fn import_stable_storage(metadata: Metadata) {
    METADATA.with(|d| d.replace(metadata));
}

pub fn init_metadata(metadata: Metadata) {
    METADATA.with(|d| d.replace(metadata));
}

pub fn get_underlying_token() -> Principal {
    METADATA.with(|d| d.borrow().underlying_token)
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
fn set_logo(logo: String) {
    assert_is_owner().unwrap();
    METADATA.with(|d| d.borrow_mut().logo = logo);
}
