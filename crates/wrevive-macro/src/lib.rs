//! # wrevive-macro
//!
//! ink!-style procedural macros for PolkaVM (pallet-revive) contracts:
//! `#[revive(constructor)]` and `#[revive(message)]` / `#[revive(message, selector = 0x...)]`.
//!
//! ## Usage
//!
//! Apply `#[revive_contract]` or `#[revive_contract(encoding = "codec")]` / `#[revive_contract(encoding = "sol")]` on a module.
//! The module must contain exactly one `#[revive(constructor)] fn deploy() { ... }` and zero or more `#[revive(message)]` functions.
//! **All contract functions (constructor and messages) must have a return value** — smart contracts must produce an execution result.
//!
//! ## Generated code
//!
//! - **deploy()** — `extern "C"` entry; **call()** — `extern "C"` entry dispatching by selector;
//! - **ABI file** — written at compile time to `target/contract/{name}.json`.

extern crate proc_macro;

mod abi;
mod attrs;
mod codegen;
mod contract;
mod manifest;
mod prefix;
mod storage;
mod type_abi;
mod type_reg;

#[proc_macro_attribute]
pub fn revive_contract(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    contract::revive_contract_impl(attr, item)
}

#[proc_macro]
pub fn storage(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    storage::storage_impl(input)
}

#[proc_macro]
pub fn mapping(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    storage::mapping_impl(input)
}

#[proc_macro]
pub fn list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    storage::list_impl(input)
}

#[proc_macro]
pub fn list_2d(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    storage::list_2d_impl(input)
}
