//! # Off-chain backend
//!
//! In-memory `Engine` for unit tests; supports single- and multi-contract (cross-contract call).
//! See [ink! engine](https://github.com/use-ink/ink/tree/master/crates/engine) test_api.
//!
//! ## Single-contract tests
//! Use [with_engine] to set `set_caller` / `set_call_data`, then call the contract's `deploy()` or
//! message functions. Storage is keyed by [Engine::current_contract] (default `[0u8;20]`). Use
//! [Engine::set_contract] if you want a specific address for the contract under test.
//!
//! ## Multi-contract (cross-contract) tests
//! 1. Assign an address to each contract (e.g. `cloud_addr`, `pod_addr`).
//! 2. [Engine::register_contract](addr, dispatcher): register the callee's `call()` entry, e.g.
//!    `e.register_contract(pod_addr, || pod::pod::call());`
//! 3. [Engine::set_contract](main_contract_addr) and [Engine::set_caller](caller).
//! 4. Call the main contract; when it does `env().call(pod_addr, ...)`, the engine runs the
//!    registered dispatcher for `pod_addr` in a new frame (its storage and call_data), then returns
//!    the callee's `return_value` to the caller via [Env::return_data_size] / [Env::return_data_copy].
//! 5. [Engine::get_storage_value_at](contract_addr, key) to assert on a specific contract's storage.
//!
//! Off-chain 实现：内存 Engine，支持单合约与多合约（跨合约调用）测试。

use crate::env::{Env, CallResult};
use crate::traits::Storable;
use crate::types::{Address, BlockNumber, H256, U256};
use pallet_revive_uapi::{CallFlags, ReturnErrorCode, ReturnFlags, StorageFlags};
#[cfg(feature = "off_chain")]
use sha3::{Digest, Keccak256};
use std::panic::{self, AssertUnwindSafe, UnwindSafe};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// Panic payload used when a contract returns via return_value(); caught in call() to implement cross-contract return.
/// 合约通过 return_value 返回时使用的 panic 载荷；在 call() 中捕获以实现跨合约返回。
#[derive(Debug)]
pub struct ReturnValuePanic(pub ReturnFlags, pub Vec<u8>);

// Thread-local engine instance; env() and test setup use this.
// 线程局部引擎实例；env() 与测试配置均使用此实例。
thread_local! {
    static ENGINE: RefCell<Engine> = RefCell::new(Engine::default());
}

/// One frame on the call stack when executing a cross-contract call.
/// 跨合约调用时调用栈中的一帧。
#[derive(Clone)]
struct CallFrame {
    contract: [u8; 20],
    caller: [u8; 20],
    call_data: Vec<u8>,
    value: U256,
}

/// Dispatcher for a contract: when call(callee) is invoked, the engine runs this to execute the callee's `call()`.
/// 合约的派发器：当 call(callee) 被调用时，引擎执行此闭包以运行被调合约的 call()。
/// Uses Arc so the engine can release the borrow before invoking (to allow reentrant env() access).
pub type Dispatcher = Arc<dyn Fn() + UnwindSafe + Send + 'static>;

