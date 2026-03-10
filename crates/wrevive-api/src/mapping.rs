//! # Mapping
//!
//! ink!-style Mapping<K, V>: set/get by prefix namespace + key; backend via Env.
//! K must Encode, V must Encode+Decode.
//! ink! 风格的 Mapping<K, V>：按前缀命名空间 + key 的 set/get 封装，底层通过传入的 Env 调用。
//! K 须 Encode，V 须 Encode+Decode。

#[cfg(not(any(test, feature = "off_chain")))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "off_chain"))]
use std::vec::Vec;

#[cfg(not(any(test, feature = "off_chain")))]
extern crate alloc;

use core::marker::PhantomData;
use pallet_revive_uapi::{ReturnErrorCode, StorageFlags};
use crate::{env, Env};

    /// Maximum size in bytes of buffer allocated for reading storage values.
    /// 为读取存储值而分配的缓冲区的最大字节大小。
    ///
    /// Values serialized larger than this size will be truncated, causing decode failures.
    /// value 序列化后超过此长度将截断并解码失败。
pub const GET_BUF_SIZE: usize = 512;

    /// Errors that can occur when interacting with Mapping storage.
    /// 与 Mapping 存储交互时可能发生的错误。
#[derive(Debug)]
pub enum MappingError {
    KeyNotFound(ReturnErrorCode),
    Decode(parity_scale_codec::Error),
}

impl From<ReturnErrorCode> for MappingError {
    fn from(e: ReturnErrorCode) -> Self {
        MappingError::KeyNotFound(e)
    }
}

impl From<parity_scale_codec::Error> for MappingError {
    fn from(e: parity_scale_codec::Error) -> Self {
        MappingError::Decode(e)
    }
}

    /// A key-value storage mapping where storage keys are formed by concatenating
    /// a fixed prefix with the actual key.
    /// 命名空间下的键值映射：storage key = `prefix` || `key`。
    ///
    /// Values are SCALE-encoded bytes. Generic types K and V are specified at initialization.
    /// 值为 SCALE 编码字节。泛型类型 K 和 V 在初始化时指定。
#[derive(Clone, Copy)]
pub struct Mapping<K, V> {
    prefix: &'static [u8],
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> Mapping<K, V> {
    /// Creates a new Mapping instance using the given prefix as a namespace.
    /// 创建以 `prefix` 为命名空间的 Mapping 实例。
    ///
    /// All storage keys under this mapping will be prefixed with these bytes.
    /// 此映射下的所有存储键都将加上此前缀。
    /// Key/value types are specified by generics K/V.
    /// key/value 类型由泛型 K/V 指定。
    pub const fn new(prefix: &'static [u8]) -> Self {
        Self {
            prefix,
            _phantom: PhantomData,
        }
    }

    /// Returns the prefix bytes used as namespace for this mapping.
    /// 返回用为此映射命名空间的前缀字节。
    pub const fn prefix(&self) -> &'static [u8] {
        self.prefix
    }

