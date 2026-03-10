//! Example contract using wrevive-api: Storage, Mapping, List, List2D with SCALE codec.
//! 示例合约：使用 wrevive-api 的 Storage、Mapping、List、List2D，采用 SCALE 编解码。

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

mod datas;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: pvm_bump_allocator::BumpAllocator<1024> = pvm_bump_allocator::BumpAllocator::new();

use wrevive_api::{env, Address, Encode, Env, List, List2D, Mapping, Storage, Vec};
use wrevive_macro::{list, list_2d, mapping, revive_contract, storage};

pub use datas::*;

#[revive_contract]
pub mod contract {
    use super::*;
    use wrevive_api::Env;

    /// Event topics (empty for simple events). 事件主题（简单事件可为空）。
    const EMPTY_TOPICS: &[[u8; 32]] = &[];

    /// Single value storage; prefix = Blake2s256(b"value")[0..4].
    /// 单值存储；prefix 由 storage! 宏用 Blake2s256 取前 4 字节。
    const VALUE: Storage<u32> = storage!(b"value");

    /// Contract owner address (20 bytes). 合约所有者地址（20 字节）。
    const OWNER: Storage<Address> = storage!(b"owner");

    /// Cluster info; prefix = Blake2s256(b"cluster")[0..4].
    const CLUSTER: Storage<Cluster> = storage!(b"cluster");

    /// Balance per account: key = Address, value = u64.
    /// 用户余额：key = 用户地址，value = 余额。
    const BALANCE_MAPPING: Mapping<Address, u64> = mapping!(b"balance");

    /// User info by (address, info_type): value = u32 (e.g. score, level).
    /// 用户信息：key = (地址, 类型)，value = u32（如积分、等级）。
    const USER_INFO_MAPPING: Mapping<(Address, u8), u32> = mapping!(b"user_info");

    /// Global list: auto-increment id (u32), value u64. 全局列表：自增 id(u32)，值 u64。
    const RECORDS: List<u32, u64> = list!(b"records");

    /// Per-user list: each Address has a list of u32. 按用户维度的列表：每用户一条 u32 列表。
    const USER_ITEMS: List2D<Address, u32, u32> = list_2d!(b"user_items");

    /// Contract error type. 合约错误类型。
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
    pub enum Error {
        InsufficientBalance,
        Unauthorized,
    }

    /// Constructor: set caller as owner and init VALUE to the given initial_value.
    /// 构造函数：设置调用者为 owner，VALUE 初始为 initial_value。
    /// 
    /// # Parameters
    /// @param initial_value The initial value to set in the contract
    /// @param initial_value 合约初始化时设置的初始值
    /// 
    /// # Returns
    /// @return Returns Ok(()) if initialization succeeds, Err(Error) if failed
    /// @return 初始化成功返回 Ok(())，失败返回 Err(Error)
    /// 
    /// # Events
    /// Emits an event with the initial value when deployment succeeds
    /// 部署成功时发送包含初始值的事件
    #[revive(constructor)]
    pub fn deploy(initial_value: u32) -> Result<(), Error> {
        VALUE.set(&initial_value);
        OWNER.set(&env().caller());
        env().deposit_event(EMPTY_TOPICS, &initial_value.to_le_bytes().as_slice());
        Ok(())
    }

    /// Set the contract's stored value.
    /// 设置合约中存储的值。
    /// 
    /// # Parameters
    /// @param value The new value to store in the contract
    /// @param value 要在合约中存储的新值
    /// 
    /// # Returns
    /// @return Returns Ok(()) if value is set successfully
    /// @return 成功设置值返回 Ok(())
    /// 
    /// # Events
    /// Emits an event with the new value
    /// 发送包含新值的事件
    #[revive(message, write)]
    pub fn set_value(value: u32) -> Result<(), Error> {
        VALUE.set(&value);
        env().deposit_event(EMPTY_TOPICS, &value.to_le_bytes().as_slice());
        Ok(())
    }

    /// Get the current stored value.
    /// 获取当前存储的值。
    /// 
    /// # Returns
    /// @return Returns the current stored value, defaults to 0 if not set
    /// @return 返回当前存储的值，未设置时默认为 0
    #[revive(message)]
    pub fn get_value() -> u32 {
        VALUE.get().unwrap_or(0)
    }

    /// Get the current stored value as Option.
    /// 以 Option 类型获取当前存储的值。
    /// 
    /// # Returns
    /// @return Returns Some(value) if value is set, None otherwise
    /// @return 设置了值返回 Some(value)，否则返回 None
    #[revive(message)]
    pub fn get_value_option() -> Option<u32> {
        VALUE.get()
    }

