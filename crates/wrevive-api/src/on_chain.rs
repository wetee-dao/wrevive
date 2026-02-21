//! On-chain 实现：委托 `pallet_revive_uapi::HostFnImpl`，仅 PolkaVM/RISC-V 目标。

use pallet_revive_uapi::{HostFn, HostFnImpl, ReturnErrorCode, ReturnFlags, StorageFlags};

pub mod ext {
    use super::*;

    #[inline(always)]
    pub fn caller(output: &mut [u8; 20]) {
        HostFnImpl::caller(output);
    }

    #[inline(always)]
    pub fn set_storage(flags: StorageFlags, key: &[u8], value: &[u8]) -> Option<u32> {
        HostFnImpl::set_storage(flags, key, value)
    }

    #[inline(always)]
    pub fn get_storage(
        flags: StorageFlags,
        key: &[u8],
        output: &mut &mut [u8],
    ) -> Result<(), ReturnErrorCode> {
        HostFnImpl::get_storage(flags, key, output)
    }

    #[inline(always)]
    pub fn deposit_event(topics: &[[u8; 32]], data: &[u8]) {
        HostFnImpl::deposit_event(topics, data);
    }

    #[inline(always)]
    pub fn return_value(flags: ReturnFlags, return_value: &[u8]) -> ! {
        HostFnImpl::return_value(flags, return_value)
    }

    #[inline(always)]
    pub fn call_data_size() -> u64 {
        HostFnImpl::call_data_size()
    }

    #[inline(always)]
    pub fn call_data_copy(output: &mut [u8], offset: u32) {
        HostFnImpl::call_data_copy(output, offset);
    }
}
