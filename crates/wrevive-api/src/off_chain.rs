//! # Off-chain backend
//!
//! In-memory `Engine` for unit tests; see [ink! engine](https://github.com/use-ink/ink/tree/master/crates/engine) test_api.
//!
//! Off-chain 实现：内存 Engine，用于单元测试（参考 [ink engine](https://github.com/use-ink/ink/tree/master/crates/engine) test_api）。

use pallet_revive_uapi::ReturnFlags;
use std::cell::RefCell;
use std::collections::HashMap;

/// Thread-local engine instance; all ext::* calls and test setup use this.
/// 线程局部引擎实例；所有 ext::* 调用与测试配置均使用此实例。
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

/// Same namespace name as on_chain::ext; delegates to the thread-local Engine.
/// 与 on_chain 同名的 ext 命名空间，内部委托给 thread_local Engine。
pub mod ext {
    use super::ENGINE;
    use pallet_revive_uapi::{ReturnErrorCode, ReturnFlags, StorageFlags};

    /// Read caller from Engine (set via Engine::set_caller in tests).
    pub fn caller(output: &mut [u8; 20]) {
        ENGINE.with(|cell| output.copy_from_slice(&cell.borrow().caller));
    }

    /// Write to Engine storage; returns previous value length if any.
    pub fn set_storage(_flags: StorageFlags, key: &[u8], value: &[u8]) -> Option<u32> {
        ENGINE.with(|cell| {
            let prev = cell.borrow().storage.get(key).map(|v| v.len() as u32);
            cell.borrow_mut().storage.insert(key.to_vec(), value.to_vec());
            prev
        })
    }

    /// Read from Engine storage; Err(KeyNotFound) if key missing.
    pub fn get_storage(
        _flags: StorageFlags,
        key: &[u8],
        output: &mut &mut [u8],
    ) -> Result<(), ReturnErrorCode> {
        ENGINE.with(|cell| {
            let data = cell.borrow().storage.get(key).cloned();
            if let Some(d) = data {
                let len = d.len().min(output.len());
                output[..len].copy_from_slice(&d[..len]);
                Ok(())
            } else {
                Err(ReturnErrorCode::KeyNotFound)
            }
        })
    }

    /// Append (topics, data) to Engine.events.
    pub fn deposit_event(topics: &[[u8; 32]], data: &[u8]) {
        ENGINE.with(|cell| {
            cell.borrow_mut()
                .events
                .push((topics.to_vec(), data.to_vec()));
        });
    }

    /// Store return in Engine and panic (contract “returns” by unwinding in tests).
    pub fn return_value(flags: ReturnFlags, return_value: &[u8]) -> ! {
        ENGINE.with(|cell| {
            cell.borrow_mut().return_value = Some((flags, return_value.to_vec()));
        });
        panic!("off_chain return_value: flags={:?}, len={}", flags, return_value.len());
    }

    /// Length of Engine.call_data (set via set_call_data).
    pub fn call_data_size() -> u64 {
        ENGINE.with(|cell| cell.borrow().call_data.len() as u64)
    }

    /// Copy Engine.call_data from offset into output; pad with zeros if needed.
    pub fn call_data_copy(output: &mut [u8], offset: u32) {
        ENGINE.with(|cell| {
            let data = &cell.borrow().call_data;
            let off = offset as usize;
            let len = output.len().min(data.len().saturating_sub(off));
            if len > 0 {
                output[..len].copy_from_slice(&data[off..off + len]);
            }
            if len < output.len() {
                output[len..].fill(0);
            }
        });
    }
}