    /// Concatenates prefix and key into the provided buffer.
    /// 将前缀和键连接到提供的缓冲区中。
    ///
    /// Returns the combined bytes slice if buffer is large enough, otherwise None.
    /// 如果缓冲区足够大，返回组合的字节切片，否则返回 None。
    #[inline]
    pub fn full_key<'a>(&self, key: &[u8], buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let n = self.prefix.len().saturating_add(key.len());
        if buf.len() < n {
            return None;
        }
        buf[..self.prefix.len()].copy_from_slice(self.prefix);
        buf[self.prefix.len()..n].copy_from_slice(key);
        Some(&buf[..n])
    }

    /// Stores a raw byte value at the given key.
    /// 按字节写入：`mapping[key] = value`。
    ///
    /// Caller must provide a buffer large enough to hold the full storage key (at least `prefix.len() + key.len()`).
    /// 调用方必须提供足够大的缓冲区来容纳完整的存储键（至少 `prefix.len() + key.len()`）。
    ///
    /// This is intended for use by Env implementations.
    /// 此方法供 Env 实现使用。
    #[inline]
    pub fn set_bytes(
        &self,
        key: &[u8],
        buf: &mut [u8],
        value: &[u8],
    ) -> Option<()> {
        let full = self.full_key(key, buf)?;
        let v = value.to_vec();
        env().set_storage(StorageFlags::empty(), full, &v);
        Some(())
    }

    /// Retrieves raw byte value at the given key.
    /// 按字节读取：返回 `mapping[key]` 的原始字节。
    ///
    /// Caller must provide a buffer large enough to hold the full storage key.
    /// 调用方必须提供足够大的缓冲区来容纳完整的存储键。
    ///
    /// Returns None if key does not exist.
    /// 如果键不存在，返回 None。
    #[inline]
    pub fn get_bytes(
        &self,
        key: &[u8],
        buf: &mut [u8],
    ) -> Option<Vec<u8>> {
        let full = self.full_key(key, buf)?;
        env().get_storage(StorageFlags::empty(), full)
    }
}

impl<K, V> Mapping<K, V>
where
    K: parity_scale_codec::Encode,
    V: parity_scale_codec::Encode + parity_scale_codec::Decode,
{
    /// Writes: `mapping[key] = value`. K/V use SCALE encoding.
    /// 写入：`mapping[key] = value`。K/V 使用 SCALE 编码。
    /// 
    /// # English
    /// Stores a value at the given key using SCALE encoding for both key and value.
    /// 
    /// # 中文
    /// 使用 SCALE 编码（对键和值）在给定键处存储值。
    #[inline]
    pub fn set(&self, key: &K, value: &V) -> Option<()> {
        let key_bytes = key.encode();

        #[cfg(not(any(test, feature = "off_chain")))]
        let mut buf = alloc::vec![0u8; self.prefix.len() + key_bytes.len()];
        #[cfg(any(test, feature = "off_chain"))]
        let mut buf = vec![0u8; self.prefix.len() + key_bytes.len()];

        let full = self.full_key(&key_bytes, &mut buf)?;
        env().set_storage(StorageFlags::empty(), full, value);
        Some(())
    }

    /// Retrieves and decodes the value at the given key.
    /// 读取：将 `mapping[key]` 解码为 `V`。
    ///
    /// Returns None if the key does not exist or if the value cannot be decoded
    /// (e.g., struct format mismatch, data corruption).
    /// 如果键不存在或值无法解码则返回 None（如结构体格式不匹配、数据损坏）。
    #[inline]
    pub fn get(&self, key: &K) -> Option<V> {
        let key_bytes = key.encode();

        #[cfg(not(any(test, feature = "off_chain")))]
        let mut key_buf = alloc::vec![0u8; self.prefix.len() + key_bytes.len()];
        #[cfg(any(test, feature = "off_chain"))]
        let mut key_buf = vec![0u8; self.prefix.len() + key_bytes.len()];

        let full = self.full_key(&key_bytes, &mut key_buf)?;
        env().get_storage(StorageFlags::empty(), full)
    }

    /// Removes the stored value at the given key.
    /// 清除：删除 `mapping[key]` 的存储。
    ///
    /// Implemented via Env::clear_storage; on-chain uses set_storage_or_clear when key is 32 bytes.
    /// 通过 Env::clear_storage 实现，链上在 key 为 32 字节时使用 set_storage_or_clear。
    #[inline]
    pub fn clear(&self, key: &K) -> Option<()> {
        let key_bytes = key.encode();

        #[cfg(not(any(test, feature = "off_chain")))]
        let mut buf = alloc::vec![0u8; self.prefix.len() + key_bytes.len()];
        #[cfg(any(test, feature = "off_chain"))]
        let mut buf = vec![0u8; self.prefix.len() + key_bytes.len()];

        let full = self.full_key(&key_bytes, &mut buf)?;
        env().clear_storage(StorageFlags::empty(), full);
        Some(())
    }
}
