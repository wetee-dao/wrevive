#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

wrevive_api::picoalloc_global_allocator!(1024);

use wrevive_api::{env, ReturnFlags, StorageFlags};

#[cfg(not(test))]
use wrevive_api::{input, HostFn};

#[allow(unused_imports)]
use wrevive_macro::{revive, revive_contract};

fn token_key(id: u32) -> [u8; 6] {
    let mut k = [0u8; 6];
    k[0..2].copy_from_slice(b"to");
    k[2..6].copy_from_slice(&id.to_le_bytes());
    k
}

fn count_key(owner: &[u8; 20]) -> [u8; 21] {
    let mut k = [0u8; 21];
    k[0] = b'c';
    k[1..21].copy_from_slice(owner);
    k
}

#[revive_contract]
mod erc721 {
    use super::*;

    fn read_owner(id: u32) -> [u8; 20] {
        let k = token_key(id);
        let mut buf = [0u8; 20];
        let mut slice: &mut [u8] = &mut buf[..];
        if env().get_storage(StorageFlags::empty(), &k, &mut slice).is_ok() && slice.len() == 20 {
            buf
        } else {
            [0u8; 20]
        }
    }

    fn write_owner(id: u32, owner: &[u8; 20]) {
        let k = token_key(id);
        env().set_storage(StorageFlags::empty(), &k, owner);
    }

    fn read_count(owner: &[u8; 20]) -> u32 {
        let k = count_key(owner);
        let mut buf = [0u8; 4];
        let mut slice: &mut [u8] = &mut buf[..];
        if env().get_storage(StorageFlags::empty(), &k, &mut slice).is_ok() && slice.len() == 4 {
            u32::from_le_bytes(buf)
        } else {
            0
        }
    }

    fn write_count(owner: &[u8; 20], n: u32) {
        let k = count_key(owner);
        env().set_storage(StorageFlags::empty(), &k, &n.to_le_bytes());
    }

    fn zero_addr(addr: &[u8; 20]) -> bool {
        addr.iter().all(|&b| b == 0)
    }

    #[revive(constructor)]
    pub fn deploy() {}

    #[revive(message)]
    pub fn mint(id: u32) {
        let owner = read_owner(id);
        if !zero_addr(&owner) {
            env().return_value(ReturnFlags::REVERT, &[]);
        }
        let caller = env().caller();
        write_owner(id, &caller);
        let c = read_count(&caller);
        write_count(&caller, c + 1);
    }

    #[revive(message)]
    pub fn balance_of(owner: [u8; 20]) -> u32 {
        read_count(&owner)
    }

    #[revive(message)]
    pub fn owner_of(id: u32) -> [u8; 20] {
        read_owner(id)
    }

    #[revive(message)]
    pub fn transfer(from: [u8; 20], to: [u8; 20], id: u32) {
        let caller = env().caller();
        if caller != from {
            env().return_value(ReturnFlags::REVERT, &[]);
        }
        let current = read_owner(id);
        if current != from {
            env().return_value(ReturnFlags::REVERT, &[]);
        }
        write_owner(id, &to);
        write_count(&from, read_count(&from).saturating_sub(1));
        write_count(&to, read_count(&to).saturating_add(1));
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
