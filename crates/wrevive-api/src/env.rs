#[cfg(not(any(test, feature = "off_chain")))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "off_chain"))]
use std::vec::Vec;

use crate::types::{Address, BlockNumber, H256, U256};
use pallet_revive_uapi::{CallFlags, ReturnErrorCode, ReturnFlags, StorageFlags};

/// Result type for contract calls.
/// 合约调用的结果类型。
pub type CallResult = core::result::Result<(), ReturnErrorCode>;

/// Contract host API: same surface for on-chain and off-chain implementations.
/// 合约链接口：链上与链下实现同一套 API。
pub trait Env {
    /// Returns the caller address.
    /// 返回调用方地址。
    fn caller(&self) -> Address;

    /// Store `value` at `key`; returns previous value length if any.
    /// 在 `key` 处存储 `value`；若有旧值则返回其长度。
    fn set_storage(&self, flags: StorageFlags, key: &[u8], value: &[u8]) -> Option<u32>;

    /// Load value at `key`; returns the value bytes.
    /// 获取 `key` 对应的值；返回值的字节。
    fn get_storage(
        &self,
        flags: StorageFlags,
        key: &[u8],
    ) -> Result<Vec<u8>, ReturnErrorCode>;

    /// Clear storage at `key`. On-chain uses set_storage_or_clear when key is 32 bytes, else set_storage(key, &[]).
    /// 清除 `key` 处的存储。链上在 key 为 32 字节时使用 set_storage_or_clear，否则 set_storage(key, &[])。
    fn clear_storage(&self, flags: StorageFlags, key: &[u8]) -> Option<u32>;

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
    fn address(&self) -> Address;

    /// Get the contract immutable data.
    /// 获取合约不可变数据。
    fn get_immutable_data(&self, output: &mut &mut [u8]);

    /// Set the contract immutable data.
    /// 设置合约不可变数据。
    fn set_immutable_data(&self, data: &[u8]);

    /// Returns the reducible balance of the current account.
    /// 返回当前账户的可减少余额。
    fn balance(&self) -> U256;

    /// Returns the reducible balance of the supplied address.
    /// 返回指定地址的可减少余额。
    fn balance_of(&self, addr: &[u8; 20]) -> U256;

    /// Returns the EIP-155 chain ID.
    /// 返回 EIP-155 链 ID。
    fn chain_id(&self) -> [u8; 32];

    /// Returns the price per ref_time, akin to the EVM GASPRICE opcode.
    /// 返回每个 ref_time 的价格，类似于 EVM GASPRICE 操作码。
    fn gas_price(&self) -> u64;

    /// Returns the base fee, akin to the EVM BASEFEE opcode.
    /// 返回基础费用，类似于 EVM BASEFEE 操作码。
    fn base_fee(&self) -> U256;

    /// Call (possibly transferring some amount of funds) into the specified account.
    /// 调用（可能转移一些资金）到指定账户。
    fn call(
        &self,
        flags: CallFlags,
        callee: &Address,
        ref_time_limit: u64,
        proof_size_limit: u64,
        deposit: &U256,
        value: &U256,
        input_data: &[u8],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult;

    /// Execute code in the context (storage, caller, value) of the current contract.
    /// 在当前合约的上下文（存储、调用者、值）中执行代码。
    fn delegate_call(
        &self,
        flags: CallFlags,
        address: &Address,
        ref_time_limit: u64,
        proof_size_limit: u64,
        deposit_limit: &U256,
        input_data: &[u8],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult;

    /// Transfer native balance from the current contract to the given account.
    /// No contract code is executed; use [Env::call] if you need to invoke the recipient.
    /// 合约向指定账户转帐（从当前合约余额转出 value 到 to）；不执行目标代码，仅转帐。
    fn transfer(&self, to: &Address, value: &U256) -> CallResult;

    /// Returns the origin address (initator of the call stack).
    /// 返回原始地址（调用栈的发起者）。
    fn origin(&self) -> [u8; 20];

    /// Returns the code hash for a specified contract address.
    /// 返回指定合约地址的代码哈希。
    fn code_hash(&self, addr: &[u8; 20]) -> H256;

    /// Returns the code size for a specified contract address.
    /// 返回指定合约地址的代码大小。
    fn code_size(&self, addr: &[u8; 20]) -> u64;


    /// Hash input using Keccak-256.
    /// 使用 Keccak-256 对输入进行哈希。
    fn hash_keccak_256(&self, input: &[u8]) -> H256;

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

    /// Returns the current block number.
    /// 返回当前区块高度。
    fn now(&self) -> BlockNumber;

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
    fn value_transferred(&self) -> U256;

    /// Same as [call] but with one-dimensional EVM gas. Adds EVM gas stipend for non-zero value. `u64::MAX` = uncapped.
    /// 与 call 相同但使用一维 EVM gas；非零 value 会加 stipend；gas = u64::MAX 表示无上限。
    fn call_evm(
        &self,
        flags: CallFlags,
        callee: &Address,
        gas: u64,
        value: &U256,
        input_data: &[u8],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult;

    /// Same as [delegate_call] but with one-dimensional EVM gas. `u64::MAX` = uncapped.
    /// 与 delegate_call 相同但使用一维 EVM gas；u64::MAX 表示无上限。
    fn delegate_call_evm(
        &self,
        flags: CallFlags,
        address: &Address,
        gas: u64,
        input_data: &[u8],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult;

    /// Returns the size of the return data.
    /// 返回返回数据的大小。
    fn return_data_size(&self) -> u64;

    /// Copy return data of the last call/instantiate into the supplied buffer at the given offset.
    /// 将上一次 call/instantiate 的返回数据从 offset 起写入 output。
    fn return_data_copy(&self, output: &mut &mut [u8], offset: u32);

    /// Returns the amount of Ethereum gas left.
    /// 返回剩余 Ethereum gas。
    fn gas_left(&self) -> u64;

    /// Returns the current block author (miner/validator address).
    /// 返回当前区块作者（矿工/验证者地址）。
    fn block_author(&self) -> Address;

    /// Returns the current block number (32-byte buffer in host; we return BlockNumber).
    /// 返回当前区块高度。
    fn block_number(&self) -> BlockNumber;

    /// Returns the block hash for the given block number.
    /// 返回指定区块高度的区块哈希。
    fn block_hash(&self, block_number: BlockNumber) -> H256;

    /// Reverts and consumes all gas (EVM INVALID opcode). Never returns.
    /// 消耗全部 gas 并 revert（EVM INVALID）；不返回。
    fn consume_all_gas(&self) -> !;

    /// Remove the calling account and transfer remaining free balance to beneficiary. Never returns.
    /// 销毁调用账户并将剩余余额转给 beneficiary；不返回。
    fn terminate(&self, beneficiary: &[u8; 20]) -> !;
}

