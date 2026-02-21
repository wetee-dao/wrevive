//! Off-chain 实现：内存 Engine，用于单元测试（参考 [ink engine](https://github.com/use-ink/ink/tree/master/crates/engine) test_api）。

use pallet_revive_uapi::ReturnFlags;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static ENGINE: RefCell<Engine> = RefCell::new(Engine::default());
}

/// Off-chain 执行引擎：内存 storage、可配置 caller/call_data、记录 events。
#[derive(Default)]
pub struct Engine {
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
    pub call_data: Vec<u8>,
    pub caller: [u8; 20],
    pub events: Vec<(Vec<[u8; 32]>, Vec<u8>)>,
    pub return_value: Option<(ReturnFlags, Vec<u8>)>,
}

impl Engine {
    /// 设置下一次调用的 caller（同 ink engine set_caller）。
    pub fn set_caller(&mut self, addr: [u8; 20]) {
        self.caller = addr;
    }

    /// 设置当前调用的 call_data（供 call_data_size / call_data_copy 读取）。
    pub fn set_call_data(&mut self, data: &[u8]) {
        self.call_data = data.to_vec();
    }

    /// 读取某 key 的 storage 值（测试断言用）。
    pub fn get_storage_value(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.get(key).cloned()
    }

    /// 取出已记录的 events（测试断言用）。
    pub fn take_events(&mut self) -> Vec<(Vec<[u8; 32]>, Vec<u8>)> {
        std::mem::take(&mut self.events)
    }

    /// 重置引擎状态（同 ink engine initialize_or_reset）。
    pub fn reset(&mut self) {
        self.storage.clear();
        self.call_data.clear();
        self.events.clear();
        self.return_value = None;
        self.caller = [0u8; 20];
    }
}

/// 在 off_chain 上下文中运行闭包（配置 Engine 或做断言）。同 ink 里对 Engine 的配置。
pub fn with_engine<F, R>(f: F) -> R
where
    F: FnOnce(&mut Engine) -> R,
{
    ENGINE.with(|cell| f(&mut *cell.borrow_mut()))
}

/// 与 on_chain 同名的 ext 命名空间，内部委托给 thread_local Engine。
pub mod ext {
    use super::ENGINE;
    use pallet_revive_uapi::{ReturnErrorCode, ReturnFlags, StorageFlags};

    pub fn caller(output: &mut [u8; 20]) {
        ENGINE.with(|cell| output.copy_from_slice(&cell.borrow().caller));
    }

    pub fn set_storage(_flags: StorageFlags, key: &[u8], value: &[u8]) -> Option<u32> {
        ENGINE.with(|cell| {
            let prev = cell.borrow().storage.get(key).map(|v| v.len() as u32);
            cell.borrow_mut().storage.insert(key.to_vec(), value.to_vec());
            prev
        })
    }

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

    pub fn deposit_event(topics: &[[u8; 32]], data: &[u8]) {
        ENGINE.with(|cell| {
            cell.borrow_mut()
                .events
                .push((topics.to_vec(), data.to_vec()));
        });
    }

    pub fn return_value(flags: ReturnFlags, return_value: &[u8]) -> ! {
        ENGINE.with(|cell| {
            cell.borrow_mut().return_value = Some((flags, return_value.to_vec()));
        });
        panic!("off_chain return_value: flags={:?}, len={}", flags, return_value.len());
    }

    pub fn call_data_size() -> u64 {
        ENGINE.with(|cell| cell.borrow().call_data.len() as u64)
    }

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
