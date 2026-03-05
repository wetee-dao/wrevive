//! 常见合约数据类型：Address、H256、U256、BlockNumber、Bytes。
//! Common contract types: Address, H256, U256, BlockNumber, Bytes.

#[cfg(not(any(test, feature = "off_chain")))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "off_chain"))]
use std::vec::Vec;

use pvm_contract_macros::SolType;

use core::ops::{Add, Div, Mul, Sub};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

// =============================================================================
// Address — 20 字节地址（EVM/账户兼容）
// =============================================================================

/// 20-byte address (EVM / account compatible). SCALE 编码为 20 字节。
/// 使用透明封装的新类型，方便实现 `zero`、`From` 等方法，同时在 ABI 中可映射为 H160。
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, TypeInfo, SolType, Debug)]
#[repr(transparent)]
pub struct Address(pub [u8; 20]);

impl Address {
    /// 全零地址（20 字节全 0）。
    pub const fn zero() -> Self {
        Self([0u8; 20])
    }
}

impl From<[u8; 20]> for Address {
    #[inline]
    fn from(b: [u8; 20]) -> Self {
        Self(b)
    }
}

impl From<Address> for [u8; 20] {
    #[inline]
    fn from(a: Address) -> Self {
        a.0
    }
}

impl AsRef<[u8; 20]> for Address {
    #[inline]
    fn as_ref(&self) -> &[u8; 20] {
        &self.0
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, TypeInfo, SolType, Debug)]
pub struct AccountId(pub [u8; 32]);

impl AccountId {
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }
}

impl From<[u8; 32]> for AccountId {
    fn from(b: [u8; 32]) -> Self {
        Self(b)
    }
}

impl From<AccountId> for [u8; 32] {
    fn from(a: AccountId) -> Self {
        a.0
    }
}

impl AsRef<[u8; 32]> for AccountId {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}
// =============================================================================
// H256 — 32 字节哈希
// =============================================================================

/// 32-byte hash (e.g. Keccak-256). SCALE 编码为 32 字节。
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, TypeInfo, SolType, Debug)]
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
#[derive(
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Encode,
    Decode,
    TypeInfo,
    SolType,
    Debug
)]
#[repr(transparent)]
pub struct U256(pub [u8; 32]);

/// 内部用 4 个 u64 小端 limbs 做运算：[0]=LSB, [3]=MSB。
fn u256_to_limbs(u: &U256) -> [u64; 4] {
    [
        u64::from_be_bytes(u.0[24..32].try_into().unwrap()),
        u64::from_be_bytes(u.0[16..24].try_into().unwrap()),
        u64::from_be_bytes(u.0[8..16].try_into().unwrap()),
        u64::from_be_bytes(u.0[0..8].try_into().unwrap()),
    ]
}

