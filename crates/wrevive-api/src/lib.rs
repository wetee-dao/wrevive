//! pallet_revive_uapi 封装：**on_chain** / **off_chain** 两套接口，参考 [ink engine](https://github.com/use-ink/ink/tree/master/crates/engine)。
//!
//! - **on_chain**（默认）：`ext` 委托 `HostFnImpl`，仅 PolkaVM/RISC-V，用于链上部署。
//! - **off_chain**（feature `off_chain`）：`ext` 使用内存 `Engine`，提供 test_api（set_caller、set_call_data、get_storage_value、take_events 等），用于单元测试。

#![cfg_attr(not(feature = "off_chain"), no_std)]

pub mod chain_api;

pub use pallet_revive_uapi::{input, HostFn, ReturnFlags, StorageFlags};

#[cfg(all(not(feature = "off_chain"), target_arch = "riscv64"))]
pub mod on_chain;

#[cfg(all(not(feature = "off_chain"), target_arch = "riscv64"))]
pub use on_chain::ext;

#[cfg(feature = "off_chain")]
pub mod off_chain;

#[cfg(feature = "off_chain")]
pub use off_chain::{with_engine, Engine};
#[cfg(feature = "off_chain")]
pub use off_chain::ext;

#[cfg(feature = "off_chain")]
#[cfg(test)]
mod tests {
    use super::{off_chain, ext, StorageFlags};

    #[test]
    fn off_chain_engine_storage_and_caller() {
        off_chain::with_engine(|e| {
            e.set_caller([1u8; 20]);
            e.set_call_data(&[]);
        });
        let mut caller = [0u8; 20];
        ext::caller(&mut caller);
        assert_eq!(caller, [1u8; 20]);

        ext::set_storage(StorageFlags::empty(), b"key", b"value");
        off_chain::with_engine(|e| {
            let v = e.get_storage_value(b"key").unwrap();
            assert_eq!(v, b"value");
        });
    }
}
