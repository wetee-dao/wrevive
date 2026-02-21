//! Chain API 抽象：与 ink [Environment](https://github.com/use-ink/ink/tree/master/crates/engine) 类似，
//! 统一 on_chain（链上 HostFnImpl）与 off_chain（测试 Engine）两套实现。

use pallet_revive_uapi::{ReturnErrorCode, ReturnFlags, StorageFlags};

/// 合约链接口：链上与链下实现同一套 API。
pub trait ChainApi {
    fn caller(&self, output: &mut [u8; 20]);
    fn set_storage(&self, flags: StorageFlags, key: &[u8], value: &[u8]) -> Option<u32>;
    fn get_storage(
        &self,
        flags: StorageFlags,
        key: &[u8],
        output: &mut &mut [u8],
    ) -> Result<(), ReturnErrorCode>;
    fn deposit_event(&self, topics: &[[u8; 32]], data: &[u8]);
    fn return_value(&self, flags: ReturnFlags, return_value: &[u8]) -> !;
    fn call_data_size(&self) -> u64;
    fn call_data_copy(&self, output: &mut [u8], offset: u32);
}
