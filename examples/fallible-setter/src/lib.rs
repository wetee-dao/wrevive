#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

wrevive_api::picoalloc_global_allocator!(1024);

use wrevive_api::{env, ReturnFlags, StorageFlags};

#[cfg(not(test))]
use wrevive_api::{input, HostFn};

#[allow(unused_imports)]
use wrevive_macro::{revive, revive_contract};

const STORAGE_KEY_VALUE: &[u8] = b"value";

#[revive_contract]
mod fallible_setter {
    use super::*;

    #[revive(constructor)]
    pub fn deploy() {
        // 默认 0，与 ink FallibleSetter::new(0) 一致
        let v: u8 = 0;
        env().set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &[v]);
    }

    /// 设置 value；若 value == 当前值或 value > 100 则 revert（对应 ink Error::NoChange / TooLarge）
    #[revive(message)]
    pub fn try_set(value: u8) {
        let mut buf = [0u8; 1];
        let mut slice: &mut [u8] = &mut buf[..];
        let current = if env().get_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &mut slice).is_ok() && slice.len() == 1 {
            buf[0]
        } else {
            0
        };
        if current == value {
            env().return_value(ReturnFlags::REVERT, &[]);
        }
        if value > 100 {
            env().return_value(ReturnFlags::REVERT, &[]);
        }
        env().set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &[value]);
    }

    #[revive(message)]
    pub fn get() -> u8 {
        let mut buf = [0u8; 1];
        let mut slice: &mut [u8] = &mut buf[..];
        if env().get_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &mut slice).is_ok() && slice.len() == 1 {
            buf[0]
        } else {
            0
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("unimp");
        core::hint::unreachable_unchecked();
    }
}

#[cfg(test)]
mod tests;
