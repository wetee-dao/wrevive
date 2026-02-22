//! # wrevive-api
//!
//! Thin wrapper over `pallet_revive_uapi` with **on_chain** and **off_chain** backends,
//! similar to the [ink! engine](https://github.com/use-ink/ink/tree/master/crates/engine).
//!
//! pallet_revive_uapi 封装：**on_chain** / **off_chain** 两套接口，参考 [ink engine](https://github.com/use-ink/ink/tree/master/crates/engine)。
//!
//! ## Backends / 后端
//!
//! - **test 暴露 off_chain**：`cargo test` 时 `cfg(test)` 为真，编译并导出 `off_chain`（内存 Engine），测试可正常运行。
//! - **正常运行暴露 on_chain**：非 test 构建时 `cfg(not(test))`，编译并导出 `on_chain`（HostFnImpl），仅 riscv64/PolkaVM。
//! - 依赖方若需在自身 test 中使用 off_chain，可启用 feature `off_chain`。

// test 或 feature "off_chain" => 有 off_chain（off_chain）；否则 no_off_chain（on_chain）。依赖方用 feature "off_chain" 获得 off_chain。
#![cfg_attr(not(any(test, feature = "off_chain")), no_std)]

/// Chain API trait: unified interface for on_chain and off_chain.
/// 链 API 抽象：on_chain / off_chain 统一接口。
pub mod chain_api;

/// Re-export from pallet_revive_uapi for contract code (input!, HostFn, flags).
/// 从 pallet_revive_uapi 再导出，供合约使用（input!、HostFn、flags）。
pub use pallet_revive_uapi::{input, HostFn, ReturnFlags, StorageFlags};

// 正常运行暴露 on_chain：cfg(not(test)) 且未启用 off_chain，目标 riscv64（与 off_chain 互斥）
#[cfg(all(not(test), not(feature = "off_chain"), target_arch = "riscv64"))]
pub mod on_chain;

#[cfg(all(not(test), not(feature = "off_chain"), target_arch = "riscv64"))]
pub use on_chain::ext;

// test 暴露 off_chain：cfg(test) 或 feature "off_chain"（依赖方 cargo test 时启用 off_chain）
#[cfg(any(test, feature = "off_chain"))]
pub mod off_chain;

#[cfg(any(test, feature = "off_chain"))]
pub use off_chain::{with_engine, Engine, ext};

/// test 时使用 off_chain，正常运行。
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
