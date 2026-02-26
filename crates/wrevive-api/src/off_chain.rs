//! # Off-chain backend
//!
//! In-memory `Engine` for unit tests; see [ink! engine](https://github.com/use-ink/ink/tree/master/crates/engine) test_api.
//!
//! Off-chain 实现：内存 Engine，用于单元测试（参考 [ink engine](https://github.com/use-ink/ink/tree/master/crates/engine) test_api）。

use crate::env::{Env, CallResult};
use crate::types::{Address, BlockNumber, H256, U256};
use pallet_revive_uapi::{CallFlags, ReturnErrorCode, ReturnFlags, StorageFlags};
#[cfg(feature = "off_chain")]
use sha3::{Digest, Keccak256};
use std::cell::RefCell;
use std::collections::HashMap;

// Thread-local engine instance; env() and test setup use this.
// 线程局部引擎实例；env() 与测试配置均使用此实例。
thread_local! {
    static ENGINE: RefCell<Engine> = RefCell::new(Engine::default());
}

/// Off-chain execution engine: in-memory storage, configurable caller/call_data, and event log.
/// Off-chain 执行引擎：内存 storage、可配置 caller/call_data、记录 events。
#[derive(Default)]
pub struct Engine {
    /// Key-value storage (same semantics as chain storage).
    /// 键值存储（与链上 storage 语义一致）。
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
    /// Input bytes for the current call (read by call_data_size / call_data_copy).
    /// 当前调用的输入字节（由 call_data_size / call_data_copy 读取）。
    pub call_data: Vec<u8>,
    /// Caller address (20 bytes); set via set_caller in tests.
    /// 调用方地址（20 字节）；测试中通过 set_caller 设置。
    pub caller: [u8; 20],
    /// Events emitted (topics, data); use take_events() for assertions.
    /// 已发出的事件（topics, data）；可用 take_events() 做断言。
    pub events: Vec<(Vec<[u8; 32]>, Vec<u8>)>,
    /// Last return_value(flags, data) call; None until contract returns.
    /// 最近一次 return_value(flags, data) 的调用；合约返回前为 None。
    pub return_value: Option<(ReturnFlags, Vec<u8>)>,
}

impl Engine {
    /// Set the caller for the next contract call (like ink! engine set_caller).
    /// 设置下一次调用的 caller（同 ink engine set_caller）。
    pub fn set_caller(&mut self, addr: [u8; 20]) {
        self.caller = addr;
    }

    /// Set call input for the current invocation (read by call_data_size / call_data_copy).
    /// 设置当前调用的 call_data（供 call_data_size / call_data_copy 读取）。
    pub fn set_call_data(&mut self, data: &[u8]) {
        self.call_data = data.to_vec();
    }

    /// Get the value at `key` for test assertions.
    /// 读取某 key 的 storage 值（测试断言用）。
    pub fn get_storage_value(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.get(key).cloned()
    }

    /// Take all recorded events (clears internal list); use for assertions.
    /// 取出已记录的 events（测试断言用）。
    pub fn take_events(&mut self) -> Vec<(Vec<[u8; 32]>, Vec<u8>)> {
        std::mem::take(&mut self.events)
    }

    /// Reset engine state (like ink! engine initialize_or_reset).
    /// 重置引擎状态（同 ink engine initialize_or_reset）。
    pub fn reset(&mut self) {
        self.storage.clear();
        self.call_data.clear();
        self.events.clear();
        self.return_value = None;
        self.caller = [0u8; 20];
    }
}

/// Run a closure with exclusive access to the thread-local Engine (configure or assert).
/// 在 off_chain 上下文中运行闭包（配置 Engine 或做断言）。同 ink 里对 Engine 的配置。
pub fn with_engine<F, R>(f: F) -> R
where
    F: FnOnce(&mut Engine) -> R,
{
    ENGINE.with(|cell| f(&mut *cell.borrow_mut()))
}

/// Off-chain 对 Env 的实现，委托给 thread_local ENGINE。
pub struct OffChainEnv;

