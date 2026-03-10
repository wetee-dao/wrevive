//! Example contract using wrevive-api: Storage, Mapping, List, List2D with SCALE codec.
//! 示例合约：使用 wrevive-api 的 Storage、Mapping、List、List2D，采用 SCALE 编解码。

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: pvm_bump_allocator::BumpAllocator<1024> = pvm_bump_allocator::BumpAllocator::new();

use pallet_revive_uapi::CallFlags;
use wrevive_api::{Address, Encode, ReturnFlags, Storage, U256, Env, env};
use wrevive_macro::{revive_contract, storage};

#[revive_contract]
pub mod contract {
    use super::*;

    /// Contract error type. 合约错误类型。
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
    pub enum Error {
        InsufficientBalance,
        Unauthorized,
        AddressNotFound,
    }

    const CONTRACT: Storage<Address> = storage!(b"contract");

    /// Constructor: set caller as owner and init VALUE to the given initial_value.
    /// 构造函数：设置调用者为 owner，VALUE 初始为 initial_value。
    #[revive(constructor)]
    pub fn deploy(contract: Address) -> Result<(), Error> {
        CONTRACT.set(&contract);
        Ok(())
    }

    #[revive(message)]
    pub fn get_contract() -> Address {
        CONTRACT.get().unwrap_or(Address::zero())
    }

    #[revive(message)]
    pub fn set_contract(contract: Address) -> Result<(), Error> {
        CONTRACT.set(&contract);
        Ok(())
    }

    #[revive(fallback)]
    pub fn fallback() {
        let api = env();
        let callee = CONTRACT.get().unwrap_or(Address::zero());
        if callee == Address::zero() {
            let error = Error::AddressNotFound;
            api.return_value(ReturnFlags::REVERT, &Encode::encode(&error));
            return;
        }
        let call_data_len = api.call_data_size() as usize;
        let call_data = api.call_data_copy(0, call_data_len);

        let result = api.delegate_call(
            CallFlags::empty(),
            &callee,
            u64::MAX,
            u64::MAX,
            &U256::MAX,
            &call_data,
            None,
        );

        let len = api.return_data_size() as usize;
        let mut full = alloc::vec![0u8; len];
        let mut slice = full.as_mut_slice();
        api.return_data_copy(&mut slice, 0);

        let flags = match result {
            Ok(()) => ReturnFlags::empty(),
            Err(_) => ReturnFlags::REVERT,
        };

        api.return_value(flags, &full);
    }
}

#[cfg(test)]
mod tests;
