//! # wrevive-api
//!
//! PolkaVM (pallet-revive) 合约运行时 API：链上/链下统一抽象、存储与列表封装。
//! PolkaVM (pallet-revive) contract runtime API: unified on-chain/off-chain env, Storage/Mapping/List/List2D.

// test / feature "off_chain" => 使用 std；否则 no_std。测试时启用 off_chain 在 host 上跑。
#![cfg_attr(not(any(test, feature = "off_chain")), no_std)]
#[cfg(not(any(test, feature = "off_chain")))]
extern crate alloc;

use scale_info::prelude::marker::PhantomData;

/// Environment trait: unified interface for on_chain and off_chain.
/// Environment 抽象：on_chain / off_chain 统一接口。
pub mod env;
pub use env::{Env, MAX_STORAGE_VALUE_SIZE};

/// Mapping：按前缀命名空间 + key 的 set/get 封装，类似 ink Mapping。
pub mod mapping;
/// List / 2D List：顺序列表与二维列表存储（参考 primitives define_map / define_double_map_base）。
pub mod list;
pub mod list_2d;

/// 常见数据类型：Address、H256、U256、BlockNumber、String。
pub mod types;
pub use types::*;

/// Vec 再导出：test 或 off_chain 用 std；否则 no_std 合约用 alloc。
#[cfg(any(test, feature = "off_chain"))]
pub use std::vec::Vec;
#[cfg(all(not(test), not(feature = "off_chain")))]
pub use alloc::vec::Vec;

/// Re-export from pallet_revive_uapi for contract code (input!, HostFn, flags).
/// 从 pallet_revive_uapi 再导出，供合约使用（input!、HostFn、flags）。
pub use pallet_revive_uapi::{input, HostFn, ReturnErrorCode, ReturnFlags, StorageFlags};

/// Scale 编解码与类型信息，供 `#[scale_derive(Encode, Decode, TypeInfo)]` 及 Mapping set/get 使用。
pub use parity_scale_codec::{Decode, Encode};
pub use scale_info::TypeInfo;
pub use mapping::{Mapping, MappingError};
pub use list::{List, ListIndex};
pub use list_2d::List2D;
pub use traits::Storable;


// 正常运行暴露 on_chain：cfg(not(test)) 且未启用 off_chain（与 off_chain 互斥）
#[cfg(all(not(test), not(feature = "off_chain")))]
pub mod on_chain;

#[cfg(all(not(test), not(feature = "off_chain")))]
pub mod buffer;

pub mod traits;

pub const BUFFER_SIZE: usize = 16384;

// test 暴露 off_chain：cfg(test) 或 feature "off_chain"（依赖方 cargo test 时启用 off_chain）
#[cfg(any(test, feature = "off_chain"))]
pub mod off_chain;

#[cfg(any(test, feature = "off_chain"))]
pub use off_chain::{with_engine, Engine, ReturnValuePanic};

/// Returns the current backend Env. Contracts use `env().caller()`, `env().set_storage()` etc.
/// 返回当前后端 Env；合约通过 `env().caller()`、`env().set_storage()` 等调用。
/// 返回具体类型（非 dyn），以便 trait 可保留泛型方法 set_storage<V>/get_storage<V>。
#[cfg(any(test, feature = "off_chain"))]
#[inline(always)]
pub fn env() -> &'static mut off_chain::OffChainEnv {
    unsafe { &mut *(&raw mut off_chain::OFF_CHAIN_ENV) }
}

/// Read storage at key and decode as V (for test/off_chain). Returns None if key missing or decode fails.
/// 测试/off_chain 下按 key 读取存储并解码为 V；不存在或解码失败返回 None。
#[cfg(any(test, feature = "off_chain"))]
pub fn get_storage<V: Storable>(flags: StorageFlags, key: &[u8]) -> Option<V> {
    crate::env::get_storage(env(), flags, key)
}

#[cfg(all(not(test), not(feature = "off_chain")))]
#[inline(always)]
pub fn env() -> &'static mut on_chain::OnChainEnv {
    unsafe { &mut *(&raw mut on_chain::ON_CHAIN_ENV) }
}

/// Single-key storage: storage key = prefix, value is Scale-encoded V.
/// 单 key 存储：storage key = prefix，value 为 Scale 编码的 `V`。
#[derive(Clone, Copy)]
pub struct Storage<V>(&'static [u8], PhantomData<V>);

impl<V> Storage<V>
where
    V: Encode + Decode,
{
    /// Create storage with the given prefix (used as the full storage key).
    /// 使用给定 prefix 创建存储（prefix 即完整 storage key）。
    pub const fn new(prefix: &'static [u8]) -> Self {
        Self(prefix, PhantomData)
    }

    /// Write value; returns previous value length in bytes if any. Encode is done in env.
    /// 写入 value；若有旧值则返回其字节长度；编码在 env 模块完成。
    pub fn set(&self, api: &mut impl crate::env::Env, value: &V) -> Option<u32> {
        crate::env::set_storage(api, StorageFlags::empty(), self.0, value)
    }

    /// Read and decode value; None if missing or decode fails. Decode is done in env.
    /// 读取并解码；不存在或解码失败时返回 None；解码在 env 模块完成。
    pub fn get(&self, api: &mut impl crate::env::Env) -> Option<V> {
        crate::env::get_storage(api, StorageFlags::empty(), self.0)
    }

    /// Clear value at this key; returns previous value length in bytes if any.
    /// 清除该 key 的值；若有旧值则返回其字节长度。
    pub fn clear(&self, api: &mut impl crate::env::Env) -> Option<u32> {
        api.clear_storage(StorageFlags::empty(), self.0)
    }
}

/// Unit tests use off_chain backend.
/// 单元测试使用 off_chain 后端。
#[cfg(test)]
mod tests;