impl Env for OffChainEnv {
    fn caller(&self) -> Address {
        ENGINE.with(|cell| Address::from(cell.borrow().caller))
    }
    fn set_storage(&self, _flags: StorageFlags, key: &[u8], value: &[u8]) -> Option<u32> {
        ENGINE.with(|cell| {
            let prev = cell.borrow().storage.get(key).map(|v| v.len() as u32);
            cell.borrow_mut().storage.insert(key.to_vec(), value.to_vec());
            prev
        })
    }
    fn get_storage(
        &self,
        _flags: StorageFlags,
        key: &[u8],
    ) -> Result<Vec<u8>, ReturnErrorCode> {
        ENGINE.with(|cell| {
            cell.borrow().storage.get(key).cloned().ok_or(ReturnErrorCode::KeyNotFound)
        })
    }



    fn deposit_event(&self, topics: &[[u8; 32]], data: &[u8]) {
        ENGINE.with(|cell| {
            cell.borrow_mut()
                .events
                .push((topics.to_vec(), data.to_vec()));
        });
    }

    fn return_value(&self, flags: ReturnFlags, return_value: &[u8]) -> ! {
        ENGINE.with(|cell| {
            cell.borrow_mut().return_value = Some((flags, return_value.to_vec()));
        });
        panic!("off_chain return_value: flags={:?}, len={}", flags, return_value.len());
    }

    fn call_data_size(&self) -> u64 {
        ENGINE.with(|cell| cell.borrow().call_data.len() as u64)
    }
    
    /// Copy `len` bytes from call data at `offset`; pad with zeros if beyond end.
    /// 从 call data 的 offset 处复制 len 字节；若越界则用零填充。
    fn call_data_copy(&self, offset: u32, len: usize) -> Vec<u8> {
        ENGINE.with(|cell| {
            let data = &cell.borrow().call_data;
            let off = offset as usize;
            // 实际可复制长度，避免越界 / actual copy length to avoid out-of-bounds
            let actual_len = len.min(data.len().saturating_sub(off));
            if actual_len > 0 {
                data[off..off + actual_len].to_vec()
            } else {
                vec![0u8; len]
            }
        })
    }

    fn address(&self) -> Address {
        // Off-chain: return zero address as default
        Address::zero()
    }

    fn get_immutable_data(&self, _output: &mut &mut [u8]) {
        // Off-chain: no immutable data by default
    }

    fn set_immutable_data(&self, _data: &[u8]) {
        // Off-chain: no-op
    }

    fn balance(&self) -> U256 {
        // Off-chain: return zero balance
        U256::ZERO
    }

    fn balance_of(&self, _addr: &[u8; 20]) -> U256 {
        // Off-chain: return zero balance
        U256::ZERO
    }

    fn chain_id(&self) -> [u8; 32] {
        // Off-chain: return default chain ID (1)
        let mut output = [0u8; 32];
        output[31] = 1;
        output
    }

    fn gas_price(&self) -> u64 {
        // Off-chain: return default gas price
        1
    }

    fn base_fee(&self) -> U256 {
        // Off-chain: return zero base fee
        U256::ZERO
    }

    fn call(
        &self,
        _flags: CallFlags,
        _callee: &Address,
        _ref_time_limit: u64,
        _proof_size_limit: u64,
        _deposit: &U256,
        _value: &U256,
        _input_data: &[u8],
        _output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        // Off-chain: calls not supported, return error
        Err(ReturnErrorCode::CalleeTrapped)
    }

    fn origin(&self) -> [u8; 20] {
        // Off-chain: return caller as origin
        *self.caller().as_ref()
    }

    fn code_hash(&self, _addr: &[u8; 20]) -> H256 {
        // Off-chain: return zero hash
        H256::zero()
    }

    fn code_size(&self, _addr: &[u8; 20]) -> u64 {
        // Off-chain: return zero size
        0
    }

    fn delegate_call(
        &self,
        _flags: CallFlags,
        _address: &Address,
        _ref_time_limit: u64,
        _proof_size_limit: u64,
        _deposit_limit: &U256,
        _input_data: &[u8],
        _output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        // Off-chain: delegate calls not supported, return error
        Err(ReturnErrorCode::CalleeTrapped)
    }