    /// Set value using Solidity encoding; only current owner may call (else revert).
    /// 使用 Solidity 编码设置值；仅当前所有者可调用，否则 revert。
    /// 
    /// # Parameters
    /// @param value The new value to store (Solidity encoded)
    /// @param value 要存储的新值（Solidity 编码）
    /// 
    /// # Returns
    /// @return Returns Ok(()) if value is set successfully
    /// @return 成功设置值返回 Ok(())
    #[revive(message, write, sol)]
    pub fn set_value_sol(value: u32) -> Result<(), Error> {
        VALUE.set(&value);
        env().deposit_event(EMPTY_TOPICS, &value.to_le_bytes().as_slice());
        Ok(())
    }

    /// Get value using Solidity encoding; only current owner may call (else revert).
    /// 使用 Solidity 编码获取值；仅当前所有者可调用，否则 revert。
    /// 
    /// # Returns
    /// @return Returns the current stored value (Solidity encoded)
    /// @return 返回当前存储的值（Solidity 编码）
    #[revive(message, sol)]
    pub fn get_value_sol() -> u32 {
        VALUE.get().unwrap_or(0)
    }

    /// Get the cluster information stored in the contract.
    /// 获取合约中存储的集群信息。
    /// 
    /// # Returns
    /// @return Returns the cluster info, defaults to Cluster::default() if not set
    /// @return 返回集群信息，未设置时默认为 Cluster::default()
    #[revive(message)]
    pub fn get_cluster() -> Cluster {
        CLUSTER.get().unwrap_or(Cluster::default())
    }

    /// Set cluster information in the contract.
    /// 在合约中设置集群信息。
    /// 
    /// # Parameters
    /// @param cluster The cluster information to store
    /// @param cluster 要存储的集群信息
    /// 
    /// # Returns
    /// @return Returns Ok(()) if cluster is set successfully
    /// @return 成功设置集群信息返回 Ok(())
    #[revive(message, write)]
    pub fn set_cluster(cluster: Cluster) -> Result<(), Error> {
        CLUSTER.set(&cluster);
        Ok(())
    }

    /// Set new contract owner. Only current owner may call (else revert).
    /// 设置新的合约所有者。仅当前所有者可调用，否则 revert。
    /// 
    /// # Parameters
    /// @param new_owner The address of the new contract owner
    /// @param new_owner 新的合约所有者地址
    /// 
    /// # Returns
    /// @return Returns Ok(()) if ownership transfer succeeds, Err(Error::Unauthorized) if caller is not current owner
    /// @return 所有权转移成功返回 Ok(())，调用者不是当前所有者返回 Err(Error::Unauthorized)
    /// 
    /// # Security
    /// Only the current contract owner can transfer ownership to prevent unauthorized changes
    /// 只有当前合约所有者才能转移所有权，防止未授权的更改
    #[revive(message, write)]
    pub fn set_owner(new_owner: Address) -> Result<(), Error> {
        let caller = env().caller();
        let current_owner = get_owner();
        if caller != current_owner {
            return Err(Error::Unauthorized);
        }
        OWNER.set(&new_owner);
        Ok(())
    }

    /// Get the current contract owner address.
    /// 获取当前合约所有者地址。
    /// 
    /// # Returns
    /// @return Returns the current owner address, zero address if not set
    /// @return 返回当前所有者地址，未设置时返回零地址
    #[revive(message)]
    pub fn get_owner() -> Address {
        OWNER.get().unwrap_or(Address::zero())
    }

    /// Set balance for a specific user account using Mapping storage.
    /// 使用 Mapping 存储为特定用户账户设置余额。
    /// 
    /// # Parameters
    /// @param user The user address to set balance for
    /// @param user 要设置余额的用户地址
    /// @param balance The new balance value for the user
    /// @param balance 用户的新余额值
    /// 
    /// # Returns
    /// @return Returns Ok(()) if balance is set successfully
    /// @return 成功设置余额返回 Ok(())
    #[revive(message, write)]
    pub fn set_balance(user: Address, balance: u64) -> Result<(), Error> {
        BALANCE_MAPPING.set(&user, &balance);
        Ok(())
    }

    /// Get balance for a specific user account using Mapping storage.
    /// 使用 Mapping 存储获取特定用户账户的余额。
    /// 
    /// # Parameters
    /// @param user The user address to get balance for
    /// @param user 要获取余额的用户地址
    /// 
    /// # Returns
    /// @return Returns the user's current balance, defaults to 0 if not set
    /// @return 返回用户当前余额，未设置时默认为 0
    #[revive(message)]
    pub fn get_balance(user: Address) -> u64 {
        BALANCE_MAPPING.get(&user).unwrap_or(0)
    }

    /// Set user information with compound key (user address + info type).
    /// 使用复合键（用户地址 + 信息类型）设置用户信息。
    /// 
    /// # Parameters
    /// @param user The user address
    /// @param user 用户地址
    /// @param info_type The type of user information (e.g., score=1, level=2)
    /// @param info_type 用户信息类型（如：积分=1，等级=2）
    /// @param value The information value to store
    /// @param value 要存储的信息值
    /// 
    /// # Returns
    /// @return Returns Ok(()) if user info is set successfully
    /// @return 成功设置用户信息返回 Ok(())
    #[revive(message, write)]
    pub fn set_user_info(user: Address, info_type: u8, value: u32) -> Result<(), Error> {
        USER_INFO_MAPPING.set(&(user, info_type), &value);
        Ok(())
    }

