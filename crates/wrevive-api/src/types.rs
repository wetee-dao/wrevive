//! Common contract types: Address, H256, U256, BlockNumber, Bytes.
//! 常见合约数据类型：Address、H256、U256、BlockNumber、Bytes。
//!
//! All Solidity ABI encoding is delegated to the underlying pvm_contract_sdk types.
//! Only SCALE Encode/Decode is implemented here.

#[cfg(not(any(test, feature = "off_chain")))]
pub use alloc::vec::Vec;
#[cfg(any(test, feature = "off_chain"))]
pub use std::vec::Vec;

use core::ops::{Add, Div, Mul, Sub};
use parity_scale_codec::{Decode, Encode, Input, Output};
use scale_info::TypeInfo;

use crate::CallMode;

// Re-export the SDK's Sol ABI types so wrappers can delegate to them.
use pvm_contract_sdk as sdk;

// =============================================================================
// Address — delegates Sol ABI to sdk::Address (correct "address" encoding)
// =============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct Address(sdk::Address);

impl Default for Address {
    fn default() -> Self {
        Self(sdk::Address::ZERO)
    }
}

impl Address {
    pub const fn zero() -> Self {
        Self(sdk::Address::ZERO)
    }
}

// SCALE: 20 bytes raw
impl Encode for Address {
    fn size_hint(&self) -> usize {
        20
    }
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        dest.write(&self.0.0);
    }
}
impl Decode for Address {
    fn decode<I: Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        let mut buf = [0u8; 20];
        input.read(&mut buf)?;
        Ok(Self(sdk::Address::from(buf)))
    }
}

impl TypeInfo for Address {
    type Identity = Self;
    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("Address", module_path!()))
            .composite(
                scale_info::build::Fields::unnamed()
                    .field(|b| b.ty::<[u8; 20]>().type_name("[u8; 20]")),
            )
    }
}

// Sol ABI: delegate to sdk::Address
impl pvm_contract_sdk::SolEncode for Address {
    const IS_DYNAMIC: bool = <sdk::Address as pvm_contract_sdk::SolEncode>::IS_DYNAMIC;
    const SOL_NAME: &'static str = <sdk::Address as pvm_contract_sdk::SolEncode>::SOL_NAME;
    fn encode_body_len(&self) -> usize {
        pvm_contract_sdk::SolEncode::encode_body_len(&self.0)
    }
    fn encode_body_to(&self, buf: &mut [u8]) {
        pvm_contract_sdk::SolEncode::encode_body_to(&self.0, buf)
    }
}
impl pvm_contract_sdk::SolDecode for Address {
    fn decode_at(input: &[u8], offset: usize) -> Result<Self, pvm_contract_sdk::DecodeError> {
        <sdk::Address as pvm_contract_sdk::SolDecode>::decode_at(input, offset).map(Address)
    }
}
impl pvm_contract_sdk::StaticEncodedLen for Address {
    const ENCODED_SIZE: usize = <sdk::Address as pvm_contract_sdk::StaticEncodedLen>::ENCODED_SIZE;
}
impl pvm_contract_sdk::StaticDecode for Address {
    unsafe fn decode_unchecked(input: &[u8], offset: usize) -> Self {
        unsafe {
            Self(<sdk::Address as pvm_contract_sdk::StaticDecode>::decode_unchecked(input, offset))
        }
    }
}
impl pvm_contract_sdk::SolArrayElement for Address {}

impl From<[u8; 20]> for Address {
    fn from(b: [u8; 20]) -> Self {
        Self(sdk::Address::from(b))
    }
}
impl From<Address> for [u8; 20] {
    fn from(a: Address) -> Self {
        a.0.into()
    }
}
impl AsRef<[u8; 20]> for Address {
    fn as_ref(&self) -> &[u8; 20] {
        self.0.as_ref()
    }
}

// =============================================================================
// AccountId — 32-byte account ID (not in SDK)
// =============================================================================

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, TypeInfo, Debug)]
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
// H256 — 32-byte hash, delegates Sol ABI to sdk::H256 if available, else bytes32
// =============================================================================

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct H256(pub [u8; 32]);

