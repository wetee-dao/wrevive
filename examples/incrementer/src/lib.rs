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
mod incrementer {
    use super::*;

    #[revive(constructor)]
    pub fn deploy() {
        // 默认 0，与 ink Incrementer::new_default() 一致
        let v: i32 = 0;
        env().set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &v.to_le_bytes());
    }

    #[revive(message)]
    pub fn inc(by: i32) {
        let mut buf = [0u8; 4];
        let mut slice: &mut [u8] = &mut buf[..];
        let current = if env().get_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &mut slice).is_ok() && slice.len() == 4 {
            i32::from_le_bytes(buf)
        } else {
            0
        };
        let new_val = current.saturating_add(by);
        env().set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &new_val.to_le_bytes());
    }

    #[revive(message)]
    pub fn get() -> i32 {
        let mut buf = [0u8; 4];
        let mut slice: &mut [u8] = &mut buf[..];
        if env().get_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &mut slice).is_ok() && slice.len() == 4 {
            i32::from_le_bytes(buf)
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
