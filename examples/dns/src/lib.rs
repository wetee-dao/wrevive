#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

wrevive_api::picoalloc_global_allocator!(1024);

use wrevive_api::{env, ReturnFlags, StorageFlags};

#[cfg(not(test))]
use wrevive_api::{input, HostFn};

#[allow(unused_imports)]
use wrevive_macro::{revive, revive_contract};

fn owner_key(name: &[u8; 32]) -> [u8; 33] {
    let mut k = [0u8; 33];
    k[0] = b'o';
    k[1..33].copy_from_slice(name);
    k
}

fn addr_key(name: &[u8; 32]) -> [u8; 33] {
    let mut k = [0u8; 33];
    k[0] = b'a';
    k[1..33].copy_from_slice(name);
    k
}

#[revive_contract]
mod dns {
    use super::*;

    fn read_owner(name: &[u8; 32]) -> [u8; 20] {
        let k = owner_key(name);
        let mut buf = [0u8; 20];
        let mut slice: &mut [u8] = &mut buf[..];
        if env().get_storage(StorageFlags::empty(), &k, &mut slice).is_ok() && slice.len() == 20 {
            buf
        } else {
            [0u8; 20]
        }
    }

    fn zero_20() -> [u8; 20] {
        [0u8; 20]
    }

    #[revive(constructor)]
    pub fn deploy() {}

    /// 注册域名 name，caller 成为 owner；若 name 已被注册则 revert
    #[revive(message)]
    pub fn register(name: [u8; 32]) {
        let current = read_owner(&name);
        if current != zero_20() {
            env().return_value(ReturnFlags::REVERT, &[]);
        }
        let caller = env().caller();
        let k = owner_key(&name);
        env().set_storage(StorageFlags::empty(), &k, &caller);
        let ak = addr_key(&name);
        env().set_storage(StorageFlags::empty(), &ak, &caller);
    }

    /// 设置域名 name 的解析地址（仅 owner 可调）
    #[revive(message)]
    pub fn set_address(name: [u8; 32], new_address: [u8; 20]) {
        let caller = env().caller();
        let owner = read_owner(&name);
        if owner != caller {
            env().return_value(ReturnFlags::REVERT, &[]);
        }
        let k = addr_key(&name);
        env().set_storage(StorageFlags::empty(), &k, &new_address);
    }

    #[revive(message)]
    pub fn get_address(name: [u8; 32]) -> [u8; 20] {
        let k = addr_key(&name);
        let mut buf = [0u8; 20];
        let mut slice: &mut [u8] = &mut buf[..];
        if env().get_storage(StorageFlags::empty(), &k, &mut slice).is_ok() && slice.len() == 20 {
            buf
        } else {
            [0u8; 20]
        }
    }

    #[revive(message)]
    pub fn get_owner(name: [u8; 32]) -> [u8; 20] {
        read_owner(&name)
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
