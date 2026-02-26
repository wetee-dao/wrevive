// test / feature "off_chain" / feature "std" => 使用 std；否则 no_std。测试时依赖方可用 feature "std" 在 host 上跑。
#![cfg_attr(not(any(test, feature = "off_chain")), no_std)]
#[cfg(not(any(test, feature = "off_chain")))]
extern crate alloc;

use scale_info::prelude::marker::PhantomData;

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


// 正常运行暴露 on_chain：cfg(not(test)) 且未启用 off_chain（与 off_chain 互斥）
#[cfg(all(not(test), not(feature = "off_chain")))]
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
#[cfg(all(not(test), not(feature = "off_chain")))]
#[inline(always)]
pub fn env() -> &'static dyn Env {
    &on_chain::ON_CHAIN_ENV
}

/// 单 key 存储：storage key = prefix，value 为 Scale 编码的 `V`。
pub struct Storage<V>(&'static [u8], PhantomData<V>);

impl<V> Storage<V>
where
    V: Encode + Decode,
{
    pub const fn new(prefix: &'static [u8]) -> Self {
        Self(prefix, PhantomData)
    }

    pub fn set(&self, api: &dyn crate::env::Env, value: &V) -> Option<u32> {
        let value_bytes = value.encode();
        api.set_storage_bytes(StorageFlags::empty(), self.0, &value_bytes)
    }

    pub fn get(&self, api: &dyn crate::env::Env) -> Result<V, ReturnErrorCode> {
        let data = api.get_storage_bytes(StorageFlags::empty(), self.0)?;
        V::decode(&mut &data[..]).map_err(|_| ReturnErrorCode::KeyNotFound)
    }
}

/// test 时使用 off_chain，正常运行。
#[cfg(test)]
mod tests;
