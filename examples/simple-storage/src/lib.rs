#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

wrevive_api::picoalloc_global_allocator!(1024);

use wrevive_api::{env, ReturnFlags, StorageFlags};

#[cfg(not(test))]
use wrevive_api::{input, HostFn};

#[allow(unused_imports)]
use wrevive_macro::{revive, revive_contract};

fn storage_key(k: u32) -> [u8; 5] {
    let mut key = [0u8; 5];
    key[0] = b'v';
    key[1..5].copy_from_slice(&k.to_le_bytes());
    key
}

#[revive_contract]
mod simple_storage {
    use super::*;

    #[revive(constructor)]
    pub fn deploy() {}

    #[revive(message)]
    pub fn set(key: u32, value: u32) {
        let sk = storage_key(key);
        env().set_storage(StorageFlags::empty(), &sk, &value.to_le_bytes());
    }

    #[revive(message)]
    pub fn get(key: u32) -> u32 {
        let sk = storage_key(key);
        let mut buf = [0u8; 4];
        let mut slice: &mut [u8] = &mut buf[..];
        if env().get_storage(StorageFlags::empty(), &sk, &mut slice).is_ok() && slice.len() == 4 {
            u32::from_le_bytes(buf)
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