impl H256 {
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

// SCALE: 32 bytes raw
impl Encode for H256 {
    fn size_hint(&self) -> usize {
        32
    }
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        dest.write(&self.0);
    }
}
impl Decode for H256 {
    fn decode<I: Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        let mut buf = [0u8; 32];
        input.read(&mut buf)?;
        Ok(Self(buf))
    }
}
impl TypeInfo for H256 {
    type Identity = Self;
    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("H256", module_path!()))
            .composite(
                scale_info::build::Fields::unnamed()
                    .field(|b| b.ty::<[u8; 32]>().type_name("[u8; 32]")),
            )
    }
}

// Sol ABI: bytes32 (right-aligned, same as [u8;32])
impl pvm_contract_sdk::SolEncode for H256 {
    const IS_DYNAMIC: bool = false;
    const SOL_NAME: &'static str = "bytes32";
    fn encode_body_len(&self) -> usize {
        32
    }
    fn encode_body_to(&self, buf: &mut [u8]) {
        buf[..32].copy_from_slice(&self.0);
    }
}
impl pvm_contract_sdk::SolDecode for H256 {
    fn decode_at(input: &[u8], offset: usize) -> Result<Self, pvm_contract_sdk::DecodeError> {
        input
            .get(offset..offset + 32)
            .map(|x| {
                let mut r = [0u8; 32];
                r.copy_from_slice(x);
                H256(r)
            })
            .ok_or(pvm_contract_sdk::DecodeError)
    }
}
impl pvm_contract_sdk::StaticEncodedLen for H256 {
    const ENCODED_SIZE: usize = 32;
}
impl pvm_contract_sdk::StaticDecode for H256 {
    unsafe fn decode_unchecked(input: &[u8], offset: usize) -> Self {
        let mut r = [0u8; 32];
        unsafe { r.copy_from_slice(input.get_unchecked(offset..offset + 32)) };
        H256(r)
    }
}
impl pvm_contract_sdk::SolArrayElement for H256 {}

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
// U256 — delegates Sol ABI to sdk::U256 (correct "uint256" encoding)
// =============================================================================

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct U256(pub sdk::U256);

impl Default for U256 {
    fn default() -> Self {
        Self(sdk::U256::ZERO)
    }
}
impl core::fmt::Debug for U256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

// SCALE: little-endian 32 bytes
impl Encode for U256 {
    fn size_hint(&self) -> usize {
        32
    }
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        dest.write(&self.0.to_le_bytes::<32>());
    }
}
impl Decode for U256 {
    fn decode<I: Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        let mut buf = [0u8; 32];
        input.read(&mut buf)?;
        Ok(Self(sdk::U256::from_le_bytes(buf)))
    }
}
impl TypeInfo for U256 {
    type Identity = Self;
    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("U256", module_path!()))
            .composite(
                scale_info::build::Fields::unnamed()
                    .field(|b| b.ty::<[u8; 32]>().type_name("[u8; 32]")),
            )
    }
}

// Sol ABI: delegate to sdk::U256
impl pvm_contract_sdk::SolEncode for U256 {
    const IS_DYNAMIC: bool = <sdk::U256 as pvm_contract_sdk::SolEncode>::IS_DYNAMIC;
    const SOL_NAME: &'static str = <sdk::U256 as pvm_contract_sdk::SolEncode>::SOL_NAME;
    fn encode_body_len(&self) -> usize {
        pvm_contract_sdk::SolEncode::encode_body_len(&self.0)
    }
    fn encode_body_to(&self, buf: &mut [u8]) {
        pvm_contract_sdk::SolEncode::encode_body_to(&self.0, buf)
    }
}
impl pvm_contract_sdk::SolDecode for U256 {
    fn decode_at(input: &[u8], offset: usize) -> Result<Self, pvm_contract_sdk::DecodeError> {
        <sdk::U256 as pvm_contract_sdk::SolDecode>::decode_at(input, offset).map(U256)
    }
}
impl pvm_contract_sdk::StaticEncodedLen for U256 {
    const ENCODED_SIZE: usize = <sdk::U256 as pvm_contract_sdk::StaticEncodedLen>::ENCODED_SIZE;
}
impl pvm_contract_sdk::StaticDecode for U256 {
    unsafe fn decode_unchecked(input: &[u8], offset: usize) -> Self {
        unsafe {
            Self(<sdk::U256 as pvm_contract_sdk::StaticDecode>::decode_unchecked(input, offset))
        }
    }
}
impl pvm_contract_sdk::SolArrayElement for U256 {}

