//! # wrevive-api
//!
//! PolkaVM (pallet-revive) 合约运行时 API：链上/链下统一抽象、存储与列表封装。
//! PolkaVM (pallet-revive) contract runtime API: unified on-chain/off-chain env, Storage/Mapping/List/List2D.

// test / feature "off_chain" => 使用 std；否则 no_std。
// 测试时启用 off_chain 在 host 上跑。
#![cfg_attr(not(any(test, feature = "off_chain")), no_std)]
#[cfg(not(any(test, feature = "off_chain")))]
extern crate alloc;

use scale_info::prelude::marker::PhantomData;

/// Environment trait: unified interface for on-chain and off-chain.
/// Environment 抽象：on_chain / off_chain 统一接口。
///
/// # English
/// Provides a unified abstraction for contract execution environment,
/// working seamlessly on both on-chain and off-chain (testing) contexts.
///
/// # 中文
/// 为合约执行环境提供统一抽象，
/// 在链上和链下（测试）上下文中无缝工作。
pub mod env;
pub use env::{CallMode, Env, MAX_STORAGE_VALUE_SIZE};

/// List / 2D List: sequential and two-dimensional list storage.
/// List / 2D List：顺序列表与二维列表存储。
///
/// # English
/// Implements list-like storage abstractions with auto-incrementing indices.
/// List provides one-dimensional sequential storage.
/// List2D provides two-dimensional grouped storage.
///
/// # 中文
/// 实现具有自增索引的列表式存储抽象。
/// List 提供一维顺序存储。
/// List2D 提供二维分组存储。
pub mod list;
pub mod list_2d;
/// Mapping: set/get encapsulation with prefix namespace + key, similar to ink Mapping.
/// Mapping：按前缀命名空间 + key 的 set/get 封装，类似 ink Mapping。
///
/// # English
/// Provides key-value storage with namespace prefixing.
/// Storage keys are formed by concatenating prefix with actual key.
///
/// # 中文
/// 提供带命名空间前缀的键值存储。
/// 存储键通过连接前缀和实际键形成。
pub mod mapping;

/// Common data types: Address, H256, U256, BlockNumber, Bytes.
/// 常见数据类型：Address、H256、U256、BlockNumber、Bytes。
///
/// # English
/// Defines frequently used types for smart contracts.
/// Includes address, hash, unsigned integers, and bytes types.
///
/// # 中文
/// 定义智能合约常用的类型。
/// 包括地址、哈希、无符号整数和字节类型。
pub mod types;
pub use types::*;

#[cfg(all(not(test), not(feature = "off_chain")))]
pub use alloc::vec::Vec;
/// Vec 再导出：test 或 off_chain 用 std；否则 no_std 合约用 alloc。
#[cfg(any(test, feature = "off_chain"))]
pub use std::vec::Vec;

/// Re-export from pallet_revive_uapi for contract code (input!, HostFn, flags).
/// 从 pallet_revive_uapi 再导出，供合约使用（input!、HostFn、flags）。
///
/// # English
/// Re-exports essential types from the PolkaVM user API.
/// These include host function definitions, error codes, and flags.
///
/// # 中文
/// 从 PolkaVM 用户 API 重新导出基本类型。
/// 包括主机函数定义、错误代码和标志。
pub use pallet_revive_uapi::{HostFn, ReturnErrorCode, ReturnFlags, StorageFlags, input};

pub use list::{List, ListIndex};
pub use list_2d::List2D;
pub use mapping::{Mapping, MappingError};
/// SCALE encoding/decoding and type info for `#[scale_derive(Encode, Decode, TypeInfo)]` and Mapping set/get.
/// Scale 编解码与类型信息，供 `#[scale_derive(Encode, Decode, TypeInfo)]` 及 Mapping set/get 使用。
///
/// # English
/// Re-exports SCALE codec traits for serialization.
/// Used by derive macros and storage operations.
///
/// # 中文
/// 重新导出用于序列化的 SCALE 编解码 trait。
/// 由派生宏和存储操作使用。
pub use parity_scale_codec::{Decode, Encode};
pub use scale_info::TypeInfo;
pub use traits::Storable;

