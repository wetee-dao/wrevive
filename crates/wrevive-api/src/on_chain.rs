//! # On-chain backend
//!
//! On-chain 实现：委托 `pallet_revive_uapi::HostFnImpl`，仅 PolkaVM/RISC-V 目标。

use alloc::vec::Vec;
use crate::env::{Env, CallResult};
use pallet_revive_uapi::{CallFlags, HostFn, HostFnImpl, ReturnErrorCode, ReturnFlags, StorageFlags};

/// On-chain 对 Env 的实现，委托 HostFnImpl。
pub struct OnChainEnv;

impl Env for OnChainEnv {
    #[inline(always)]
    fn caller(&self) -> [u8; 20] {
        let mut output = [0u8; 20];
        HostFnImpl::caller(&mut output);
        output
    }

    #[inline(always)]
    fn set_storage_bytes(&self, flags: StorageFlags, key: &[u8], value: &[u8]) -> Option<u32> {
        HostFnImpl::set_storage(flags, key, value)
    }

    #[inline(always)]
    fn get_storage_bytes(
        &self,
        flags: StorageFlags,
        key: &[u8],
    ) -> Result<Vec<u8>, ReturnErrorCode> {
        let mut buf = alloc::vec![0u8; 256];
        let mut cursor: &mut [u8] = buf.as_mut_slice();
        HostFnImpl::get_storage(flags, key, &mut cursor)?;
        let written = 256 - cursor.len();
        Ok(buf[..written].to_vec())
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
    fn address(&self) -> [u8; 20] {
        let mut output = [0u8; 20];
        HostFnImpl::address(&mut output);
        output
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
    fn balance(&self) -> [u8; 32] {
        let mut output = [0u8; 32];
        HostFnImpl::balance(&mut output);
        output
    }

    #[inline(always)]
    fn balance_of(&self, addr: &[u8; 20]) -> [u8; 32] {
        let mut output = [0u8; 32];
        HostFnImpl::balance_of(addr, &mut output);
        output
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
    fn base_fee(&self) -> [u8; 32] {
        let mut output = [0u8; 32];
        HostFnImpl::base_fee(&mut output);
        output
    }

    #[inline(always)]
    fn call(
        &self,
        flags: CallFlags,
        callee: &[u8; 20],
        ref_time_limit: u64,
        proof_size_limit: u64,
        deposit: &[u8; 32],
        value: &[u8; 32],
        input_data: &[u8],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        HostFnImpl::call(flags, callee, ref_time_limit, proof_size_limit, deposit, value, input_data, output)
    }

    #[inline(always)]
    fn origin(&self) -> [u8; 20] {
        let mut output = [0u8; 20];
        HostFnImpl::origin(&mut output);
        output
    }

    #[inline(always)]
    fn code_hash(&self, addr: &[u8; 20]) -> [u8; 32] {
        let mut output = [0u8; 32];
        HostFnImpl::code_hash(addr, &mut output);
        output
    }

    #[inline(always)]
    fn code_size(&self, addr: &[u8; 20]) -> u64 {
        HostFnImpl::code_size(addr)
    }

    #[inline(always)]
    fn delegate_call(
        &self,
        flags: CallFlags,
        address: &[u8; 20],
        ref_time_limit: u64,
        proof_size_limit: u64,
        deposit_limit: &[u8; 32],
        input_data: &[u8],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        HostFnImpl::delegate_call(flags, address, ref_time_limit, proof_size_limit, deposit_limit, input_data, output)
    }

    #[inline(always)]
    fn hash_keccak_256(&self, input: &[u8]) -> [u8; 32] {
        let mut output = [0u8; 32];
        HostFnImpl::hash_keccak_256(input, &mut output);
        output
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
        _input_data: &[u8],
        address: &mut [u8; 20],
        output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        HostFnImpl::instantiate(
            proof_size_limit,
            ref_time_limit,
            code_hash,
            value,
            deposit,
            Some(address),
            output,
            None,
        )
    }

    #[inline(always)]
    fn now(&self) -> [u8; 32] {
        let mut output = [0u8; 32];
        HostFnImpl::now(&mut output);
        output
    }

    #[inline(always)]
    fn gas_limit(&self) -> u64 {
        HostFnImpl::gas_limit()
    }

    #[inline(always)]
    fn set_storage_or_clear(&self, flags: StorageFlags, key: &[u8; 32], value: &[u8; 32]) -> Option<u32> {
        HostFnImpl::set_storage_or_clear(flags, key, value)
    }

    #[inline(always)]
    fn get_storage_or_zero(&self, flags: StorageFlags, key: &[u8; 32]) -> [u8; 32] {
        let mut output = [0u8; 32];
        HostFnImpl::get_storage_or_zero(flags, key, &mut output);
        output
    }

    #[inline(always)]
    fn value_transferred(&self) -> [u8; 32] {
        let mut output = [0u8; 32];
        HostFnImpl::value_transferred(&mut output);
        output
    }

    #[inline(always)]
    fn weight_to_fee(&self, _ref_time_limit: u64, _proof_size_limit: u64) -> [u8; 32] {
        // HostFnImpl doesn't have weight_to_fee, return zero fee
        [0u8; 32]
    }

    #[inline(always)]
    fn return_data_size(&self) -> u64 {
        HostFnImpl::return_data_size()
    }
}

/// 链上 Env 静态实例。
pub static ON_CHAIN_ENV: OnChainEnv = OnChainEnv;
