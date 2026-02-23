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
//!
//! ## no_std 与 Mapping（无需调用方 buffer）
//!
//! 链上 no_std 时，`Mapping::set` / `Mapping::get` 内部分别用 `value.encode()` 与 `vec![0u8; 256]`，
//! 不再要求调用方传入 buffer。这依赖 **全局分配器**：需在合约根（如 `lib.rs`）中设置 `#[global_allocator]`，
//! 否则 `alloc::vec::Vec` 无法工作。例如使用 [picoalloc](https://crates.io/crates/picoalloc)（通过 `wrevive-macro`）：
//!
//! ```ignore
//! use wrevive_macro::picoalloc_global_allocator;
//! picoalloc_global_allocator!(1024); // 1024 字节堆，可按需调大
//! ```

// test 或 feature "off_chain" => 有 off_chain（off_chain）；否则 no_off_chain（on_chain）。依赖方用 feature "off_chain" 获得 off_chain。
#![cfg_attr(not(any(test, feature = "off_chain")), no_std)]
#[cfg(not(any(test, feature = "off_chain")))]
extern crate alloc;

/// Environment trait: unified interface for on_chain and off_chain.
/// Environment 抽象：on_chain / off_chain 统一接口。
pub mod env;
pub use env::Env;

/// Mapping：按前缀命名空间 + key 的 set/get 封装，类似 ink Mapping。
pub mod mapping;

/// Re-export from pallet_revive_uapi for contract code (input!, HostFn, flags).
/// 从 pallet_revive_uapi 再导出，供合约使用（input!、HostFn、flags）。
pub use pallet_revive_uapi::{input, HostFn, ReturnErrorCode, ReturnFlags, StorageFlags};

/// Scale 编解码与类型信息，供 `#[scale_derive(Encode, Decode, TypeInfo)]` 及 Mapping set/get 使用。
pub use parity_scale_codec::{Decode, Encode};
pub use scale_info::TypeInfo;
pub use mapping::{Mapping, MappingError};


// 正常运行暴露 on_chain：cfg(not(test)) 且未启用 off_chain，目标 riscv64（与 off_chain 互斥）
#[cfg(all(not(test), not(feature = "off_chain"), target_arch = "riscv64"))]
pub mod on_chain;

// test 暴露 off_chain：cfg(test) 或 feature "off_chain"（依赖方 cargo test 时启用 off_chain）
#[cfg(any(test, feature = "off_chain"))]
pub mod off_chain;

#[cfg(any(test, feature = "off_chain"))]
pub use off_chain::{with_engine, Engine};

/// 当前 backend 的 Env，合约通过 `env().caller()`、`env().set_storage()` 等调用。
#[cfg(any(test, feature = "off_chain"))]
#[inline(always)]
pub fn env() -> &'static dyn Env {
    &off_chain::OFF_CHAIN_ENV
}
#[cfg(all(not(test), not(feature = "off_chain"), target_arch = "riscv64"))]
#[inline(always)]
pub fn env() -> &'static dyn Env {
    &on_chain::ON_CHAIN_ENV
}

/// 便利函数：使用 Scale 编码的 key 和 value 进行存储。
/// 支持任意实现 `Encode` 的 key 和 value 类型。
#[inline]
pub fn set_storage<K: Encode + ?Sized, V: Encode + ?Sized>(
    flags: StorageFlags,
    key: &K,
    value: &V,
) -> Option<u32> {
    let key_bytes = key.encode();
    let value_bytes = value.encode();
    #[cfg(any(test, feature = "off_chain"))]
    {
        off_chain::OFF_CHAIN_ENV.set_storage_bytes(flags, &key_bytes, &value_bytes)
    }
    #[cfg(all(not(test), not(feature = "off_chain"), target_arch = "riscv64"))]
    {
        on_chain::ON_CHAIN_ENV.set_storage_bytes(flags, &key_bytes, &value_bytes)
    }
}

/// 便利函数：使用 Scale 编码的 key 读取存储值。
/// 支持任意实现 `Encode` 的 key 类型和 `Decode` 的 value 类型。
#[inline]
pub fn get_storage<K: Encode + ?Sized, V: Decode>(
    flags: StorageFlags,
    key: &K,
) -> Result<V, ReturnErrorCode> {
    let key_bytes = key.encode();
    #[cfg(any(test, feature = "off_chain"))]
    let data = off_chain::OFF_CHAIN_ENV.get_storage_bytes(flags, &key_bytes)?;
    #[cfg(all(not(test), not(feature = "off_chain"), target_arch = "riscv64"))]
    let data = on_chain::ON_CHAIN_ENV.get_storage_bytes(flags, &key_bytes)?;
    V::decode(&mut &data[..]).map_err(|_| ReturnErrorCode::KeyNotFound)
}

/// 链上全局分配器：包装 picoalloc，使 static 满足 Sync（合约单线程执行）。
/// 类型定义保留在此处，供 `wrevive_macro::picoalloc_global_allocator!` 宏使用。
#[cfg(feature = "on_chain")]
mod picoalloc_allocator {
    pub use picoalloc;

    pub struct PicoallocWrapper<const N: usize>(pub picoalloc::Mutex<picoalloc::Allocator<picoalloc::ArrayPointer<N>>>);

    unsafe impl<const N: usize> Send for PicoallocWrapper<N> {}
    unsafe impl<const N: usize> Sync for PicoallocWrapper<N> {}

    unsafe impl<const N: usize> core::alloc::GlobalAlloc for PicoallocWrapper<N> {
        unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
            let align = picoalloc::Size::from_bytes_usize(layout.align().max(1)).unwrap();
            let size = picoalloc::Size::from_bytes_usize(layout.size().max(1)).unwrap();
            self.0.lock().alloc(align, size).map(|p| p.as_ptr()).unwrap_or(core::ptr::null_mut())
        }
        unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
            if !ptr.is_null() {
                self.0.lock().free(core::ptr::NonNull::new_unchecked(ptr));
            }
        }
    }
}
#[cfg(feature = "on_chain")]
pub use picoalloc_allocator::{picoalloc, PicoallocWrapper};

/// test 时使用 off_chain，正常运行。
#[cfg(test)]
mod tests;
