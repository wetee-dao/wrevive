//! 常见合约数据类型：Address、H256、U256、BlockNumber、Bytes。
//! Common contract types: Address, H256, U256, BlockNumber, Bytes.

#[cfg(not(any(test, feature = "off_chain")))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "off_chain"))]
use std::vec::Vec;

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

// =============================================================================
// Address — 20 字节地址（EVM/账户兼容）
// =============================================================================

/// 20-byte address (EVM / account compatible). SCALE 编码为 20 字节。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, TypeInfo, Debug)]
#[repr(transparent)]
pub struct Address(pub [u8; 20]);

impl Address {
    pub const fn zero() -> Self {
        Self([0u8; 20])
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl From<[u8; 20]> for Address {
    fn from(b: [u8; 20]) -> Self {
        Self(b)
    }
}

impl From<Address> for [u8; 20] {
    fn from(a: Address) -> Self {
        a.0
    }
}

impl AsRef<[u8; 20]> for Address {
    fn as_ref(&self) -> &[u8; 20] {
        &self.0
    }
}

// =============================================================================
// H256 — 32 字节哈希
// =============================================================================

/// 32-byte hash (e.g. Keccak-256). SCALE 编码为 32 字节。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, TypeInfo, Debug)]
#[repr(transparent)]
pub struct H256(pub [u8; 32]);

impl H256 {
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for H256 {
    fn from(b: [u8; 32]) -> Self {
        Self(b)
    }
}

impl From<H256> for [u8; 32] {
    fn from(h: H256) -> Self {
        h.0
    }
}

impl AsRef<[u8; 32]> for H256 {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

// =============================================================================
// U256 — 256 位无符号整数（大端存储，SCALE 编码为 32 字节）
// =============================================================================

/// 256-bit unsigned integer. Stored and SCALE-encoded as 32 bytes big-endian (EVM-compatible).
/// 256 位无符号整数；内部及 SCALE 编码均为 32 字节大端（与 EVM 一致）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, TypeInfo, Debug, Default)]
#[repr(transparent)]
pub struct U256(pub [u8; 32]);

impl U256 {
    pub const ZERO: Self = Self([0u8; 32]);
    pub const ONE: Self = Self({
        let mut b = [0u8; 32];
        b[31] = 1;
        b
    });

    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// 从 u64 构造（低 8 字节大端放在 [24..32]）。
    pub fn from_u64(v: u64) -> Self {
        let mut b = [0u8; 32];
        b[24..32].copy_from_slice(&v.to_be_bytes());
        Self(b)
    }

    /// 转为 u64（取低 8 字节大端）；高位非零则截断。
    pub fn to_u64(&self) -> u64 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&self.0[24..32]);
        u64::from_be_bytes(buf)
    }

    /// 大端 32 字节（与内部表示一致，用于 EVM 风格存储/事件）。
    pub fn to_be_bytes(&self) -> [u8; 32] {
        self.0
    }

    /// 从大端 32 字节构造。
    pub fn from_be_bytes(b: [u8; 32]) -> Self {
        Self(b)
    }
}

impl From<u64> for U256 {
    fn from(v: u64) -> Self {
        Self::from_u64(v)
    }
}

// =============================================================================
// BlockNumber — 区块高度类型别名
// =============================================================================

/// Block number type (Substrate 常用 u32).
/// 区块高度类型别名（Substrate 常用 u32）。
pub type BlockNumber = u32;

// =============================================================================
// String / 文本存储
// =============================================================================

/// 用于合约存储/消息的“字符串”类型：SCALE 编码为长度前缀 + 字节。可直接用于 Storage/Mapping。
/// String-like type for storage/messages; SCALE = length-prefixed bytes.
pub type Bytes = Vec<u8>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_encode_decode() {
        let a = Address::from([1u8; 20]);
        let enc = a.encode();
        assert_eq!(enc.len(), 20);
        let dec: Address = Decode::decode(&mut &enc[..]).unwrap();
        assert_eq!(a, dec);
    }

    #[test]
    fn h256_encode_decode() {
        let h = H256::from([2u8; 32]);
        let enc = h.encode();
        assert_eq!(enc.len(), 32);
        let dec: H256 = Decode::decode(&mut &enc[..]).unwrap();
        assert_eq!(h, dec);
    }

    #[test]
    fn u256_from_u64_to_u64() {
        assert_eq!(U256::ZERO.to_u64(), 0);
        assert_eq!(U256::ONE.to_u64(), 1);
        let u = U256::from_u64(12345);
        assert_eq!(u.to_u64(), 12345);
        let u = U256::from(999u64);
        assert_eq!(u.to_u64(), 999);
    }

    #[test]
    fn u256_be_bytes_roundtrip() {
        let b = [0u8; 31];
        let mut buf = [0u8; 32];
        buf[..31].copy_from_slice(&b);
        buf[31] = 1;
        let u = U256::from_be_bytes(buf);
        assert_eq!(u.to_be_bytes(), buf);
    }

    #[test]
    fn u256_encode_decode() {
        let u = U256::from_u64(0x1234_5678);
        let enc = u.encode();
        assert_eq!(enc.len(), 32);
        let dec: U256 = Decode::decode(&mut &enc[..]).unwrap();
        assert_eq!(u, dec);
    }
}
