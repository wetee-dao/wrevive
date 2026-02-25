// test / feature "off_chain" / feature "std" => 使用 std；否则 no_std。测试时依赖方可用 feature "std" 在 host 上跑。
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
    #[cfg(all(not(test), not(feature = "off_chain")))]
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
    #[cfg(all(not(test), not(feature = "off_chain")))]
    let data = on_chain::ON_CHAIN_ENV.get_storage_bytes(flags, &key_bytes)?;
    V::decode(&mut &data[..]).map_err(|_| ReturnErrorCode::KeyNotFound)
}

/// test 时使用 off_chain，正常运行。
#[cfg(test)]
mod tests;
