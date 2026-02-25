#!/usr/bin/env bash
# 使用 host target + std 运行测试（合约编译仍使用 riscv64emac-unknown-none-polkavm）
set -e
export CARGO_BUILD_TARGET=$(rustc -vV | sed -n 's/.*host: *//p')
cargo test --features test "$@"
