#![cfg_attr(not(feature = "test"), no_std)]
#![cfg_attr(not(feature = "test"), no_main)]

use wrevive_api::{ext, ReturnFlags, StorageFlags,input, HostFn};
#[allow(unused_imports)]
use wrevive_macro::{revive, revive_contract};

const STORAGE_KEY_VALUE: &[u8] = b"value";
const STORAGE_KEY_OWNER: &[u8] = b"owner";
const EMPTY_TOPICS: &[[u8; 32]] = &[];

#[revive_contract]
mod contract {
    use super::*;

    #[revive(constructor)]
    pub fn deploy() {
        let mut caller = [0u8; 20];
        ext::caller(&mut caller);
        
        ext::set_storage(StorageFlags::empty(), STORAGE_KEY_OWNER, &caller);
        let default_value: u32 = 0;
        ext::set_storage(
            StorageFlags::empty(),
            STORAGE_KEY_VALUE,
            &default_value.to_le_bytes(),
        );
    }

    #[revive(message, selector = 0x60fe47b1)]
    pub fn set_value(_value: u32) {
        ext::set_storage(
            StorageFlags::empty(),
            STORAGE_KEY_VALUE,
            &_value.to_le_bytes(),
        );
        ext::deposit_event(EMPTY_TOPICS, &_value.to_le_bytes().as_slice());
    }

    #[revive(message, selector = 0x6d4ce633)]
    pub fn get_value() -> u32 {
        let mut value_bytes = [0u8; 4];
        let mut slice: &mut [u8] = &mut value_bytes[..];
        if ext::get_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &mut slice).is_ok()
            && slice.len() == 4
        {
            u32::from_le_bytes(value_bytes)
        } else {
            0
        }
    }

    #[revive(message, selector = 0x13af4035)]
    pub fn set_owner(new_owner: [u8; 20], _v: u32) {
        let mut caller = [0u8; 20];
        ext::caller(&mut caller);

        let current_owner = get_owner();
        if caller != current_owner {
            ext::return_value(ReturnFlags::REVERT, &[]);
        } else {
            ext::set_storage(StorageFlags::empty(), STORAGE_KEY_OWNER, &new_owner);
        }
    }

    #[revive(message, selector = 0x8f8f9f8f)]
    pub fn get_owner() -> [u8; 20] {
        let mut owner = [0u8; 20];
        let mut slice: &mut [u8] = &mut owner[..];
        if ext::get_storage(StorageFlags::empty(), STORAGE_KEY_OWNER, &mut slice).is_ok()
            && slice.len() == 20
        {
            owner
        } else {
            [0u8; 20]
        }
    }
}

#[cfg(not(feature = "test"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
	// Safety: The unimp instruction is guaranteed to trap
	unsafe {
		core::arch::asm!("unimp");
		core::hint::unreachable_unchecked();
	}
}

#[cfg(all(feature = "test", test))]
mod tests;