// --- U256: constants & helpers ---
impl U256 {
    pub const ZERO: Self = Self(sdk::U256::ZERO);
    pub const MAX: Self = Self(sdk::U256::MAX);
    pub const ONE: Self = Self({
        let mut b = [0u8; 32];
        b[31] = 1;
        sdk::U256::from_be_bytes(b)
    });

    pub fn to_le_bytes(&self) -> [u8; 32] {
        self.0.to_le_bytes::<32>()
    }
    pub fn from_le_bytes(b: [u8; 32]) -> Self {
        Self(sdk::U256::from_le_bytes(b))
    }
    pub fn to_be_bytes(&self) -> [u8; 32] {
        self.0.to_be_bytes::<32>()
    }
    pub fn from_be_bytes(b: [u8; 32]) -> Self {
        Self(sdk::U256::from_be_bytes(b))
    }
    pub fn from_u64(v: u64) -> Self {
        Self(sdk::U256::from(v))
    }
    pub fn to_u64(&self) -> u64 {
        self.0.try_into().unwrap_or(u64::MAX)
    }

    pub fn as_bytes(&self, mode: CallMode) -> [u8; 32] {
        match mode {
            CallMode::Codec => self.to_le_bytes(),
            CallMode::Sol => self.to_be_bytes(),
        }
    }

    pub fn wrapping_add(self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }
    pub fn wrapping_sub(self, other: Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }
    pub fn wrapping_mul(self, other: Self) -> Self {
        Self(self.0.wrapping_mul(other.0))
    }
    pub fn checked_div(self, other: Self) -> Option<Self> {
        self.0.checked_div(other.0).map(Self)
    }
    pub fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub fn shl_bits(self, n: u32) -> Self {
        if n >= 256 {
            return Self::ZERO;
        }
        Self(self.0 << (n as usize))
    }
}

impl From<u64> for U256 {
    fn from(v: u64) -> Self {
        Self::from_u64(v)
    }
}
impl From<sdk::U256> for U256 {
    fn from(v: sdk::U256) -> Self {
        Self(v)
    }
}
impl From<U256> for sdk::U256 {
    fn from(u: U256) -> Self {
        u.0
    }
}

impl Add for U256 {
    type Output = Self;
    fn add(self, r: Self) -> Self {
        Self(self.0 + r.0)
    }
}
impl Sub for U256 {
    type Output = Self;
    fn sub(self, r: Self) -> Self {
        Self(self.0 - r.0)
    }
}
impl Mul for U256 {
    type Output = Self;
    fn mul(self, r: Self) -> Self {
        Self(self.0 * r.0)
    }
}
impl Div for U256 {
    type Output = Self;
    fn div(self, r: Self) -> Self {
        Self(self.0 / r.0)
    }
}

// =============================================================================
// BlockNumber, Bytes
// =============================================================================

pub type BlockNumber = u32;
pub type Bytes = Vec<u8>;

// =============================================================================
// Tests
// =============================================================================

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
        let mut buf = [0u8; 32];
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
        assert_eq!(
            U256::from_u64(10).wrapping_mul(U256::from_u64(20)),
            U256::from_u64(200)
        );
        assert_eq!(U256::from_u64(7) * U256::from_u64(6), U256::from_u64(42));
    }

    #[test]
    fn u256_add_sub() {
        assert_eq!(U256::ONE.wrapping_sub(U256::ONE), U256::ZERO);
        assert_eq!(U256::from_u64(10) + U256::from_u64(20), U256::from_u64(30));
        assert_eq!(U256::from_u64(99) - U256::from_u64(9), U256::from_u64(90));
    }

    #[test]
    fn u256_div() {
        assert_eq!(U256::ONE.checked_div(U256::ZERO), None);
        assert_eq!(
            U256::from_u64(100).checked_div(U256::from_u64(5)),
            Some(U256::from_u64(20))
        );
        assert_eq!(U256::from_u64(100) / U256::from_u64(4), U256::from_u64(25));
    }
}
