#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
extern crate alloc;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: pvm_bump_allocator::BumpAllocator<1024> = pvm_bump_allocator::BumpAllocator::new();

use wrevive_api::{env, get_storage, set_storage, Mapping, ReturnFlags, StorageFlags};
use wrevive_macro::revive_contract;

#[revive_contract]
mod contract {
    use super::*;

    const EMPTY_TOPICS: &[[u8; 32]] = &[];

    const STORAGE_KEY_VALUE: &[u8] = b"value";
    const STORAGE_KEY_OWNER: &[u8] = b"owner";

    // 创建一个用于存储用户余额的 Mapping
    // key: 用户地址 [u8; 20], value: 余额 u64
    static BALANCE_MAPPING: Mapping = Mapping::new(b"balance");

    // 创建一个用于存储用户信息的 Mapping
    // key: 用户地址 [u8; 20], subkey: 信息类型 u8, value: 信息值 u32
    static USER_INFO_MAPPING: Mapping = Mapping::new(b"user_info");

    #[revive(constructor)]
    pub fn deploy() {
        let caller = env().caller();

        set_storage(StorageFlags::empty(), STORAGE_KEY_OWNER, &caller);
        let default_value: u32 = 0;
        set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &default_value);
    }

    #[revive(message)]
    pub fn set_value(value: u32) {
        set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &value);
        env().deposit_event(EMPTY_TOPICS, &value.to_le_bytes().as_slice());
    }

    #[revive(message)]
    pub fn get_value() -> u32 {
        get_storage::<_, u32>(StorageFlags::empty(), STORAGE_KEY_VALUE).unwrap_or(0)
    }

    #[revive(message)]
    pub fn set_owner(new_owner: [u8; 20], _v: u32) {
        let caller = env().caller();
        let current_owner = get_owner();
        if caller != current_owner {
            env().return_value(ReturnFlags::REVERT, &[]);
        } else {
            set_storage(StorageFlags::empty(), STORAGE_KEY_OWNER, &new_owner);
        }
    }

    #[revive(message)]
    pub fn get_owner() -> [u8; 20] {
        get_storage::<_, [u8; 20]>(StorageFlags::empty(), STORAGE_KEY_OWNER).unwrap_or([0u8; 20])
    }

    /// 设置用户余额（使用 Mapping）
    #[revive(message)]
    pub fn set_balance(user: [u8; 20], balance: u64) {
        BALANCE_MAPPING.set(env(), &user, &(), &balance);
    }

    /// 获取用户余额（使用 Mapping）
    #[revive(message)]
    pub fn get_balance(user: [u8; 20]) -> u64 {
        BALANCE_MAPPING.get(env(), &user, &()).unwrap_or(0)
    }

    /// 设置用户信息（使用 Mapping，带 subkey）
    #[revive(message)]
    pub fn set_user_info(user: [u8; 20], info_type: u8, value: u32) {
        USER_INFO_MAPPING.set(env(), &user, &info_type, &value);
    }

    /// 获取用户信息（使用 Mapping，带 subkey）
    #[revive(message)]
    pub fn get_user_info(user: [u8; 20], info_type: u8) -> u32 {
        USER_INFO_MAPPING.get(env(), &user, &info_type).unwrap_or(0)
    }

    /// 转账：从一个用户转移余额到另一个用户（使用 Mapping）
    #[revive(message)]
    pub fn transfer_balance(from: [u8; 20], to: [u8; 20], amount: u64) {
        let from_balance = BALANCE_MAPPING.get(env(), &from, &()).unwrap_or(0);
        if from_balance < amount {
            env().return_value(ReturnFlags::REVERT, &[]);
        }

        let to_balance = BALANCE_MAPPING.get(env(), &to, &()).unwrap_or(0);
        BALANCE_MAPPING.set(env(), &from, &(), &(from_balance - amount));
        BALANCE_MAPPING.set(env(), &to, &(), &(to_balance + amount));
    }
}

#[cfg(test)]
mod tests;

