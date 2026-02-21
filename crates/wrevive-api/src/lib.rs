//! # wrevive-api
//!
//! Thin wrapper over `pallet_revive_uapi` with **on_chain** and **off_chain** backends,
//! similar to the [ink! engine](https://github.com/use-ink/ink/tree/master/crates/engine).
//!
//! pallet_revive_uapi 封装：**on_chain** / **off_chain** 两套接口，参考 [ink engine](https://github.com/use-ink/ink/tree/master/crates/engine)。
//!
//! ## Backends / 后端
//!
//! - **on_chain** (default): `ext` delegates to `HostFnImpl`; PolkaVM/RISC-V only; for deployed contracts.
//!   **on_chain**（默认）：`ext` 委托 `HostFnImpl`，仅 PolkaVM/RISC-V，用于链上部署。
//!
//! - **off_chain** (feature `off_chain`): `ext` uses an in-memory `Engine` with test helpers
//!   (`set_caller`, `set_call_data`, `get_storage_value`, `take_events`, etc.) for unit tests.
//!   **off_chain**（feature `off_chain`）：`ext` 使用内存 `Engine`，提供 test_api（set_caller、set_call_data、get_storage_value、take_events 等），用于单元测试。

#![cfg_attr(not(feature = "off_chain"), no_std)]

/// Chain API trait: unified interface for on_chain and off_chain.
/// 链 API 抽象：on_chain / off_chain 统一接口。
pub mod chain_api;

/// Re-export from pallet_revive_uapi for contract code (input!, HostFn, flags).
/// 从 pallet_revive_uapi 再导出，供合约使用（input!、HostFn、flags）。
pub use pallet_revive_uapi::{input, HostFn, ReturnFlags, StorageFlags};

// On-chain backend: only when not off_chain and target is riscv64 (PolkaVM contract).
// 链上后端：仅在未启用 off_chain 且目标为 riscv64（PolkaVM 合约）时编译。
#[cfg(all(not(feature = "off_chain"), target_arch = "riscv64"))]
pub mod on_chain;

#[cfg(all(not(feature = "off_chain"), target_arch = "riscv64"))]
pub use on_chain::ext;

// Off-chain backend: in-memory Engine for tests.
// 链下后端：内存 Engine，用于测试。
#[cfg(feature = "off_chain")]
pub mod off_chain;

#[cfg(feature = "off_chain")]
pub use off_chain::{with_engine, Engine};
#[cfg(feature = "off_chain")]
pub use off_chain::ext;

/// Off-chain engine tests: set_caller, set_call_data, get_storage_value.
/// 链下引擎测试：set_caller、set_call_data、get_storage_value。
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
