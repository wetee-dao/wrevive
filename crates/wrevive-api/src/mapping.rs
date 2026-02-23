//! # Mapping
//!
//! 类似 ink! 的 `Mapping<K, V>`：按前缀命名空间 + key 的 set/get 封装，底层通过传入的 Env 调用。
//! **value 必须实现 Scale**（Encode / Decode）；可用 `#[wrevive_api::scale_derive(Encode, Decode, TypeInfo)]` 派生。
//! `set` / `get` 方法支持任意实现 `Encode` 的 key / subkey（如 u8, u16, u32, u64, i8, [u8; N] 等）。

#[cfg(not(any(test, feature = "off_chain")))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "off_chain"))]
use std::vec::Vec;

#[cfg(not(any(test, feature = "off_chain")))]
extern crate alloc;

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

/// 命名空间下的键值映射：storage key = `prefix` || `subkey` || `key`，value 为 Scale 编码字节。
/// set/get 的 key / subkey / value 类型必须实现 `Encode`（key/subkey）或 `Encode` / `Decode`（value）。
/// 可用 `#[scale_derive(Encode, Decode, TypeInfo)]` 派生。
#[derive(Clone, Copy)]
pub struct Mapping {
    prefix: &'static [u8],
}

impl Mapping {
    /// 创建以 `prefix` 为命名空间的 Mapping。
    pub const fn new(prefix: &'static [u8]) -> Self {
        Self { prefix }
    }

    /// 返回 prefix。
    pub const fn prefix(&self) -> &'static [u8] {
        self.prefix
    }

    /// 将完整 key 写入 `subkey`，返回 `prefix || key` 的切片；若 `subkey.len() < prefix.len() + key.len()` 则返回 `None`。
    #[inline]
    pub fn full_key<'a>(&self, key: &[u8], subkey: &'a mut [u8]) -> Option<&'a [u8]> {
        let n = self.prefix.len().saturating_add(key.len());
        if subkey.len() < n {
            return None;
        }
        subkey[..self.prefix.len()].copy_from_slice(self.prefix);
        subkey[self.prefix.len()..n].copy_from_slice(key);
        Some(&subkey[..n])
    }

    /// 将完整 key 写入 `buf`，返回 `prefix || subkey || key` 的切片；若 `buf.len() < prefix.len() + subkey.len() + key.len()` 则返回 `None`。
    #[inline]
    pub fn full_key_with_subkey<'a>(&self, subkey: &[u8], key: &[u8], buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let n = self.prefix.len().saturating_add(subkey.len()).saturating_add(key.len());
        if buf.len() < n {
            return None;
        }
        let mut offset = 0;
        buf[offset..offset + self.prefix.len()].copy_from_slice(self.prefix);
        offset += self.prefix.len();
        buf[offset..offset + subkey.len()].copy_from_slice(subkey);
        offset += subkey.len();
        buf[offset..offset + key.len()].copy_from_slice(key);
        Some(&buf[..n])
    }

    /// 写入：`mapping[key] = value`。key / subkey / value 使用 Scale 编码。
    #[inline]
    pub fn set<K: parity_scale_codec::Encode + ?Sized, S: parity_scale_codec::Encode + ?Sized, V: parity_scale_codec::Encode + ?Sized>(
        &self,
        api: &dyn crate::env::Env,
        key: &K,
        subkey: &S,
        value: &V,
    ) -> Option<()> {
        let key_bytes = key.encode();
        let subkey_bytes = subkey.encode();
        
        #[cfg(not(any(test, feature = "off_chain")))]
        let mut buf = alloc::vec![0u8; self.prefix.len() + subkey_bytes.len() + key_bytes.len()];
        #[cfg(any(test, feature = "off_chain"))]
        let mut buf = vec![0u8; self.prefix.len() + subkey_bytes.len() + key_bytes.len()];
        
        let full = self.full_key_with_subkey(&subkey_bytes, &key_bytes, &mut buf)?;
        let encoded = value.encode();
        api.set_storage_bytes(StorageFlags::empty(), full, &encoded);
        Some(())
    }

    /// 读取：将 `mapping[key]` 解码为 `V`。key / subkey 使用 Scale 编码；key 不存在或解码失败返回 `MappingError`。
    #[inline]
    pub fn get<K: parity_scale_codec::Encode + ?Sized, S: parity_scale_codec::Encode + ?Sized, V: parity_scale_codec::Decode>(
        &self,
        api: &dyn crate::env::Env,
        key: &K,
        subkey: &S,
    ) -> Result<V, MappingError> {
        let key_bytes = key.encode();
        let subkey_bytes = subkey.encode();
        
        #[cfg(not(any(test, feature = "off_chain")))]
        let mut key_buf = alloc::vec![0u8; self.prefix.len() + subkey_bytes.len() + key_bytes.len()];
        #[cfg(any(test, feature = "off_chain"))]
        let mut key_buf = vec![0u8; self.prefix.len() + subkey_bytes.len() + key_bytes.len()];
        
        let full = self.full_key_with_subkey(&subkey_bytes, &key_bytes, &mut key_buf)
            .ok_or(ReturnErrorCode::KeyNotFound)?;
        let data = api.get_storage_bytes(StorageFlags::empty(), full)?;
        V::decode(&mut &data[..]).map_err(MappingError::Decode)
    }

    /// 按字节写入：`mapping[key] = value`，使用调用方提供的 `subkey` 缓冲（长度至少 `prefix.len() + key.len()`）。供 Env 使用。
    #[inline]
    pub fn set_bytes(
        &self,
        api: &dyn crate::env::Env,
        key: &[u8],
        subkey: &mut [u8],
        value: &[u8],
    ) -> Option<()> {
        let full = self.full_key(key, subkey)?;
        api.set_storage_bytes(StorageFlags::empty(), full, value);
        Some(())
    }

    /// 按字节读取：返回 `mapping[key]` 的原始字节；使用调用方提供的 `subkey` 缓冲。供 Env 使用。
    #[inline]
    pub fn get_bytes(
        &self,
        api: &dyn crate::env::Env,
        key: &[u8],
        subkey: &mut [u8],
    ) -> Result<Vec<u8>, MappingError> {
        let full = self.full_key(key, subkey).ok_or(ReturnErrorCode::KeyNotFound)?;
        api.get_storage_bytes(StorageFlags::empty(), full)
            .map_err(|e| MappingError::KeyNotFound(e))
    }
}