fn u256_from_limbs(limbs: [u64; 4]) -> U256 {
    let mut b = [0u8; 32];
    b[24..32].copy_from_slice(&limbs[0].to_be_bytes());
    b[16..24].copy_from_slice(&limbs[1].to_be_bytes());
    b[8..16].copy_from_slice(&limbs[2].to_be_bytes());
    b[0..8].copy_from_slice(&limbs[3].to_be_bytes());
    U256(b)
}

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

    /// 左移 `n` 位（0 ≤ n ≤ 255）；n ≥ 256 则结果为 0。
    pub fn shl_bits(self, n: u32) -> Self {
        if n == 0 {
            return self;
        }
        if n >= 256 {
            return Self::ZERO;
        }
        let limbs = u256_to_limbs(&self);
        let n_limb = (n / 64) as usize;
        let n_bits = n % 64;
        let mut out = [0u64; 4];
        if n_bits == 0 {
            for (i, &l) in limbs.iter().enumerate() {
                if i >= n_limb {
                    out[i - n_limb] = l;
                }
            }
        } else {
            let carry_shift = 64 - n_bits;
            for (i, &l) in limbs.iter().enumerate() {
                if i >= n_limb {
                    out[i - n_limb] |= l << n_bits;
                }
                if i > n_limb {
                    out[i - n_limb - 1] |= l >> carry_shift;
                }
            }
        }
        u256_from_limbs(out)
    }

    /// 按位或。
    pub fn bitor(self, other: Self) -> Self {
        let a = u256_to_limbs(&self);
        let b = u256_to_limbs(&other);
        u256_from_limbs([a[0] | b[0], a[1] | b[1], a[2] | b[2], a[3] | b[3]])
    }

    /// 包装加法 self + other（溢出时模 2^256）。
    pub fn wrapping_add(self, other: Self) -> Self {
        let a = u256_to_limbs(&self);
        let b = u256_to_limbs(&other);
        let (r0, c0) = a[0].overflowing_add(b[0]);
        let (r1, _) = a[1].overflowing_add(b[1]);
        let (r2, _) = a[2].overflowing_add(b[2]);
        let (r3, c3) = a[3].overflowing_add(b[3]);
        let (r1, c1) = r1.overflowing_add(c0 as u64);
        let (r2, c2) = r2.overflowing_add(c1 as u64);
        let (r3, _) = r3.overflowing_add(c2 as u64);
        let r3 = r3.wrapping_add(c3 as u64);
        u256_from_limbs([r0, r1, r2, r3])
    }

    /// 包装减法 self − other（假设 self ≥ other，否则结果为模 2^256）。
    pub fn wrapping_sub(self, other: Self) -> Self {
        let a = u256_to_limbs(&self);
        let b = u256_to_limbs(&other);
        let (r0, c0) = a[0].overflowing_sub(b[0]);
        let (r1, _) = a[1].overflowing_sub(b[1]);
        let (r2, _) = a[2].overflowing_sub(b[2]);
        let (r3, c3) = a[3].overflowing_sub(b[3]);
        let (r1, c1) = r1.overflowing_sub(c0 as u64);
        let (r2, c2) = r2.overflowing_sub(c1 as u64);
        let (r3, _) = r3.overflowing_sub(c2 as u64);
        let r3 = r3.wrapping_sub(c3 as u64);
        u256_from_limbs([r0, r1, r2, r3])
    }

    /// 乘法（低 256 位， wrapping）。
    pub fn wrapping_mul(self, other: Self) -> Self {
        let a = u256_to_limbs(&self);
        let b = u256_to_limbs(&other);
        let mut res = [0u64; 8];
        for i in 0..4 {
            let mut carry: u64 = 0;
            for j in 0..4 {
                let k = i + j;
                let p = a[i] as u128 * b[j] as u128 + res[k] as u128 + carry as u128;
                res[k] = p as u64;
                carry = (p >> 64) as u64;
            }
            res[i + 4] = carry;
        }
        u256_from_limbs([res[0], res[1], res[2], res[3]])
    }

    /// 除法；除数为 0 时返回 `None`。
    pub fn checked_div(self, other: Self) -> Option<Self> {
        if other == Self::ZERO {
            return None;
        }
        if self < other {
            return Some(Self::ZERO);
        }
        // 快速路径：两者高 16 字节均为 0 时用 u128 做除法。
        let self_hi_zero = self.0[0..16].iter().all(|&b| b == 0);
        let other_hi_zero = other.0[0..16].iter().all(|&b| b == 0);
        if self_hi_zero && other_hi_zero {
            let mut lo = [0u8; 16];
            lo.copy_from_slice(&self.0[16..32]);
            let sl = u128::from_be_bytes(lo);
            let mut olo = [0u8; 16];
            olo.copy_from_slice(&other.0[16..32]);
            let ol = u128::from_be_bytes(olo);
            if ol == 0 {
                return None;
            }
            let q = sl / ol;
            let mut out = [0u8; 32];
            out[16..32].copy_from_slice(&q.to_be_bytes());
            return Some(Self(out));
        }
        let mut q = Self::ZERO;
        let mut r = self;
        for i in (0..256).rev() {
            let b_shifted = other.shl_bits(i);
            if r >= b_shifted {
                q = q.bitor(Self::ONE.shl_bits(i));
                r = r.wrapping_sub(b_shifted);
            }
        }
        Some(q)
    }
}

impl From<u64> for U256 {
    fn from(v: u64) -> Self {
        Self::from_u64(v)
    }
}

impl Add for U256 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.wrapping_add(rhs)
    }
}

impl Sub for U256 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.wrapping_sub(rhs)
    }
}

impl Mul for U256 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.wrapping_mul(rhs)
    }
}

impl Div for U256 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.checked_div(rhs).expect("U256 division by zero")
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

    #[test]
    fn u256_mul() {
        assert_eq!(U256::ZERO.wrapping_mul(U256::ONE), U256::ZERO);
        assert_eq!(U256::ONE.wrapping_mul(U256::ONE), U256::ONE);
        assert_eq!(
            U256::from_u64(10).wrapping_mul(U256::from_u64(20)),
            U256::from_u64(200)
        );
        assert_eq!(
            U256::from_u64(0xFFFF_FFFF)
                .wrapping_mul(U256::from_u64(2))
                .to_u64(),
            0x1_FFFF_FFFE
        );
        // Mul trait
        assert_eq!(U256::from_u64(7) * U256::from_u64(6), U256::from_u64(42));
    }

    #[test]
    fn u256_add_sub() {
        assert_eq!(U256::ZERO.wrapping_add(U256::ONE), U256::ONE);
        assert_eq!(
            U256::from_u64(100).wrapping_add(U256::from_u64(50)),
            U256::from_u64(150)
        );
        assert_eq!(
            U256::from_u64(100).wrapping_sub(U256::from_u64(30)),
            U256::from_u64(70)
        );
        assert_eq!(U256::ONE.wrapping_sub(U256::ONE), U256::ZERO);
        // Add/Sub traits
        assert_eq!(U256::from_u64(10) + U256::from_u64(20), U256::from_u64(30));
        assert_eq!(U256::from_u64(99) - U256::from_u64(9), U256::from_u64(90));
    }

    #[test]
    fn u256_div() {
        assert_eq!(U256::ZERO.checked_div(U256::ONE), Some(U256::ZERO));
        assert_eq!(U256::ONE.checked_div(U256::ZERO), None);
        assert_eq!(
            U256::from_u64(100).checked_div(U256::from_u64(5)),
            Some(U256::from_u64(20))
        );
        assert_eq!(
            U256::from_u64(99).checked_div(U256::from_u64(5)),
            Some(U256::from_u64(19))
        );
        assert_eq!(
            U256::from_u64(50).checked_div(U256::from_u64(100)),
            Some(U256::ZERO)
        );
        // Div trait
        assert_eq!(U256::from_u64(100) / U256::from_u64(4), U256::from_u64(25));
    }
}
