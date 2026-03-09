//! # On-chain backend
//!
//! On-chain Env implementation: delegates to `pallet_revive_uapi::HostFnImpl` (PolkaVM/RISC-V only).
//! 链上 Env 实现：委托 `pallet_revive_uapi::HostFnImpl`，仅用于 PolkaVM/RISC-V 目标。
//!
//! get_storage 链上使用栈上小缓冲区（256 字节），避免 16K 静态导致合约执行时报错；
//! 单次读取最多 256 字节。set_storage 拒绝 value.len() > 16K 的写入。

use crate::env::{CallResult, Env};
use crate::types::{Address, BlockNumber, H256, U256};
use alloc::vec::Vec;
use pallet_revive_uapi::{
    CallFlags, HostFn, HostFnImpl, ReturnErrorCode, ReturnFlags, StorageFlags,
};
use crate::buffer::{ScopedBuffer, StaticBuffer};
use crate::traits::Storable;

/// On-chain Env: forwards all calls to HostFnImpl (host interface).
/// 链上 Env：所有调用转发给 HostFnImpl（宿主接口）。
pub struct OnChainEnv{
    buffer: StaticBuffer,
}

impl OnChainEnv {
    pub const fn new() -> Self {
        Self { buffer: StaticBuffer::new() }
    }

    #[inline(always)]
    /// Returns a new scoped buffer for the entire scope of the static 16 kB buffer.
    fn scoped_buffer(&mut self) -> ScopedBuffer<'_> {
        ScopedBuffer::from(&mut self.buffer[..])
    }
}

impl Env for OnChainEnv {
    #[inline(always)]
    fn caller(&self) -> Address {
        let mut output = [0u8; 20];
        HostFnImpl::caller(&mut output);
        Address::from(output)
    }
    
    fn set_storage<V>(&mut self, flags: StorageFlags, key: &[u8], value: &V) -> Option<u32>
    where
        V: Storable,
    {
        let mut buffer = self.scoped_buffer();
        let value = buffer.take_storable_encoded(value);

        HostFnImpl::set_storage(flags, key, &value)
    }

    fn get_storage<V>(&mut self, flags: StorageFlags, key: &[u8]) -> Option<V>
    where
        V: Storable,
    {
        let buffer = self.scoped_buffer();
        let output = &mut buffer.take_rest();
        match HostFnImpl::get_storage(flags, key, output) {
            Ok(_) => (),
            Err(ReturnErrorCode::KeyNotFound) => return None,
            Err(_) => panic!("encountered unexpected error"),
        }

        V::decode(&mut &output[..]).ok()
    }

    #[inline(always)]
    fn clear_storage(&self, flags: StorageFlags, key: &[u8]) -> Option<u32> {
        if key.len() == 32 {
            let mut k = [0u8; 32];
            k.copy_from_slice(key);
            let zero = [0u8; 32];
            HostFnImpl::set_storage_or_clear(flags, &k, &zero)
        } else {
            HostFnImpl::set_storage(flags, key, &[])
        }
    }

    #[inline(always)]
    fn deposit_event(&self, topics: &[[u8; 32]], data: &[u8]) {
        HostFnImpl::deposit_event(topics, data);
    }

    #[inline(always)]
    fn return_value(&self, flags: ReturnFlags, return_value: &[u8]) -> ! {
        HostFnImpl::return_value(flags, return_value)
    }

    #[inline(always)]
    fn call_data_size(&self) -> u64 {
        HostFnImpl::call_data_size()
    }

    #[inline(always)]
    fn call_data_copy(&self, offset: u32, len: usize) -> Vec<u8> {
        let mut output = alloc::vec![0u8; len];
        HostFnImpl::call_data_copy(&mut output, offset);
        output
    }

    #[inline(always)]
    fn address(&self) -> Address {
        let mut output = [0u8; 20];
        HostFnImpl::address(&mut output);
        Address::from(output)
    }

