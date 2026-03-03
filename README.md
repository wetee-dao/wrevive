# wrevive — Rust contract toolkit for `pallet-revive` (PolkaVM)

[中文文档](README.zh-CN.md)

> **Status: Work in progress. Not production-ready.**  
> APIs, ABIs, and behaviors may change without notice. Use at your own risk.

This repository is a **Cargo workspace** that includes:

- **`wrevive-api`**: contract runtime API with a unified `Env` (on-chain/off-chain) plus `Storage` / `Mapping` / `List` / `List2D`.
- **`wrevive-macro`**: ink!-style proc macros: `#[revive_contract]` + `#[revive(constructor)]` / `#[revive(message)]`, generating `deploy()` / `call()` dispatch and ABI.
- **`cargo-wrevive`**: `cargo wrevive build` subcommand to build PolkaVM `.polkavm` artifacts and emit ABI.
- **`examples`**:
  - `wrevive-contract`: a SCALE(codec) example contract using `wrevive-api` + `wrevive-macro` (recommended).

> Notes: commands below assume **Linux**. Run them at the repo root.

## Quick start

### Prerequisites

- **Rust toolchain (nightly recommended)**: `cargo wrevive build` may rely on unstable `-Z ...` flags.
  - If you must use stable, you may need `RUSTC_BOOTSTRAP=1` (not recommended for production).
- **`rust-src`** (required for build-std / cross builds)

```bash
rustup component add rust-src
```

### Install `cargo wrevive`

```bash
cargo install --path crates/cargo-wrevive
```

### Build the example contract (`.polkavm` + ABI)

```bash
cargo wrevive build -p wrevive-contract
```

Outputs (under workspace `target/`):

- **PolkaVM bytecode**: `target/<bin>.release.polkavm`
- **ABI (JSON)**: `target/<bin>.release.abi.json` (emitted by `cargo-wrevive`)
- **ABI (ink! style)**: `target/contract/<contract_name>.json` (emitted at compile time by `#[revive_contract]`)

> Filenames depend on the bin name and contract name resolution. Check `target/` after building.

### Run unit tests (off-chain engine)

```bash
# wrevive-api tests (includes off_chain Env)
cargo test -p wrevive-api

# example contract tests (off_chain Engine)
cargo test -p wrevive-contract
```

## Workspace layout

```
wrevive/
├── Cargo.toml
├── crates/
│   ├── wrevive-api/
│   ├── wrevive-macro/
│   └── cargo-wrevive/
├── examples/
│   └── wrevive-contract/
├── COVERAGE.md
└── README.md
```

## Writing contracts (recommended: `wrevive-api` + `wrevive-macro`)

See `examples/wrevive-contract/src/contract.rs`:

- **Entrypoints**: put `#[revive_contract]` on `mod contract { ... }`
- **Constructor**: `#[revive(constructor)] pub fn deploy(...) -> ...`
- **Messages**: `#[revive(message)] pub fn foo(...) -> ...`
- **Storage helpers**:
  - `storage!(b"...")` → `Storage<T>`
  - `mapping!(b"...")` → `Mapping<K, V>`
  - `list!(b"...")` → `List<Idx, V>`
  - `list_2d!(b"...")` → `List2D<K1, Idx, V>`

> `storage!/mapping!/list!/list_2d!` prefixes are derived from Blake2s256 (first 4 bytes). `#[revive_contract]` checks for duplicate prefixes at compile time.

### Common types (`wrevive-api`)

`wrevive-api` provides SCALE-encodable types that are convenient for storage and messages:

| Type | Meaning | Encoding |
|------|---------|----------|
| `Address` | 20-byte address (EVM/account compatible) | 20 bytes |
| `H256` | 32-byte hash | 32 bytes |
| `U256` | 256-bit unsigned integer (big-endian, EVM compatible) | 32 bytes |
| `BlockNumber` | block height (type alias of `u32`) | `u32` |
| `Bytes` | variable bytes (`Vec<u8>` alias) | length prefix + bytes |

Examples: `Storage<Address>`, `Mapping<Address, U256>`, `Mapping<H256, Bytes>`. `Address`/`H256` can be converted to/from `[u8; 20]`/`[u8; 32]` via `From`/`Into`.

## Call convention (selector + SCALE args)

The generated `call()` does:

1. Read the first **4 bytes** of call data as selector (`u32::from_be_bytes`).
2. Decode `call_data[4..]` as **SCALE-encoded** arguments, in order.

So the payload is:

- `payload = selector(4 bytes) ++ SCALE(args...)`

Selector rules:

- With `#[revive(message, selector = 0x...)]`: use the provided 4-byte selector
- Otherwise: use **first 4 bytes of BLAKE2s256(function_name)** (ink!-compatible)

## References

- [pallet-revive source](https://github.com/paritytech/polkadot-sdk/tree/master/substrate/frame/revive)
- [pallet-revive-uapi docs](https://paritytech.github.io/polkadot-sdk/master/pallet_revive_uapi/)
- [PolkaVM](https://github.com/paritytech/polkavm)
