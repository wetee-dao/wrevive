#[cfg(not(any(test, feature = "off_chain")))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "off_chain"))]
use std::vec::Vec;

use pallet_revive_uapi::{CallFlags, ReturnErrorCode, ReturnFlags, StorageFlags};

/// Result type for contract calls.
/// 合约调用的结果类型。
pub type CallResult = core::result::Result<(), ReturnErrorCode>;

/// Contract host API: same surface for on-chain and off-chain implementations.
/// 合约链接口：链上与链下实现同一套 API。
pub trait Env {
    /// Returns the caller address (20 bytes).
    /// 返回调用方地址（20 字节）。
    fn caller(&self) -> [u8; 20];

    /// Store `value` at `key`; returns previous value length if any.
    /// 在 `key` 处存储 `value`；若有旧值则返回其长度。
    fn set_storage_bytes(&self, flags: StorageFlags, key: &[u8], value: &[u8]) -> Option<u32>;

    /// Load value at `key`; returns the value bytes.
    /// 获取 `key` 对应的值；返回值的字节。
    fn get_storage_bytes(
        &self,
        flags: StorageFlags,
        key: &[u8],
    ) -> Result<Vec<u8>, ReturnErrorCode>;

    /// Emit an event with `topics` and `data`.
    /// 发出事件，主题为 `topics`，数据为 `data`。
    fn deposit_event(&self, topics: &[[u8; 32]], data: &[u8]);

    /// Return from the contract (never returns; may revert).
    /// 从合约返回（不返回；可能 revert）。
    fn return_value(&self, flags: ReturnFlags, return_value: &[u8]) -> !;

    /// Size in bytes of the current call input.
    /// 当前调用输入的字节长度。
    fn call_data_size(&self) -> u64;

    /// Copy call input bytes from `offset`; returns the copied bytes.
    /// 从 `offset` 起复制调用输入；返回复制的字节。
    fn call_data_copy(&self, offset: u32, len: usize) -> Vec<u8>;

    /// Returns the address of the current contract.
    /// 返回当前合约地址。
    fn address(&self) -> [u8; 20];

    /// Get the contract immutable data.
    /// 获取合约不可变数据。
    fn get_immutable_data(&self, output: &mut &mut [u8]);

    /// Set the contract immutable data.
    /// 设置合约不可变数据。
    fn set_immutable_data(&self, data: &[u8]);

    /// Returns the reducible balance of the current account.
    /// 返回当前账户的可减少余额。
    fn balance(&self) -> [u8; 32];

    /// Returns the reducible balance of the supplied address.
    /// 返回指定地址的可减少余额。
    fn balance_of(&self, addr: &[u8; 20]) -> [u8; 32];

    /// Returns the EIP-155 chain ID.
    /// 返回 EIP-155 链 ID。
    fn chain_id(&self) -> [u8; 32];

    /// Returns the price per ref_time, akin to the EVM GASPRICE opcode.
    /// 返回每个 ref_time 的价格，类似于 EVM GASPRICE 操作码。
    fn gas_price(&self) -> u64;

    /// Returns the base fee, akin to the EVM BASEFEE opcode.
    /// 返回基础费用，类似于 EVM BASEFEE 操作码。
    fn base_fee(&self) -> [u8; 32];

    /// Call (possibly transferring some amount of funds) into the specified account.
    /// 调用（可能转移一些资金）到指定账户。
    fn call(
        &self,
        flags: CallFlags,
        callee: &[u8; 20],
        ref_time_limit: u64,
        proof_size_limit: u64,
        deposit: &[u8; 32],
        value: &[u8; 32],
        input_data: &[u8],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult;

    /// Returns the origin address (initator of the call stack).
    /// 返回原始地址（调用栈的发起者）。
    fn origin(&self) -> [u8; 20];

    /// Returns the code hash for a specified contract address.
    /// 返回指定合约地址的代码哈希。
    fn code_hash(&self, addr: &[u8; 20]) -> [u8; 32];

    /// Returns the code size for a specified contract address.
    /// 返回指定合约地址的代码大小。
    fn code_size(&self, addr: &[u8; 20]) -> u64;

    /// Execute code in the context (storage, caller, value) of the current contract.
    /// 在当前合约的上下文（存储、调用者、值）中执行代码。
    fn delegate_call(
        &self,
        flags: CallFlags,
        address: &[u8; 20],
        ref_time_limit: u64,
        proof_size_limit: u64,
        deposit_limit: &[u8; 32],
        input_data: &[u8],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult;

    /// Hash input using Keccak-256.
    /// 使用 Keccak-256 对输入进行哈希。
    fn hash_keccak_256(&self, input: &[u8]) -> [u8; 32];

    /// Load 32 bytes from call data at the given offset.
    /// 从给定偏移量的调用数据中加载 32 字节。
    fn call_data_load(&self, offset: u32) -> [u8; 32];

    /// Instantiate a contract.
    /// 实例化合约。
    fn instantiate(
        &self,
        flags: CallFlags,
        code_hash: &[u8; 32],
        ref_time_limit: u64,
        proof_size_limit: u64,
        deposit: &[u8; 32],
        value: &[u8; 32],
        input_data: &[u8],
        address: &mut [u8; 20],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult;

    /// Returns the current timestamp.
    /// 返回当前时间戳。
    fn now(&self) -> [u8; 32];

    /// Returns the gas limit.
    /// 返回 gas 限制。
    fn gas_limit(&self) -> u64;

    /// Set storage or clear it if value is zero.
    /// 设置存储，如果值为零则清除。
    fn set_storage_or_clear(&self, flags: StorageFlags, key: &[u8; 32], value: &[u8; 32]) -> Option<u32>;

    /// Get storage or return zero if not found.
    /// 获取存储，如果未找到则返回零。
    fn get_storage_or_zero(&self, flags: StorageFlags, key: &[u8; 32]) -> [u8; 32];

    /// Returns the value transferred in the current call.
    /// 返回当前调用中转移的值。
    fn value_transferred(&self) -> [u8; 32];

    /// Convert weight to fee.
    /// 将权重转换为费用。
    fn weight_to_fee(&self, ref_time_limit: u64, proof_size_limit: u64) -> [u8; 32];

    /// Returns the size of the return data.
    /// 返回返回数据的大小。
    fn return_data_size(&self) -> u64;
}