    #[inline(always)]
    fn get_immutable_data(&self, output: &mut &mut [u8]) {
        HostFnImpl::get_immutable_data(output);
    }

    #[inline(always)]
    fn set_immutable_data(&self, data: &[u8]) {
        HostFnImpl::set_immutable_data(data);
    }

    #[inline(always)]
    fn balance(&self) -> U256 {
        let mut output = [0u8; 32];
        HostFnImpl::balance(&mut output);
        U256::from_be_bytes(output)
    }

    #[inline(always)]
    fn balance_of(&self, addr: &[u8; 20]) -> U256 {
        let mut output = [0u8; 32];
        HostFnImpl::balance_of(addr, &mut output);
        U256::from_be_bytes(output)
    }

    #[inline(always)]
    fn chain_id(&self) -> [u8; 32] {
        let mut output = [0u8; 32];
        HostFnImpl::chain_id(&mut output);
        output
    }

    #[inline(always)]
    fn gas_price(&self) -> u64 {
        HostFnImpl::gas_price()
    }

    #[inline(always)]
    fn base_fee(&self) -> U256 {
        let mut output = [0u8; 32];
        HostFnImpl::base_fee(&mut output);
        U256::from_be_bytes(output)
    }

    #[inline(always)]
    fn origin(&self) -> [u8; 20] {
        let mut output = [0u8; 20];
        HostFnImpl::origin(&mut output);
        output
    }

    #[inline(always)]
    fn code_hash(&self, addr: &[u8; 20]) -> H256 {
        let mut output = [0u8; 32];
        HostFnImpl::code_hash(addr, &mut output);
        H256::from(output)
    }

    #[inline(always)]
    fn code_size(&self, addr: &[u8; 20]) -> u64 {
        HostFnImpl::code_size(addr)
    }

    #[inline(always)]
    fn call(
        &self,
        flags: CallFlags,
        callee: &Address,
        ref_time_limit: u64,
        proof_size_limit: u64,
        deposit: &U256,
        value: &U256,
        input_data: &[u8],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        HostFnImpl::call(
            flags,
            callee.as_ref(),
            ref_time_limit,
            proof_size_limit,
            deposit.as_bytes(),
            value.as_bytes(),
            input_data,
            output,
        )
    }

    /// 合约向账户转帐：通过 call 空 data + value 实现。
    #[inline(always)]
    fn transfer(&self, to: &Address, value: &U256) -> CallResult {
        let deposit = U256::ZERO;
        HostFnImpl::call(
            CallFlags::empty(),
            to.as_ref(),
            10_000_000,
            10_000_000,
            deposit.as_bytes(),
            value.as_bytes(),
            &[],
            None,
        )
    }

    #[inline(always)]
    fn delegate_call(
        &self,
        flags: CallFlags,
        address: &Address,
        ref_time_limit: u64,
        proof_size_limit: u64,
        deposit_limit: &U256,
        input_data: &[u8],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        HostFnImpl::delegate_call(
            flags,
            address.as_ref(),
            ref_time_limit,
            proof_size_limit,
            deposit_limit.as_bytes(),
            input_data,
            output,
        )
    }

    #[inline(always)]
    fn hash_keccak_256(&self, input: &[u8]) -> H256 {
        let mut output = [0u8; 32];
        HostFnImpl::hash_keccak_256(input, &mut output);
        H256::from(output)
    }

    #[inline(always)]
    fn call_data_load(&self, offset: u32) -> [u8; 32] {
        let mut output = [0u8; 32];
        HostFnImpl::call_data_load(&mut output, offset);
        output
    }

