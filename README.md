# wrevive — Rust contract toolkit wrapping `pallet-revive` (PolkaVM)

[中文文档](README.zh-CN.md)

> **Status: Work in progress. Not production-ready.**  
> APIs, ABIs, and behaviors may change without notice. Use at your own risk.

**wrevive** is a **wrapper around pallet-revive** that provides an **ink!-like development experience** for writing PolkaVM contracts. It abstracts away low-level pallet-revive UAPI calls while maintaining full compatibility.

## What is wrevive?

- **Purpose**: Wrap pallet-revive's low-level UAPI to provide a developer-friendly, ink!-like experience
- **Key Features**:
  - Declarative storage macros: `storage!`, `mapping!`, `list!`, `list_2d!`
  - Automatic entry point generation: `#[revive_contract]` generates `deploy()`/`call()`
  - Unified on-chain/off-chain testing: same `Env` trait for both environments
  - ABI generation: ink!-like JSON ABI for tooling integration
- **Compatibility**: Compiles to native pallet-revive contracts - no runtime overhead

This repository is a **Cargo workspace** that includes:

- **`wrevive-api`**: Contract runtime API with unified `Env` (on-chain/off-chain) and high-level storage abstractions (`Storage`/`Mapping`/`List`/`List2D`)
- **`wrevive-macro`**: ink!-style proc macros: `#[revive_contract]` + `#[revive(constructor)]`/`#[revive(message)]`, generating `deploy()`/`call()` dispatch and ABI
- **`cargo-wrevive`**: `cargo wrevive build` subcommand to build PolkaVM `.polkavm` artifacts and emit ABI
- **`examples`**:
  - `wrevive-contract`: A SCALE(codec) example contract using `wrevive-api` + `wrevive-macro` (recommended)

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

See `examples/wrevive-contract/src/contract.rs` for a complete example:

### Basic contract structure

```rust
#[revive_contract]
pub mod contract {
    use wrevive_api::Address;
    use wrevive_macro::{storage, mapping, list, list_2d, revive_contract};
    
    // Storage definitions
    const VALUE: Storage<u32> = storage!(b"value");
    const BALANCE: Mapping<Address, u64> = mapping!(b"balance");
    const RECORDS: List<u32, u64> = list!(b"records");
    
    // Constructor (required, exactly one)
    #[revive(constructor)]
    pub fn deploy(initial_value: u32) -> Result<(), Error> {
        VALUE.set(&initial_value);
        Ok(())
    }
    
    // Message functions
    #[revive(message, write)]
    pub fn set_value(value: u32) -> Result<(), Error> {
        VALUE.set(&value);
        Ok(())
    }
    
    #[revive(message)]
    pub fn get_value() -> u32 {
        VALUE.get().unwrap_or(0)
    }
}
```

### Storage helpers

- **`storage!(b"...")`** → `Storage<T>`: Single value storage
- **`mapping!(b"...")`** → `Mapping<K, V>`: Key-value mapping storage
- **`list!(b"...")`** → `List<Idx, V>`: One-dimensional list with auto-increment IDs
- **`list_2d!(b"...")`** → `List2D<K1, Idx, V>`: Two-dimensional list (outer key + inner auto-increment)

> `storage!/mapping!/list!/list_2d!` prefixes are derived from Blake2s256 (first 4 bytes). `#[revive_contract]` checks for duplicate prefixes at compile time.

### Message attributes

- **`#[revive(message)]`**: Read-only message (pure view)
- **`#[revive(message, write)]`**: Mutates storage (required for functions that modify state)
- **`#[revive(message, sol)]`**: Use Solidity encoding instead of SCALE
- **`#[revive(message, selector = 0x12345678)]`**: Custom 4-byte selector
- **`#[revive(constructor)]`**: Contract constructor (exactly one required)
- **`#[revive(fallback)]`**: Fallback function for unknown selectors (optional, at most one)

### Return types

All contract functions (constructor and messages) must have a return value:
- **`Result<T, E>`**: For operations that may fail (recommended)
- **`T`**: For operations that always succeed
- **`Option<T>`**: For operations that may return no value

Example error types:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum Error {
    InsufficientBalance,
    Unauthorized,
    NotFound,
}
```

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
