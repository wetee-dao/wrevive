#!/usr/bin/env bash
# 编译 PolkaVM 合约：外层用命令行 -Z，内层 cargo（PvmBuilder）读 config.polkavm
set -e
cd "$(dirname "$0")"
cfg=.cargo/config.toml
cfg_pvm=.cargo/config.polkavm.toml
cp "$cfg" "$cfg.bak"
cp "$cfg_pvm" "$cfg"
trap "mv -f \"$cfg.bak\" \"$cfg\"" EXIT
RUSTC_BOOTSTRAP=1 cargo build --release \
  --target riscv64emac-unknown-none-polkavm.json \
  -Z build-std=core,alloc -Z json-target-spec \
  "$@"