    #[inline(always)]
    fn instantiate(
        &self,
        _flags: CallFlags,
        code_hash: &[u8; 32],
        ref_time_limit: u64,
        proof_size_limit: u64,
        deposit: &[u8; 32],
        value: &[u8; 32],
        input_data: &[u8],
        address: &mut [u8; 20],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        // HostFnImpl::instantiate 的 input = code_hash(32 字节) + 构造函数 call data
        let mut input = Vec::with_capacity(32 + input_data.len());
        input.extend_from_slice(code_hash);
        input.extend_from_slice(input_data);
        HostFnImpl::instantiate(
            ref_time_limit,
            proof_size_limit,
            deposit,
            value,
            &input,
            Some(address),
            output,
            None, // salt 由上层或 host 决定，此处不传
        )
    }

    #[inline(always)]
    fn now(&self) -> BlockNumber {
        let mut output = [0u8; 32];
        HostFnImpl::now(&mut output);
        // 取低 4 字节小端为 BlockNumber（u32）/ first 4 bytes LE as block number
        BlockNumber::from_le_bytes(output[0..4].try_into().unwrap())
    }

    #[inline(always)]
    fn gas_limit(&self) -> u64 {
        HostFnImpl::gas_limit()
    }

    #[inline(always)]
    fn set_storage_or_clear(
        &mut self,
        flags: StorageFlags,
        key: &[u8; 32],
        value: &[u8; 32],
    ) -> Option<u32> {
        HostFnImpl::set_storage_or_clear(flags, key, value)
    }

    #[inline(always)]
    fn get_storage_or_zero(&mut self, flags: StorageFlags, key: &[u8; 32]) -> [u8; 32] {
        let mut output = [0u8; 32];
        HostFnImpl::get_storage_or_zero(flags, key, &mut output);
        output
    }

    #[inline(always)]
    fn value_transferred(&self) -> U256 {
        let mut output = [0u8; 32];
        HostFnImpl::value_transferred(&mut output);
        U256::from_be_bytes(output)
    }

    #[inline(always)]
    fn return_data_size(&self) -> u64 {
        HostFnImpl::return_data_size()
    }

    #[inline(always)]
    fn call_evm(
        &self,
        flags: CallFlags,
        callee: &Address,
        gas: u64,
        value: &U256,
        input_data: &[u8],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        HostFnImpl::call_evm(
            flags,
            callee.as_ref(),
            gas,
            value.as_bytes(),
            input_data,
            output,
        )
    }

    #[inline(always)]
    fn delegate_call_evm(
        &self,
        flags: CallFlags,
        address: &Address,
        gas: u64,
        input_data: &[u8],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        HostFnImpl::delegate_call_evm(flags, address.as_ref(), gas, input_data, output)
    }

    #[inline(always)]
    fn return_data_copy(&self, output: &mut &mut [u8], offset: u32) {
        HostFnImpl::return_data_copy(output, offset);
    }

    #[inline(always)]
    fn gas_left(&self) -> u64 {
        HostFnImpl::gas_left()
    }

    #[inline(always)]
    fn block_author(&self) -> Address {
        let mut output = [0u8; 20];
        HostFnImpl::block_author(&mut output);
        Address::from(output)
    }

    #[inline(always)]
    fn block_number(&self) -> BlockNumber {
        let mut output = [0u8; 32];
        HostFnImpl::block_number(&mut output);
        BlockNumber::from_le_bytes(output[0..4].try_into().unwrap())
    }

    #[inline(always)]
    fn block_hash(&self, block_number: BlockNumber) -> H256 {
        let mut bn_buf = [0u8; 32];
        bn_buf[0..4].copy_from_slice(&block_number.to_le_bytes());
        let mut output = [0u8; 32];
        HostFnImpl::block_hash(&bn_buf, &mut output);
        H256::from(output)
    }

    #[inline(always)]
    fn consume_all_gas(&self) -> ! {
        HostFnImpl::consume_all_gas()
    }

    #[inline(always)]
    fn terminate(&self, beneficiary: &[u8; 20]) -> ! {
        HostFnImpl::terminate(beneficiary)
    }
}

/// 链上 Env 静态实例。
pub static mut ON_CHAIN_ENV: OnChainEnv = OnChainEnv::new();
