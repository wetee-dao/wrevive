//! # Mapping
//!
//! ink!-style Mapping<K, V>: set/get by prefix namespace + key; backend via Env. K must Encode, V must Encode+Decode.
//! 按前缀命名空间 + key 的 set/get 封装，底层通过传入的 Env 调用；K 须 Encode，V 须 Encode+Decode。

#[cfg(not(any(test, feature = "off_chain")))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "off_chain"))]
use std::vec::Vec;

#[cfg(not(any(test, feature = "off_chain")))]
extern crate alloc;

use core::marker::PhantomData;
use pallet_revive_uapi::{ReturnErrorCode, StorageFlags};
use crate::{env, Env};

    /// Internal buffer size (bytes) for `get`; values serialized longer than this will be truncated and fail to decode.
    /// `get` 内部分配的缓冲区大小（字节）；value 序列化后超过此长度将截断并解码失败。
    /// 
    /// # English
    /// Maximum size in bytes of buffer allocated for reading storage values.
    /// Values serialized larger than this size will be truncated,
    /// causing decode failures.
    /// 
    /// # 中文
    /// 为读取存储值而分配的缓冲区的最大字节大小。
    /// 序列化后超过此大小的值将被截断，
    /// 导致解码失败。
pub const GET_BUF_SIZE: usize = 512;

    /// Possible errors from set/get: key not found or value decode failure.
    /// set/get 可能产生的错误：key 不存在或 value 解码失败。
    /// 
    /// # English
    /// Errors that can occur when interacting with Mapping storage.
    /// 
    /// # 中文
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

    /// Key-value mapping under a namespace: storage key = `prefix` || `key`, value is SCALE-encoded bytes.
    /// 命名空间下的键值映射：storage key = `prefix` || `key`，value 为 Scale 编码字节。
    /// 
    /// # English
    /// A key-value storage mapping where storage keys are formed by concatenating
    /// a fixed prefix with the actual key. Values are SCALE-encoded bytes.
    /// Generic types K and V are specified at initialization.
    /// 
    /// # 中文
    /// 键值存储映射，其中存储键通过连接固定前缀和实际键形成。
    /// 值为 SCALE 编码字节。泛型类型 K 和 V 在初始化时指定。
#[derive(Clone, Copy)]
pub struct Mapping<K, V> {
    prefix: &'static [u8],
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> Mapping<K, V> {
    /// Creates a Mapping with `prefix` as namespace; key/value types specified by generics K/V.
    /// 创建以 `prefix` 为命名空间的 Mapping，key/value 类型由泛型 K/V 指定。
    /// 
    /// # English
    /// Creates a new Mapping instance using the given prefix as a namespace.
    /// All storage keys under this mapping will be prefixed with these bytes.
    /// 
    /// # 中文
    /// 使用给定前缀作为命名空间创建新的 Mapping 实例。
    /// 此映射下的所有存储键都将加上此前缀。
    pub const fn new(prefix: &'static [u8]) -> Self {
        Self {
            prefix,
            _phantom: PhantomData,
        }
    }

    /// Returns the prefix.
    /// 返回 prefix。
    /// 
    /// # English
    /// Gets the prefix bytes used as namespace for this mapping.
    /// 
    /// # 中文
    /// 获取用为此映射作命名空间的前缀字节。
    pub const fn prefix(&self) -> &'static [u8] {
        self.prefix
    }

    /// Builds full storage key = prefix || key in buf; returns slice of length prefix.len()+key.len().
    /// 将完整 key 写入 buf，返回 prefix||key 的切片；buf 不足时返回 None。
    /// 
    /// # English
    /// Concatenates prefix and key into the provided buffer.
    /// Returns the combined bytes slice if buffer is large enough, otherwise None.
    /// 
    /// # 中文
    /// 将前缀和键连接到提供的缓冲区中。
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

    /// Writes by bytes: `mapping[key] = value`, using caller-provided `buf` (length at least `prefix.len() + key.len()`). For Env use.
    /// 按字节写入：`mapping[key] = value`，使用调用方提供的 `buf`（长度至少 `prefix.len() + key.len()`）。供 Env 使用。
    /// 
    /// # English
    /// Stores a raw byte value at the given key.
    /// Caller must provide a buffer large enough to hold the full storage key.
    /// This is intended for use by Env implementations.
    /// 
    /// # 中文
    /// 在给定键处存储原始字节值。
    /// 调用方必须提供足够大的缓冲区来容纳完整的存储键。
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

    /// Reads by bytes: returns raw bytes of `mapping[key]`; uses caller-provided `buf`. Returns `None` if key does not exist.
    /// 按字节读取：返回 `mapping[key]` 的原始字节；使用调用方提供的 `buf`。key 不存在返回 `None`。
    /// 
    /// # English
    /// Retrieves raw byte value at the given key.
    /// Caller must provide a buffer large enough to hold the full storage key.
    /// Returns None if key does not exist.
    /// 
    /// # 中文
    /// 检索给定键处的原始字节值。
    /// 调用方必须提供足够大的缓冲区来容纳完整的存储键。
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
    /// Writes: `mapping[key] = value`. K / V use SCALE encoding.
    /// 写入：`mapping[key] = value`。K / V 使用 Scale 编码。
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

    /// Reads: decodes `mapping[key]` as `V`. Returns `None` if key does not exist or value decode fails (e.g., struct format mismatch, data corruption).
    /// 读取：将 `mapping[key]` 解码为 `V`。key 不存在或 value 解码失败（如结构体格式不匹配、数据损坏）时返回 `None`。
    /// 
    /// # English
    /// Retrieves and decodes the value at the given key.
    /// Returns None if the key does not exist or if the value cannot be decoded,
    /// which may happen due to format mismatches or data corruption.
    /// 
    /// # 中文
    /// 检索并解码给定键处的值。
    /// 如果键不存在或值无法解码，则返回 None，
    /// 这可能是由于格式不匹配或数据损坏导致的。
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

    /// Clears: deletes storage of `mapping[key]`. Implemented via Env::clear_storage; on-chain uses set_storage_or_clear when key is 32 bytes.
    /// 清除：删除 `mapping[key]` 的存储。通过 Env::clear_storage 实现，链上在 key 为 32 字节时使用 set_storage_or_clear。
    /// 
    /// # English
    /// Removes the stored value at the given key.
    /// On-chain, if key is 32 bytes, uses set_storage_or_clear optimization.
    /// 
    /// # 中文
    /// 移除给定键处的存储值。
    /// 链上，如果键为 32 字节，使用 set_storage_or_clear 优化。
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