/// Off-chain execution engine: per-contract storage, call stack, and optional cross-contract dispatchers.
/// Off-chain 执行引擎：按合约的 storage、调用栈、可选的跨合约 dispatcher。
#[derive(Default)]
pub struct Engine {
    /// Per-contract storage. Key = contract address (20 bytes). Unset current_contract uses [0u8;20] for backward compat.
    /// 按合约的存储。key = 合约地址(20 字节)。未设置时 current_contract 为 [0u8;20]，兼容单合约测试。
    pub contract_storages: HashMap<[u8; 20], HashMap<Vec<u8>, Vec<u8>>>,
    /// Call stack for nested calls. Top frame is the current contract context.
    /// 嵌套调用的调用栈。栈顶为当前合约上下文。
    call_stack: Vec<CallFrame>,
    /// Current contract address (whose storage/call_data is active). Default [0u8;20].
    /// 当前合约地址（其 storage/call_data 生效）。默认 [0u8;20]。
    pub current_contract: [u8; 20],
    /// Input bytes for the current call (read by call_data_size / call_data_copy).
    /// 当前调用的输入字节（由 call_data_size / call_data_copy 读取）。
    pub call_data: Vec<u8>,
    /// Caller address (20 bytes); set via set_caller in tests or when entering a sub-call.
    /// 调用方地址（20 字节）；测试中通过 set_caller 设置，或进入子调用时由引擎设置。
    pub caller: [u8; 20],
    /// Value transferred in the current call.
    /// 当前调用转入的 value。
    pub value_transferred: U256,
    /// Events emitted (topics, data); use take_events() for assertions.
    /// 已发出的事件（topics, data）；可用 take_events() 做断言。
    pub events: Vec<(Vec<[u8; 32]>, Vec<u8>)>,
    /// Last return_value(flags, data) from a contract (used by return_data_size / return_data_copy after call).
    /// 最近一次合约 return_value(flags, data)；call 返回后供 return_data_size / return_data_copy 使用。
    pub return_value: Option<(ReturnFlags, Vec<u8>)>,
    /// Registered contract dispatchers: address -> fn() that runs the contract's call().
    /// 已注册的合约派发器：地址 -> 执行该合约 call() 的闭包。
    dispatchers: HashMap<[u8; 20], Dispatcher>,
    /// Next address for instantiate (simple counter-based mock).
    /// instantiate 时分配的下一个地址（简单计数）。
    next_address_counter: u32,
}

impl Engine {
    /// Set the caller for the next contract call (like ink! engine set_caller).
    /// 设置下一次调用的 caller（同 ink engine set_caller）。
    pub fn set_caller(&mut self, addr: [u8; 20]) {
        self.caller = addr;
    }

    /// Set the current contract address (whose storage is used). Use before calling a contract in single-contract tests.
    /// 设置当前合约地址（其 storage 生效）。单合约测试中在调用合约前设置。
    pub fn set_contract(&mut self, addr: Address) {
        self.current_contract = *addr.as_ref();
    }

    /// Register a contract at the given address. When env().call(callee, ...) is invoked, the engine will run this dispatcher.
    /// The dispatcher should invoke the contract's `call()` entry (e.g. `|| pod::pod::call()`).
    /// 在指定地址注册合约。当 env().call(callee, ...) 被调用时，引擎会执行该 dispatcher。
    pub fn register_contract<F>(&mut self, addr: Address, dispatcher: F)
    where
        F: Fn() + UnwindSafe + Send + 'static,
    {
        self.dispatchers.insert(*addr.as_ref(), Arc::new(dispatcher));
        self.contract_storages.entry(*addr.as_ref()).or_default();
    }

    /// Set call input for the current invocation (read by call_data_size / call_data_copy).
    /// 设置当前调用的 call_data（供 call_data_size / call_data_copy 读取）。
    pub fn set_call_data(&mut self, data: &[u8]) {
        self.call_data = data.to_vec();
    }

    /// Get the value at `key` for the current contract (for test assertions).
    /// 读取当前合约某 key 的 storage 值（测试断言用）。
    pub fn get_storage_value(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.contract_storages
            .get(&self.current_contract)
            .and_then(|m| m.get(key).cloned())
    }

    /// Get storage value for a specific contract address (for assertions in multi-contract tests).
    /// 读取指定合约地址的某 key 的 storage 值（多合约测试断言用）。
    pub fn get_storage_value_at(&self, contract: &Address, key: &[u8]) -> Option<Vec<u8>> {
        self.contract_storages
            .get(contract.as_ref())
            .and_then(|m| m.get(key).cloned())
    }

