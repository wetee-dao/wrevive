//! # Chain API trait
//!
//! Abstraction over the contract host interface, similar to ink!'s
//! [Environment](https://github.com/use-ink/ink/tree/master/crates/engine).
//! Both on_chain (HostFnImpl) and off_chain (Engine) implement the same API.
//!
//! Chain API 抽象：与 ink [Environment](https://github.com/use-ink/ink/tree/master/crates/engine) 类似，
//! 统一 on_chain（链上 HostFnImpl）与 off_chain（测试 Engine）两套实现。

use pallet_revive_uapi::{ReturnErrorCode, ReturnFlags, StorageFlags};

/// Contract host API: same surface for on-chain and off-chain implementations.
/// 合约链接口：链上与链下实现同一套 API。
pub trait ChainApi {
    /// Write the caller address (20 bytes) into `output`.
    /// 将调用方地址（20 字节）写入 `output`。
    fn caller(&self, output: &mut [u8; 20]);

    /// Store `value` at `key`; returns previous value length if any.
    /// 在 `key` 处存储 `value`；若有旧值则返回其长度。
    fn set_storage(&self, flags: StorageFlags, key: &[u8], value: &[u8]) -> Option<u32>;

    /// Load value at `key` into `output`; may shrink `output` to bytes read.
    /// 将 `key` 对应的值写入 `output`；可能缩短 `output` 为实际读取长度。
    fn get_storage(
        &self,
        flags: StorageFlags,
        key: &[u8],
        output: &mut &mut [u8],
    ) -> Result<(), ReturnErrorCode>;

    /// Emit an event with `topics` and `data`.
    /// 发出事件，主题为 `topics`，数据为 `data`。
    fn deposit_event(&self, topics: &[[u8; 32]], data: &[u8]);

    /// Return from the contract (never returns; may revert).
    /// 从合约返回（不返回；可能 revert）。
    fn return_value(&self, flags: ReturnFlags, return_value: &[u8]) -> !;

    /// Size in bytes of the current call input.
    /// 当前调用输入的字节长度。
    fn call_data_size(&self) -> u64;

    /// Copy call input bytes from `offset` into `output`.
    /// 从 `offset` 起将调用输入复制到 `output`。
    fn call_data_copy(&self, output: &mut [u8], offset: u32);
}
