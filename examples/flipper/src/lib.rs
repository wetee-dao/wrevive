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
mod flipper {
    use super::*;

    #[revive(constructor)]
    pub fn deploy() {
        // 默认 false，与 ink Flipper::new(false) 一致
        let v: u8 = 0;
        env().set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &[v]);
    }

    #[revive(message)]
    pub fn flip() {
        let mut buf = [0u8; 1];
        let mut slice: &mut [u8] = &mut buf[..];
        let current = if env().get_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &mut slice).is_ok() && slice.len() == 1 {
            buf[0] != 0
        } else {
            false
        };
        let new_val: u8 = if current { 0 } else { 1 };
        env().set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &[new_val]);
    }

    #[revive(message)]
    pub fn get() -> bool {
        let mut buf = [0u8; 1];
        let mut slice: &mut [u8] = &mut buf[..];
        if env().get_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &mut slice).is_ok() && slice.len() == 1 {
            buf[0] != 0
        } else {
            false
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
