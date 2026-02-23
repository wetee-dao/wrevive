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
mod trait_incrementer {
    use super::*;

    #[revive(constructor)]
    pub fn deploy() {
        let v: u64 = 0;
        env().set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &v.to_le_bytes());
    }

    #[revive(message)]
    pub fn inc_by(delta: u64) {
        let mut buf = [0u8; 8];
        let mut slice: &mut [u8] = &mut buf[..];
        let current = if env().get_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &mut slice).is_ok() && slice.len() == 8 {
            u64::from_le_bytes(buf)
        } else {
            0
        };
        let new_val = current.saturating_add(delta);
        env().set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &new_val.to_le_bytes());
    }

    /// 对应 trait Increment::inc，自增 1
    #[revive(message)]
    pub fn inc() {
        inc_by(1);
    }

    #[revive(message)]
    pub fn get() -> u64 {
        let mut buf = [0u8; 8];
        let mut slice: &mut [u8] = &mut buf[..];
        if env().get_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &mut slice).is_ok() && slice.len() == 8 {
            u64::from_le_bytes(buf)
        } else {
            0
        }
    }

    /// 对应 trait Reset::reset
    #[revive(message)]
    pub fn reset() {
        let v: u64 = 0;
        env().set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &v.to_le_bytes());
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