// 正常运行暴露 on_chain：cfg(not(test)) 且未启用 off_chain（与 off_chain 互斥）
/// On-chain environment implementation.
#[cfg(all(not(test), not(feature = "off_chain")))]
pub mod on_chain;

#[cfg(all(not(test), not(feature = "off_chain")))]
pub mod buffer;

pub mod traits;

pub const BUFFER_SIZE: usize = 16384;

// test 暴露 off_chain：cfg(test) 或 feature "off_chain"（依赖方 cargo test 时启用 off_chain）
/// Off-chain environment implementation for testing.
#[cfg(any(test, feature = "off_chain"))]
pub mod off_chain;

#[cfg(any(test, feature = "off_chain"))]
pub use off_chain::{Engine, ReturnValuePanic, with_engine};

/// Returns the current backend Env.
/// 返回当前后端 Env。
///
/// Contracts use `env().caller()`, `env().set_storage()` etc.
/// 合约通过 `env().caller()`、`env().set_storage()` 等调用。
///
/// Returns concrete type (not dyn) so the trait can retain generic methods set_storage<V>/get_storage<V>.
/// 返回具体类型（非 dyn），以便 trait 可保留泛型方法 set_storage<V>/get_storage<V>。
#[cfg(any(test, feature = "off_chain"))]
#[inline(always)]
pub fn env() -> &'static mut off_chain::OffChainEnv {
    unsafe { &mut *(&raw mut off_chain::OFF_CHAIN_ENV) }
}

/// Read storage at key and decode as V (for test/off_chain).
/// 测试/off_chain 下按 key 读取存储并解码为 V。
///
/// Returns None if key missing or decode fails.
/// 不存在或解码失败返回 None。
#[cfg(any(test, feature = "off_chain"))]
pub fn get_storage<V: Storable>(flags: StorageFlags, key: &[u8]) -> Option<V> {
    env().get_storage(flags, key)
}

#[cfg(all(not(test), not(feature = "off_chain")))]
#[inline(always)]
pub fn env() -> &'static mut on_chain::OnChainEnv {
    unsafe { &mut *(&raw mut on_chain::ON_CHAIN_ENV) }
}

/// Single-key storage: storage key = prefix, value is SCALE-encoded V.
/// 单 key 存储：storage key = prefix，value 为 SCALE 编码的 `V`。
#[derive(Clone, Copy)]
pub struct Storage<V>(&'static [u8], PhantomData<V>);

impl<V> Storage<V>
where
    V: Encode + Decode,
{
    /// Creates storage with the given prefix (used as the full storage key).
    /// 使用给定 prefix 创建存储（prefix 即完整 storage key）。
    pub const fn new(prefix: &'static [u8]) -> Self {
        Self(prefix, PhantomData)
    }

    /// Writes value; returns previous value length in bytes if any.
    /// 写入 value；若有旧值则返回其字节长度。
    ///
    /// Encoding is done in env.
    /// 编码在 env 模块完成。
    pub fn set(&self, value: &V) -> Option<u32> {
        env().set_storage(StorageFlags::empty(), self.0, value)
    }

    /// Reads and decodes value.
    /// 读取并解码。
    ///
    /// Returns None if missing or decode fails.
    /// 不存在或解码失败时返回 None。
    ///
    /// Decoding is done in env.
    /// 解码在 env 模块完成。
    pub fn get(&self) -> Option<V> {
        env().get_storage(StorageFlags::empty(), self.0)
    }

    /// Clears value at this key; returns previous value length in bytes if any.
    /// 清除该 key 的值；若有旧值则返回其字节长度。
    pub fn clear(&self) -> Option<u32> {
        env().clear_storage(StorageFlags::empty(), self.0)
    }
}

/// Unit tests use off-chain backend.
/// 单元测试使用 off-chain 后端。
#[cfg(test)]
mod tests;