    fn hash_keccak_256(&self, input: &[u8]) -> H256 {
        // Off-chain: use sha3 crate for Keccak-256
        #[cfg(feature = "off_chain")]
        {
            let hash = Keccak256::digest(input);
            let mut output = [0u8; 32];
            output.copy_from_slice(&hash);
            H256::from(output)
        }
        #[cfg(not(feature = "off_chain"))]
        {
            // Fallback: return zero hash if sha3 is not available
            H256::zero()
        }
    }

    /// Load 32 bytes from call data at offset (EVM-style); zero-pad if shorter.
    /// 从 offset 处加载 32 字节（EVM 风格）；不足则零填充。
    fn call_data_load(&self, offset: u32) -> [u8; 32] {
        ENGINE.with(|cell| {
            let data = &cell.borrow().call_data;
            let off = offset as usize;
            let mut output = [0u8; 32];
            if off < data.len() {
                let len = 32.min(data.len() - off);
                output[..len].copy_from_slice(&data[off..off + len]);
            }
            output
        })
    }

    fn instantiate(
        &self,
        _flags: CallFlags,
        _code_hash: &[u8; 32],
        _ref_time_limit: u64,
        _proof_size_limit: u64,
        _deposit: &[u8; 32],
        _value: &[u8; 32],
        _input_data: &[u8],
        address: &mut [u8; 20],
        _output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        // Off-chain: generate a mock address
        address.fill(0);
        Err(ReturnErrorCode::CalleeTrapped)
    }

    /// Return current block number (off-chain mock: 1).
    /// 返回当前区块高度（链下模拟为 1）。
    fn now(&self) -> BlockNumber {
        1
    }

    fn gas_limit(&self) -> u64 {
        // Off-chain: return default gas limit
        u64::MAX
    }

    fn set_storage_or_clear(&self, flags: StorageFlags, key: &[u8; 32], value: &[u8; 32]) -> Option<u32> {
        // Off-chain: if value is all zeros, clear storage
        if value.iter().all(|&b| b == 0) {
            ENGINE.with(|cell| {
                let key_vec = key.to_vec();
                let prev = cell.borrow().storage.get(&key_vec).map(|v| v.len() as u32);
                cell.borrow_mut().storage.remove(&key_vec);
                prev
            })
        } else {
            self.set_storage(flags, key, value)
        }
    }

    /// Get storage or return 32 zero bytes if key not found.
    /// 获取存储；若 key 不存在则返回 32 字节零。
    fn get_storage_or_zero(&self, flags: StorageFlags, key: &[u8; 32]) -> [u8; 32] {
        match self.get_storage(flags, key as &[u8]) {
            Ok(data) => {
                let mut output = [0u8; 32];
                let len = data.len().min(32);
                output[..len].copy_from_slice(&data[..len]);
                output
            }
            Err(_) => [0u8; 32],
        }
    }

    fn value_transferred(&self) -> U256 {
        // Off-chain: return zero value
        U256::ZERO
    }

    fn return_data_size(&self) -> u64 {
        // Off-chain: return zero size
        0
    }

    fn call_evm(
        &self,
        _flags: CallFlags,
        _callee: &Address,
        _gas: u64,
        _value: &U256,
        _input_data: &[u8],
        _output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        Err(ReturnErrorCode::CalleeTrapped)
    }

    fn delegate_call_evm(
        &self,
        _flags: CallFlags,
        _address: &Address,
        _gas: u64,
        _input_data: &[u8],
        _output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        Err(ReturnErrorCode::CalleeTrapped)
    }

    fn return_data_copy(&self, _output: &mut &mut [u8], _offset: u32) {
        // Off-chain: no return data, no-op
    }

    fn gas_left(&self) -> u64 {
        u64::MAX
    }

    fn block_author(&self) -> Address {
        Address::zero()
    }

    fn block_number(&self) -> BlockNumber {
        0
    }

    fn block_hash(&self, _block_number: BlockNumber) -> H256 {
        H256::zero()
    }

    fn consume_all_gas(&self) -> ! {
        panic!("off_chain consume_all_gas")
    }

    fn terminate(&self, _beneficiary: &[u8; 20]) -> ! {
        panic!("off_chain terminate")
    }
}

/// Off-chain Env 静态实例。
pub static OFF_CHAIN_ENV: OffChainEnv = OffChainEnv;
