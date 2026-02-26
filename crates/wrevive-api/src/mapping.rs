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

/// `get` 内部分配的缓冲区大小（字节）；value 序列化后超过此长度将截断并解码失败。
pub const GET_BUF_SIZE: usize = 512;

/// set/get 可能产生的错误：key 不存在或 value 解码失败。
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

/// 命名空间下的键值映射：storage key = `prefix` || `key`，value 为 Scale 编码字节。
/// 泛型 K, V 在初始化时指定；无 subkey。
#[derive(Clone, Copy)]
pub struct Mapping<K, V> {
    prefix: &'static [u8],
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> Mapping<K, V> {
    /// 创建以 `prefix` 为命名空间的 Mapping，key/value 类型由泛型 K/V 指定。
    pub const fn new(prefix: &'static [u8]) -> Self {
        Self {
            prefix,
            _phantom: PhantomData,
        }
    }

    /// 返回 prefix。
    pub const fn prefix(&self) -> &'static [u8] {
        self.prefix
    }

    /// Build full storage key = prefix || key in buf; returns slice of length prefix.len()+key.len().
    /// 将完整 key 写入 buf，返回 prefix||key 的切片；buf 不足时返回 None。
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

    /// 按字节写入：`mapping[key] = value`，使用调用方提供的 `buf`（长度至少 `prefix.len() + key.len()`）。供 Env 使用。
    #[inline]
    pub fn set_bytes(
        &self,
        api: &dyn crate::env::Env,
        key: &[u8],
        buf: &mut [u8],
        value: &[u8],
    ) -> Option<()> {
        let full = self.full_key(key, buf)?;
        api.set_storage_bytes(StorageFlags::empty(), full, value);
        Some(())
    }

    /// 按字节读取：返回 `mapping[key]` 的原始字节；使用调用方提供的 `buf`。供 Env 使用。
    #[inline]
    pub fn get_bytes(
        &self,
        api: &dyn crate::env::Env,
        key: &[u8],
        buf: &mut [u8],
    ) -> Result<Vec<u8>, MappingError> {
        let full = self.full_key(key, buf).ok_or(ReturnErrorCode::KeyNotFound)?;
        api.get_storage_bytes(StorageFlags::empty(), full).map_err(MappingError::KeyNotFound)
    }
}

impl<K, V> Mapping<K, V>
where
    K: parity_scale_codec::Encode,
    V: parity_scale_codec::Encode + parity_scale_codec::Decode,
{
    /// 写入：`mapping[key] = value`。K / V 使用 Scale 编码。
    #[inline]
    pub fn set(&self, api: &dyn crate::env::Env, key: &K, value: &V) -> Option<()> {
        let key_bytes = key.encode();

        #[cfg(not(any(test, feature = "off_chain")))]
        let mut buf = alloc::vec![0u8; self.prefix.len() + key_bytes.len()];
        #[cfg(any(test, feature = "off_chain"))]
        let mut buf = vec![0u8; self.prefix.len() + key_bytes.len()];

        let full = self.full_key(&key_bytes, &mut buf)?;
        let encoded = value.encode();
        api.set_storage_bytes(StorageFlags::empty(), full, &encoded);
        Some(())
    }

    /// 读取：将 `mapping[key]` 解码为 `V`。key 不存在或解码失败返回 `MappingError`。
    #[inline]
    pub fn get(&self, api: &dyn crate::env::Env, key: &K) -> Result<V, MappingError> {
        let key_bytes = key.encode();

        #[cfg(not(any(test, feature = "off_chain")))]
        let mut key_buf = alloc::vec![0u8; self.prefix.len() + key_bytes.len()];
        #[cfg(any(test, feature = "off_chain"))]
        let mut key_buf = vec![0u8; self.prefix.len() + key_bytes.len()];

        let full = self.full_key(&key_bytes, &mut key_buf).ok_or(ReturnErrorCode::KeyNotFound)?;
        let data = api.get_storage_bytes(StorageFlags::empty(), full)?;
        V::decode(&mut &data[..]).map_err(MappingError::Decode)
    }
}