    /// Get user information with compound key (user address + info type).
    /// 使用复合键（用户地址 + 信息类型）获取用户信息。
    /// 
    /// # Parameters
    /// @param user The user address
    /// @param user 用户地址
    /// @param info_type The type of user information to retrieve
    /// @param info_type 要获取的用户信息类型
    /// 
    /// # Returns
    /// @return Returns the user information value, defaults to 0 if not set
    /// @return 返回用户信息值，未设置时默认为 0
    #[revive(message)]
    pub fn get_user_info(user: Address, info_type: u8) -> u32 {
        USER_INFO_MAPPING
            .get(&(user, info_type))
            .unwrap_or(0)
    }

    /// Transfer balance from one account to another. Only the sender (`from`) may call (else revert).
    /// Reverts if `from` has insufficient balance. Self-transfer (from == to) is a no-op.
    /// 转账：仅 from 可发起；余额不足时 revert；from == to 时不操作。
    /// 
    /// # Parameters
    /// @param from The source address to transfer from (must be caller)
    /// @param from 转出地址（必须是调用者）
    /// @param to The destination address to transfer to
    /// @param to 转入地址
    /// @param amount The amount to transfer
    /// @param amount 转账金额
    /// 
    /// # Returns
    /// @return Returns Ok(()) if transfer succeeds
    /// @return Returns Err(Error::Unauthorized) if caller is not the from address
    /// @return Returns Err(Error::InsufficientBalance) if from address has insufficient balance
    /// @return 转账成功返回 Ok(())
    /// @return 调用者不是 from 地址返回 Err(Error::Unauthorized)
    /// @return from 地址余额不足返回 Err(Error::InsufficientBalance)
    /// 
    /// # Security
    /// Only the 'from' address can initiate the transfer to prevent unauthorized transfers
    /// 只有 'from' 地址可以发起转账，防止未授权的转账
    /// 
    /// # Edge Cases
    /// - Self-transfer (from == to) is treated as a no-op and returns Ok(())
    /// - Zero amount transfers are treated as no-ops
    /// - 自转账（from == to）被视为无操作并返回 Ok(())
    /// - 零金额转账被视为无操作
    #[revive(message, write)]
    pub fn transfer_balance(from: Address, to: Address, amount: u64) -> Result<(), Error> {
        let caller = env().caller();
        if caller != from {
            return Err(Error::Unauthorized);
        }
        if from == to || amount == 0 {
            return Ok(());
        }
        let from_balance = BALANCE_MAPPING.get(&from).unwrap_or(0);
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }
        let to_balance = BALANCE_MAPPING.get(&to).unwrap_or(0);
        BALANCE_MAPPING.set(&from, &(from_balance - amount));
        BALANCE_MAPPING.set(&to, &(to_balance + amount));
        Ok(())
    }

    // ======================== List 示例 ========================

    /// 向全局 records 列表追加一条 u64，返回分配到的 id
    #[revive(message, write)]
    pub fn records_push(value: u64) -> Option<u32> {
        RECORDS.insert(&value)
    }

    /// 按 id 取 records 中的值
    #[revive(message)]
    pub fn records_get(id: u32) -> u64 {
        RECORDS.get(&id).unwrap_or(0)
    }

    /// 全局 records 长度
    #[revive(message)]
    pub fn records_len() -> u32 {
        RECORDS.len()
    }

    /// 分页：从 start 起取最多 size 条 (id, value)。返回长度 0 表示无数据或参数不合法
    #[revive(message)]
    pub fn records_list(start: u32, size: u32) -> Vec<(u32, u64)> {
        RECORDS.list(start, size)
    }

    // ======================== List2D 示例（按用户） ========================

    /// 在指定用户下追加一条 u32，返回该用户下的 k2
    #[revive(message, write)]
    pub fn user_items_push(user: Address, value: u32) -> Option<u32> {
        USER_ITEMS.insert(&user, &value)
    }

    /// 取用户 user 下第 k2 条
    #[revive(message)]
    pub fn user_items_get(user: Address, k2: u32) -> u32 {
        USER_ITEMS.get(&user, k2).unwrap_or(0)
    }

    /// 用户 user 下的条目数量
    #[revive(message)]
    pub fn user_items_len(user: Address) -> u32 {
        USER_ITEMS.len(&user)
    }

    /// 分页：用户 user 下从 start 起取最多 size 条 (k2, value)
    #[revive(message)]
    pub fn user_items_list(user: Address, start: u32, size: u32) -> Vec<(u32, u32)> {
        USER_ITEMS.list(&user, start, size)
    }
}

#[cfg(test)]
mod tests;
