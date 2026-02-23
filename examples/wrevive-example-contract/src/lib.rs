#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

wrevive_macro::picoalloc_global_allocator!(1024);

use wrevive_api::{env, get_storage, set_storage, ReturnFlags, StorageFlags};

#[allow(unused_imports)]
use wrevive_macro::{revive, revive_contract};

#[revive_contract]
mod contract {
    use super::*;
    const STORAGE_KEY_VALUE: &[u8] = b"value";
    const STORAGE_KEY_OWNER: &[u8] = b"owner";
    const EMPTY_TOPICS: &[[u8; 32]] = &[];

    #[revive(constructor)]
    pub fn deploy() {
        let caller = env().caller();
        
        set_storage(StorageFlags::empty(), STORAGE_KEY_OWNER, &caller);
        let default_value: u32 = 0;
        set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &default_value);
    }

    #[revive(message, selector = 0x60fe47b1)]
    pub fn set_value(_value: u32) {
        set_storage(StorageFlags::empty(), STORAGE_KEY_VALUE, &_value);
        env().deposit_event(EMPTY_TOPICS, &_value.to_le_bytes().as_slice());
    }

    #[revive(message, selector = 0x6d4ce633)]
    pub fn get_value() -> u32 {
        get_storage::<_, u32>(StorageFlags::empty(), STORAGE_KEY_VALUE).unwrap_or(0)
    }

    #[revive(message, selector = 0x13af4035)]
    pub fn set_owner(new_owner: [u8; 20], _v: u32) {
        let caller = env().caller();

        let current_owner = get_owner();
        if caller != current_owner {
            env().return_value(ReturnFlags::REVERT, &[]);
        } else {
            set_storage(StorageFlags::empty(), STORAGE_KEY_OWNER, &new_owner);
        }
    }

    #[revive(message)]
    pub fn get_owner() -> [u8; 20] {
        get_storage::<_, [u8; 20]>(StorageFlags::empty(), STORAGE_KEY_OWNER).unwrap_or([0u8; 20])
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
	// Safety: The unimp instruction is guaranteed to trap
	unsafe {
		core::arch::asm!("unimp");
		core::hint::unreachable_unchecked();
	}
}

#[cfg(test)]
mod tests;