    /// Take all recorded events (clears internal list); use for assertions.
    /// 取出已记录的 events（测试断言用）。
    pub fn take_events(&mut self) -> Vec<(Vec<[u8; 32]>, Vec<u8>)> {
        std::mem::take(&mut self.events)
    }

    /// Reset engine state (like ink! engine initialize_or_reset). Clears storage, events, call stack, and return_value.
    /// Dispatchers and current_contract/caller are left so multi-contract tests can reuse registrations.
    /// 重置引擎状态；清除 storage、events、调用栈和 return_value。dispatcher 与 current_contract/caller 保留。
    pub fn reset(&mut self) {
        self.contract_storages.clear();
        self.call_stack.clear();
        self.call_data.clear();
        self.events.clear();
        self.return_value = None;
        self.caller = [0u8; 20];
        self.value_transferred = U256::ZERO;
        // keep current_contract, dispatchers, next_address_counter for reuse
    }

    /// Reset everything including dispatchers and current contract (full clean slate).
    /// 完全重置，包括 dispatcher 与当前合约。
    pub fn reset_all(&mut self) {
        self.reset();
        self.current_contract = [0u8; 20];
        self.dispatchers.clear();
        self.next_address_counter = 0;
    }

    /// Return the current contract's storage map, if any (for tests that need to inspect multiple keys).
    /// 返回当前合约的 storage（若有）；便于测试中检查多个 key。
    pub fn current_storage(&self) -> Option<&HashMap<Vec<u8>, Vec<u8>>> {
        self.contract_storages.get(&self.current_contract)
    }

