#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
extern crate alloc;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: pvm_bump_allocator::BumpAllocator<1024> = pvm_bump_allocator::BumpAllocator::new();

use wrevive_api::{env, Mapping, ReturnFlags, Storage};
use wrevive_macro::{mapping, revive_contract, storage};

#[revive_contract]
mod contract {
    use super::*;

    const EMPTY_TOPICS: &[[u8; 32]] = &[];

    // 存储值（prefix 使用 Blake2s256 取前 4 字节）
    const VALUE: Storage<u32> = storage!(b"value");

    // 存储所有者
    const OWNER: Storage<[u8; 20]> = storage!(b"owner");

    // 创建一个用于存储用户余额的 Mapping：key = 用户地址 [u8; 20], value = 余额 u64
    const BALANCE_MAPPING: Mapping<[u8; 20], u64> = mapping!(b"balance");

    // 创建一个用于存储用户信息的 Mapping：key = (用户地址, 信息类型), value = u32
    const USER_INFO_MAPPING: Mapping<([u8; 20], u8), u32> = mapping!(b"user_info");

    /// 错误类型
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Error {
        InsufficientBalance,
    }
    impl AsRef<[u8]> for Error {
        fn as_ref(&self) -> &[u8] {
            match *self {
                Error::InsufficientBalance => b"InsufficientBalance",
            }
        }
    }

    #[revive(constructor)]
    pub fn deploy() -> Result<(), Error> {
        let caller = env().caller();

        OWNER.set(env(), &caller);
        let default_value: u32 = 0;
        VALUE.set(env(), &default_value);

        Ok(())
    }

    #[revive(message)]
    pub fn set_value(value: u32) -> Result<(), Error> {
        VALUE.set(env(), &value);
        env().deposit_event(EMPTY_TOPICS, &value.to_le_bytes().as_slice());
        Ok(())
    }

    #[revive(message)]
    pub fn get_value() -> u32 {
        VALUE.get(env()).unwrap_or(0)
    }

    #[revive(message)]
    pub fn set_owner(new_owner: [u8; 20], _v: u32) {
        let caller = env().caller();
        let current_owner = get_owner();
        if caller != current_owner {
            env().return_value(ReturnFlags::REVERT, &[]);
        } else {
            OWNER.set(env(), &new_owner);
        }
    }

    #[revive(message)]
    pub fn get_owner() -> [u8; 20] {
        OWNER.get(env()).unwrap_or([0u8; 20])
    }

    /// 设置用户余额（使用 Mapping）
    #[revive(message)]
    pub fn set_balance(user: [u8; 20], balance: u64) {
        BALANCE_MAPPING.set(env(), &user, &balance);
    }

    /// 获取用户余额（使用 Mapping）
    #[revive(message)]
    pub fn get_balance(user: [u8; 20]) -> u64 {
        BALANCE_MAPPING.get(env(), &user).unwrap_or(0)
    }

    /// 设置用户信息（key = (user, info_type)）
    #[revive(message)]
    pub fn set_user_info(user: [u8; 20], info_type: u8, value: u32) {
        USER_INFO_MAPPING.set(env(), &(user, info_type), &value);
    }

    /// 获取用户信息
    #[revive(message)]
    pub fn get_user_info(user: [u8; 20], info_type: u8) -> u32 {
        USER_INFO_MAPPING.get(env(), &(user, info_type)).unwrap_or(0)
    }

    /// 转账：从一个用户转移余额到另一个用户（使用 Mapping）
    #[revive(message)]
    pub fn transfer_balance(from: [u8; 20], to: [u8; 20], amount: u64) {
        let from_balance = BALANCE_MAPPING.get(env(), &from).unwrap_or(0);
        if from_balance < amount {
            env().return_value(ReturnFlags::REVERT, &[]);
        }

        let to_balance = BALANCE_MAPPING.get(env(), &to).unwrap_or(0);
        BALANCE_MAPPING.set(env(), &from, &(from_balance - amount));
        BALANCE_MAPPING.set(env(), &to, &(to_balance + amount));
    }
}

#[cfg(test)]
mod tests;

