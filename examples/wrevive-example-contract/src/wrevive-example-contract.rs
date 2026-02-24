//! PolkaVM 合约 bin 入口：对外暴露 deploy/call，实现由 lib 提供。
#![no_main]
#![no_std]

use wrevive_example_contract::{call as do_call, deploy as do_deploy};

#[polkavm_derive::polkavm_export]
pub extern "C" fn deploy() {
    do_deploy();
}

#[polkavm_derive::polkavm_export]
pub extern "C" fn call() {
    do_call();
}