    fn storage_mut(&mut self) -> &mut HashMap<Vec<u8>, Vec<u8>> {
        self.contract_storages
            .entry(self.current_contract)
            .or_default()
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


    fn set_storage<V>(&mut self, _flags: StorageFlags, key: &[u8], value: &V) -> Option<u32>
    where
        V: Storable,
    {
        let mut value_bytes = Vec::new();
        value.encode(&mut value_bytes);
        if value_bytes.len() > crate::env::MAX_STORAGE_VALUE_SIZE {
            return None;
        }
        ENGINE.with(|cell| {
            let mut engine = cell.borrow_mut();
            let storage = engine.storage_mut();
            let prev = storage.get(key).map(|v| v.len() as u32);
            if value_bytes.is_empty() {
                storage.remove(key);
            } else {
                storage.insert(key.to_vec(), value_bytes);
            }
            prev
        })
    }

    fn get_storage<V>(&mut self, _flags: StorageFlags, key: &[u8]) -> Option<V>
    where
        V: Storable,
    {
        let value_bytes = ENGINE.with(|cell| {
            let e = cell.borrow();
            e.contract_storages
                .get(&e.current_contract)
                .and_then(|m| m.get(key).cloned())
        })?;
        let truncated = if value_bytes.len() > crate::env::MAX_STORAGE_VALUE_SIZE {
            value_bytes[..crate::env::MAX_STORAGE_VALUE_SIZE].to_vec()
        } else {
            value_bytes
        };
        V::decode(&mut truncated.as_slice()).ok()
    }

    fn clear_storage(&self, _flags: StorageFlags, key: &[u8]) -> Option<u32> {
        ENGINE.with(|cell| {
            let mut engine = cell.borrow_mut();
            let storage = engine.storage_mut();
            let prev = storage.get(key).map(|v| v.len() as u32);
            storage.remove(key);
            prev
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
        panic::panic_any(ReturnValuePanic(flags, return_value.to_vec()));
    }

    fn call_data_size(&self) -> u64 {
        ENGINE.with(|cell| cell.borrow().call_data.len() as u64)
    }

    /// Copy `len` bytes from call data at `offset`; pad with zeros if beyond end.
    fn call_data_copy(&self, offset: u32, len: usize) -> Vec<u8> {
        ENGINE.with(|cell| {
            let data = &cell.borrow().call_data;
            let off = offset as usize;
            let actual_len = len.min(data.len().saturating_sub(off));
            if actual_len > 0 {
                data[off..off + actual_len].to_vec()
            } else {
                vec![0u8; len]
            }
        })
    }

    fn address(&self) -> Address {
        ENGINE.with(|cell| Address::from(cell.borrow().current_contract))
    }

    fn get_immutable_data(&self, _output: &mut &mut [u8]) {}

    fn set_immutable_data(&self, _data: &[u8]) {}

    fn balance(&self) -> U256 {
        U256::ZERO
    }

    fn balance_of(&self, _addr: &[u8; 20]) -> U256 {
        U256::ZERO
    }

    fn chain_id(&self) -> [u8; 32] {
        let mut output = [0u8; 32];
        output[31] = 1;
        output
    }

    fn gas_price(&self) -> u64 {
        1
    }

    fn base_fee(&self) -> U256 {
        U256::ZERO
    }

    fn call(
        &self,
        _flags: CallFlags,
        callee: &Address,
        _ref_time_limit: u64,
        _proof_size_limit: u64,
        _deposit: &U256,
        value: &U256,
        input_data: &[u8],
        _output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        ENGINE.with(|cell| {
            let mut engine = cell.borrow_mut();
            let callee_addr = *callee.as_ref();
            let dispatcher = match engine.dispatchers.get(&callee_addr) {
                Some(d) => Arc::clone(d),
                None => return Err(ReturnErrorCode::CalleeTrapped),
            };

            let frame = CallFrame {
                contract: engine.current_contract,
                caller: engine.caller,
                call_data: std::mem::take(&mut engine.call_data),
                value: engine.value_transferred,
            };
            engine.caller = frame.contract;
            engine.call_stack.push(frame);
            engine.current_contract = callee_addr;
            engine.call_data = input_data.to_vec();
            engine.value_transferred = *value;
            engine.contract_storages.entry(callee_addr).or_default();
            drop(engine);

            let result = panic::catch_unwind(AssertUnwindSafe(move || dispatcher()));

            let mut engine = cell.borrow_mut();
            let frame = engine.call_stack.pop().expect("call_stack non-empty");
            engine.current_contract = frame.contract;
            engine.caller = frame.caller;
            engine.call_data = frame.call_data;
            engine.value_transferred = frame.value;

            match result {
                Ok(()) => {
                    if let Some((flags, _)) = &engine.return_value {
                        if flags.contains(ReturnFlags::REVERT) {
                            return Err(ReturnErrorCode::CalleeReverted);
                        }
                    }
                    Ok(())
                }
                Err(panic_payload) => {
                    if let Some(panic) = panic_payload.downcast_ref::<ReturnValuePanic>() {
                        engine.return_value = Some((panic.0.clone(), panic.1.clone()));
                        if panic.0.contains(ReturnFlags::REVERT) {
                            return Err(ReturnErrorCode::CalleeReverted);
                        }
                        return Ok(());
                    }
                    panic::resume_unwind(panic_payload);
                }
            }
        })
    }

    /// Off-chain: native transfer not tracked; no-op for tests.
    fn transfer(&self, _to: &Address, _value: &U256) -> CallResult {
        Ok(())
    }

    fn origin(&self) -> [u8; 20] {
        *self.caller().as_ref()
    }

    fn code_hash(&self, _addr: &[u8; 20]) -> H256 {
        H256::zero()
    }

    fn code_size(&self, _addr: &[u8; 20]) -> u64 {
        0
    }

    fn delegate_call(
        &self,
        _flags: CallFlags,
        address: &Address,
        _ref_time_limit: u64,
        _proof_size_limit: u64,
        _deposit_limit: &U256,
        input_data: &[u8],
        _output: Option<&mut &mut [u8]>,
    ) -> CallResult {
        ENGINE.with(|cell| {
            let mut engine = cell.borrow_mut();
            let callee_addr = *address.as_ref();
            let dispatcher = match engine.dispatchers.get(&callee_addr) {
                Some(d) => Arc::clone(d),
                None => return Err(ReturnErrorCode::CalleeTrapped),
            };

            let frame = CallFrame {
                contract: engine.current_contract,
                caller: engine.caller,
                call_data: std::mem::take(&mut engine.call_data),
                value: engine.value_transferred,
            };
            engine.current_contract = callee_addr;
            engine.call_data = input_data.to_vec();
            engine.contract_storages.entry(callee_addr).or_default();
            drop(engine);

            let result = panic::catch_unwind(AssertUnwindSafe(move || dispatcher()));

            let mut engine = cell.borrow_mut();
            engine.current_contract = frame.contract;
            engine.call_data = frame.call_data;

            match result {
                Ok(()) => {
                    if let Some((flags, _)) = &engine.return_value {
                        if flags.contains(ReturnFlags::REVERT) {
                            return Err(ReturnErrorCode::CalleeReverted);
                        }
                    }
                    Ok(())
                }
                Err(panic_payload) => {
                    if let Some(panic) = panic_payload.downcast_ref::<ReturnValuePanic>() {
                        engine.return_value = Some((panic.0.clone(), panic.1.clone()));
                        if panic.0.contains(ReturnFlags::REVERT) {
                            return Err(ReturnErrorCode::CalleeReverted);
                        }
                        return Ok(());
                    }
                    panic::resume_unwind(panic_payload);
                }
            }
        })
    }

    fn hash_keccak_256(&self, input: &[u8]) -> H256 {
        #[cfg(feature = "off_chain")]
        {
            let hash = Keccak256::digest(input);
            let mut output = [0u8; 32];
            output.copy_from_slice(&hash);
            H256::from(output)
        }
        #[cfg(not(feature = "off_chain"))]
        {
            H256::zero()
        }
    }

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
        ENGINE.with(|cell| {
            let mut engine = cell.borrow_mut();
            engine.next_address_counter = engine.next_address_counter.saturating_add(1);
            let id = engine.next_address_counter;
            address[16..20].copy_from_slice(&id.to_be_bytes());
            Ok(())
        })
    }

    fn now(&self) -> BlockNumber {
        1
    }

    fn gas_limit(&self) -> u64 {
        u64::MAX
    }

    fn set_storage_or_clear(&mut self, flags: StorageFlags, key: &[u8; 32], value: &[u8; 32]) -> Option<u32> {
        if value.iter().all(|&b| b == 0) {
            ENGINE.with(|cell| {
                let mut engine = cell.borrow_mut();
                let storage = engine.storage_mut();
                let key_vec = key.to_vec();
                let prev = storage.get(&key_vec).map(|v| v.len() as u32);
                storage.remove(&key_vec);
                prev
            })
        } else {
            self.set_storage(flags, key as &[u8], value)
        }
    }

    fn get_storage_or_zero(&mut self, flags: StorageFlags, key: &[u8; 32]) -> [u8; 32] {
        self.get_storage::<[u8; 32]>(flags, key as &[u8])
            .unwrap_or([0u8; 32])
    }

    fn value_transferred(&self) -> U256 {
        ENGINE.with(|cell| cell.borrow().value_transferred)
    }

    fn return_data_size(&self) -> u64 {
        ENGINE.with(|cell| {
            cell.borrow()
                .return_value
                .as_ref()
                .map(|(_, d)| d.len() as u64)
                .unwrap_or(0)
        })
    }

    fn return_data_copy(&self, output: &mut &mut [u8], offset: u32) {
        ENGINE.with(|cell| {
            if let Some((_, data)) = &cell.borrow().return_value {
                let off = offset as usize;
                let len = output.len().min(data.len().saturating_sub(off));
                if len > 0 {
                    output[..len].copy_from_slice(&data[off..off + len]);
                }
            }
        });
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
pub static mut OFF_CHAIN_ENV: OffChainEnv = OffChainEnv;
