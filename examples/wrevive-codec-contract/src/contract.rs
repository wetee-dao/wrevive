#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
extern crate alloc;

#[cfg(not(test))]
use alloc::vec::Vec;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: pvm_bump_allocator::BumpAllocator<1024> = pvm_bump_allocator::BumpAllocator::new();

use wrevive_api::{env, Encode, List, List2D, Mapping, ReturnFlags, Storage};
use wrevive_macro::{list, list_2d, mapping, revive_contract, storage};

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

    // 全局 List：自增 id，存 u64。用 list! 宏，
    const RECORDS: List<u32, u64> = list!(b"records");

    // 按用户维度的 List2D：每个 [u8;20] 下一条 u32 列表。用 list_2d! 宏
    const USER_ITEMS: List2D<[u8; 20], u32, u32> = list_2d!(b"user_items");

    /// 错误类型
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
    pub enum Error {
        InsufficientBalance,
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

    // ======================== List 示例 ========================

    /// 向全局 records 列表追加一条 u64，返回分配到的 id
    #[revive(message)]
    pub fn records_push(value: u64) -> Option<u32> {
        RECORDS.insert(env(), &value)
    }

    /// 按 id 取 records 中的值
    #[revive(message)]
    pub fn records_get(id: u32) -> u64 {
        RECORDS.get(env(), &id).unwrap_or(0)
    }

    /// 全局 records 长度
    #[revive(message)]
    pub fn records_len() -> u32 {
        RECORDS.len(env())
    }

    /// 分页：从 start 起取最多 size 条 (id, value)。返回长度 0 表示无数据或参数不合法
    #[revive(message)]
    pub fn records_list(start: u32, size: u32) -> Vec<(u32, u64)> {
        RECORDS.list(env(), start, size)
    }

    // ======================== List2D 示例（按用户） ========================

    /// 在指定用户下追加一条 u32，返回该用户下的 k2
    #[revive(message)]
    pub fn user_items_push(user: [u8; 20], value: u32) -> Option<u32> {
        USER_ITEMS.insert(env(), &user, &value)
    }

    /// 取用户 user 下第 k2 条
    #[revive(message)]
    pub fn user_items_get(user: [u8; 20], k2: u32) -> u32 {
        USER_ITEMS.get(env(), &user, k2).unwrap_or(0)
    }

    /// 用户 user 下的条目数量
    #[revive(message)]
    pub fn user_items_len(user: [u8; 20]) -> u32 {
        USER_ITEMS.len(env(), &user)
    }

    /// 分页：用户 user 下从 start 起取最多 size 条 (k2, value)
    #[revive(message)]
    pub fn user_items_list(user: [u8; 20], start: u32, size: u32) -> Vec<(u32, u32)> {
        USER_ITEMS.list(env(), &user, start, size)
    }
}

#[cfg(test)]
mod tests;

