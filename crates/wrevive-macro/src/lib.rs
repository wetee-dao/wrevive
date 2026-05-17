//! # wrevive-macro
//!
//! Procedural macros for PolkaVM (pallet-revive) contracts:
//! `#[revive(constructor)]` and `#[revive(message)]` / `#[revive(message, selector = 0x...)]`.
//!
//! ## Features
//!
//! - **Keccak-256 selectors** — Ethereum-compatible 4-byte function selectors
//! - **Dual encoding** — SCALE (`codec`) and Solidity ABI (`sol`) encoding modes
//!
//! ## Usage
//!
//! Apply `#[revive_contract]` on a module. Default encoding is SCALE (codec).
//! Use `#[revive_contract(encoding = "sol")]` for Solidity ABI encoding.
//! Per-function override: `#[revive(message, sol)]` or `#[revive(message, codec)]`.
//! The module must contain exactly one `#[revive(constructor)]` and zero or more `#[revive(message)]` functions.
//! **All contract functions must have a return value** — smart contracts must produce an execution result.
//!
//! ## Generated code
//!
//! - **deploy()** — `extern "C"` entry; **call()** — `extern "C"` entry dispatching by Keccak-256 selector;
//! - **ABI file** — written at compile time to `target/contract/{name}.json`.

extern crate proc_macro;

mod abi;
mod attrs;
mod codegen;
mod contract;
mod docs;
mod interface;
mod manifest;
mod prefix;
mod storage;
mod type_reg;

/// The main contract macro - generates deploy/call entry points and ABI.
/// 主合约宏 - 生成 deploy/call 入口点和 ABI。
///
/// # English
/// Proc macro attribute that transforms a module into a PolkaVM contract.
/// Generates deploy() and call() entry points along with ABI JSON.
///
/// # 中文
/// 将模块转换为 PolkaVM 合约的过程宏属性。
/// 生成 deploy() 和 call() 入口点以及 ABI JSON。
#[proc_macro_attribute]
pub fn revive_contract(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    contract::revive_contract_impl(attr, item)
}

/// Defines a contract storage cell - a single named storage entry.
/// 定义合约存储单元 - 单个命名存储条目。
///
/// # English
/// Macro for declaring a contract storage cell with a type and optional initial value.
///
/// # 中文
/// 用于声明带有类型和可选初始值的合约存储单元的宏。
#[proc_macro]
pub fn storage(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    storage::storage_impl(input)
}

/// Defines a key-value mapping storage.
/// 定义键值映射存储。
///
/// # English
/// Macro for declaring a storage mapping from keys to values.
///
/// # 中文
/// 用于声明从键到值的存储映射的宏。
#[proc_macro]
pub fn mapping(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    storage::mapping_impl(input)
}

/// Defines a one-dimensional list storage.
/// 定义一维列表存储。
///
/// # English
/// Macro for declaring a storage list with auto-incrementing indices.
///
/// # 中文
/// 用于声明带有自增索引的存储列表的宏。
#[proc_macro]
pub fn list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    storage::list_impl(input)
}

/// Defines a two-dimensional list storage.
/// 定义二维列表存储。
///
/// # English
/// Macro for declaring a storage list indexed by an outer key and inner auto-increment.
///
/// # 中文
/// 用于声明由外键和内部自增索引的存储列表的宏。
#[proc_macro]
pub fn list_2d(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    storage::list_2d_impl(input)
}
