//! # wrevive-macro
//!
//! # wrevive-macro (English)
//!
//! ink!-style procedural macros for PolkaVM (pallet-revive) contracts:
//! `#[revive(constructor)]` and `#[revive(message)]` / `#[revive(message, selector = 0x...)]`.
//!
//! 类似 ink! 的 `#[revive(constructor)]` / `#[revive(message)]` 宏，用于 PolkaVM（pallet-revive）合约。
//!
//! ## Usage / 用法
//!
//! Apply `#[revive_contract]` on a module. The module must contain:
//! 在模块上标注 `#[revive_contract]`，模块内包含：
//!
//! - **Exactly one** `#[revive(constructor)] fn deploy() { ... }` — called when the contract is instantiated.
//!   **恰好一个** `#[revive(constructor)] fn deploy() { ... }`：合约部署时调用的构造函数；
//!
//! - **Zero or more** `#[revive(message)]` or `#[revive(message, selector = 0x...)] fn name(args) -> ret { ... }` —
//!   externally callable messages; if no selector is given, the first 4 bytes of BLAKE2s(function_name) are used (ink!-compatible).
//!   **若干** `#[revive(message)]` 或 `#[revive(message, selector = 0x...)] fn name(args) -> ret { ... }`：
//!   对外可调用的 message；未提供 selector 时自动用 BLAKE2s(函数名) 前 4 字节（与 ink! 一致）。
//!
//! ## Generated code / 宏生成内容
//!
//! - **deploy()** — `extern "C"` entry called by the chain on instantiation; forwards to the user's `deploy()`.
//!   **deploy()**：`extern "C"` 入口，供链上实例化时调用，内部转发到用户的 `deploy()`；
//!
//! - **call()** — `extern "C"` entry that reads the first 4 bytes of call data as selector and dispatches to the matching message.
//!   **call()**：`extern "C"` 入口，读取 call data 前 4 字节作为 selector，按 selector 分发到对应 message；
//!
//! - **ABI file** — written at compile time to `target/contract/abi.json` in ink!-style format for frontend/JS encoding and decoding.
//!   **ABI 文件**：在编译时生成到 `target/contract/abi.json`，ink! 风格，供前端/JS 编码与解码。

extern crate proc_macro;

use blake2::Blake2s256;
use digest::Digest;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::env;
use std::fs;
use std::path::Path;
use syn::{
    parse_macro_input,
    punctuated::Punctuated,
    Attribute, FnArg, Item, ItemFn, Lit, Meta, ReturnType, Token,
};

// =============================================================================
// Attribute parsing and type mapping (for ABI and codegen)
// 属性解析与类型映射（供 ABI 与代码生成使用）
// =============================================================================

/// Computes the 4-byte message selector from the function name (ink!-compatible):
/// first 4 bytes of BLAKE2s256(name).
/// 从函数名生成 selector（与 ink! 一致）：BLAKE2s(函数名)，取前 4 字节。
fn selector_from_name(name: &str) -> [u8; 4] {
    let hash = Blake2s256::digest(name.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]]
}

/// Parses the 4-byte selector from `#[revive(message, selector = 0x...)]`.
/// E.g. `selector = 0x60fe47b1` → `[0x60, 0xfe, 0x47, 0xb1]` (big-endian).
/// Returns `None` if no `revive(..., selector = ...)` is present.
/// 从函数的 `#[revive(message, selector = 0x...)]` 属性中解析出 4 字节 selector。
/// 例如 `selector = 0x60fe47b1` 解析为 `[0x60, 0xfe, 0x47, 0xb1]`（大端序）。
/// 若没有 `revive(..., selector = ...)` 则返回 `None`。
fn parse_selector_from_attrs(attrs: &[Attribute]) -> Option<[u8; 4]> {
    for attr in attrs {
        if !attr.path().is_ident("revive") {
            continue;
        }
        let syn::Meta::List(list) = &attr.meta else { continue };
        let nested = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).ok()?;
        for meta in nested {
            let Meta::NameValue(nv) = meta else { continue };
            if !nv.path.is_ident("selector") {
                continue;
            }
            let syn::Expr::Lit(expr_lit) = &nv.value else { continue };
            let Lit::Int(lit) = &expr_lit.lit else { continue };
            let v: u32 = lit.base10_parse().ok()?;
            return Some(v.to_be_bytes());
        }
    }
    None
}

/// Returns true if the function is marked with `#[revive(constructor)]` (contract constructor).
/// 判断函数是否被标记为 `#[revive(constructor)]`（合约构造函数）。
fn is_revive_constructor(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("revive") {
            continue;
        }
        let syn::Meta::List(list) = &attr.meta else { continue };
        if list.tokens.to_string().contains("constructor") {
            return true;
        }
    }
    false
}

/// Returns true if the function is marked with `#[revive(message)]` or `#[revive(message, selector = ...)]`.
/// 判断函数是否被标记为 `#[revive(message)]` 或 `#[revive(message, selector = ...)]`。
fn is_revive_message(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("revive") {
            continue;
        }
        let syn::Meta::List(list) = &attr.meta else { continue };
        if list.tokens.to_string().contains("message") {
            return true;
        }
    }
    false
}

/// Removes all `#[revive(...)]` attributes so they are not processed again by other macros or the compiler.
/// After parsing, constructor and message functions no longer keep revive attributes.
/// 去掉所有 `#[revive(...)]` 属性，避免这些属性被下游宏或编译器再次处理。
/// 解析完成后，构造函数和 message 函数上不再保留 revive 属性。
fn strip_revive_attrs(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|a| !a.path().is_ident("revive"))
        .cloned()
        .collect()
}

/// Maps a Rust type to an ABI type name and optional length for abi.json.
/// - `u32` → `("u32", None)`
/// - `[u8; 20]` → `("AccountId", Some(20))`
/// 将 Rust 类型映射为 ABI 中的类型名及可选长度。
/// 用于生成 abi.json 里 message 的 args/returnType。
fn type_to_abi(ty: &syn::Type) -> Option<(String, Option<u32>)> {
    if let syn::Type::Path(p) = ty {
        if p.path.is_ident("u32") {
            return Some(("u32".into(), None));
        }
    }
    if let syn::Type::Array(arr) = ty {
        if let syn::Type::Path(inner) = *arr.elem.clone() {
            if inner.path.is_ident("u8") {
                if let syn::Expr::Lit(lit) = &arr.len {
                    if let Lit::Int(n) = &lit.lit {
                        if let Ok(len) = n.base10_parse::<u32>() {
                            return Some(("AccountId".into(), Some(len)));
                        }
                    }
                }
            }
        }
    }
    None
}

/// Formats the 4-byte selector as a hex string, e.g. `"0x60fe47b1"`, for the ABI selector field.
/// 将 4 字节 selector 格式化为十六进制字符串，如 `"0x60fe47b1"`，用于 ABI 的 selector 字段。
fn selector_hex(sel: [u8; 4]) -> String {
    format!("0x{:02x}{:02x}{:02x}{:02x}", sel[0], sel[1], sel[2], sel[3])
}

/// Generates ink!-style ABI JSON from contract name, constructor name, and message list.
/// Writes to `{CARGO_TARGET_DIR}/contract/{contract_name}.json` at compile time (used by frontend/JS).
/// 根据合约名、构造函数名与 message 列表，生成 ink! 风格的 ABI JSON，
/// 写入 `{CARGO_TARGET_DIR}/contract/{contract_name}.json`（编译时由宏调用，供前端/JS 使用）。
fn emit_abi(
    contract_name: &str,
    constructor_name: &str,
    message_fns: &[(ItemFn, [u8; 4])],
) {
    // Resolve output dir: CARGO_TARGET_DIR first, else {CARGO_MANIFEST_DIR}/target
    // 确定输出目录：优先 CARGO_TARGET_DIR，否则 {CARGO_MANIFEST_DIR}/target
    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(d) => d,
        Err(_) => return,
    };
    let target_dir = env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| Path::new(&manifest_dir).join("target").to_string_lossy().into_owned());
    let contract_dir = Path::new(&target_dir).join("contract");
    if fs::create_dir_all(&contract_dir).is_err() {
        return;
    }
    let out_path = contract_dir.join(format!("{}.json", contract_name));

    // Constructor is always "deploy" in ABI with selector 0x00000000
    // 构造函数在 ABI 中固定为 deploy，selector 0x00000000
    let constructors = vec![serde_json::json!({
        "label": constructor_name,
        "selector": "0x00000000",
        "payable": false,
        "args": [],
        "returnType": serde_json::Value::Null,
        "docs": [],
        "default": false
    })];

    // One ABI message entry per #[revive(message, selector = ...)]
    // 为每个 #[revive(message, selector = ...)] 生成一条 message 条目
    let mut messages = Vec::new();
    for (f, sel) in message_fns {
        let label = f.sig.ident.to_string();
        let selector = selector_hex(*sel);
        let mut args = Vec::new();
        // Build ABI args from function parameters (label + type/length)
        // 从函数参数构建 ABI args（label + type/length）
        for arg in &f.sig.inputs {
            let FnArg::Typed(pt) = arg else { continue };
            let arg_name = match pt.pat.as_ref() {
                syn::Pat::Ident(pi) => pi.ident.to_string(),
                _ => continue,
            };
            if let Some((ty_name, len)) = type_to_abi(pt.ty.as_ref()) {
                let mut arg_obj = serde_json::json!({ "label": arg_name, "type": { "type": 0, "displayName": [ty_name] } });
                if let Some(l) = len {
                    arg_obj["length"] = serde_json::json!(l);
                }
                args.push(arg_obj);
            }
        }
        // Return type: no return → Null; u32 / [u8;20] → displayName + optional length
        // 返回类型：无返回值 -> Null；u32 / [u8;20] -> displayName + 可选 length
        let (return_type, length) = match &f.sig.output {
            ReturnType::Default => (serde_json::Value::Null, None),
            ReturnType::Type(_, ty) => type_to_abi(ty)
                .map(|(name, len)| {
                    (
                        serde_json::json!({ "type": 0, "displayName": [name] }),
                        len,
                    )
                })
                .unwrap_or((serde_json::Value::Null, None)),
        };
        let mut msg = serde_json::json!({
            "label": label,
            "selector": selector,
            "mutates": true,
            "payable": false,
            "args": args,
            "returnType": return_type,
            "docs": [],
            "default": false
        });
        if let Some(l) = length {
            msg["length"] = serde_json::json!(l);
        }
        messages.push(msg);
    }

    let abi = serde_json::json!({
        "metadataVersion": "0.1",
        "contract": {
            "name": contract_name
        },
        "spec": {
            "constructors": constructors,
            "messages": messages,
            "events": [],
            "docs": []
        }
    });

    let _ = fs::write(&out_path, serde_json::to_string_pretty(&abi).unwrap_or_default());
}

// =============================================================================
// Call data layout and decode/encode (aligned with ABI)
// call() 内参数字节布局与解码/编码（与 ABI 约定一致）
// =============================================================================
// Layout: [ selector(4) | arg1 | arg2 | ... ]
// 布局： [ selector(4) | arg1 | arg2 | ... ]
// - u32: 4 bytes, little-endian, i.e. __input[4..8]
// - [u8; 20] (AccountId): 20 bytes, i.e. __input[4..24]

/// How to parse a single argument in manual_parse (cfg(test)).
/// manual_parse 时每个参数的解析方式。
#[derive(Clone, Copy)]
enum ParseKind {
    U8,
    U32,
    I32,
    Bool,
    U64,
    AccountId20,
    H256, // [u8; 32]
}

/// Builds (byte size, parse kind, type token for input!, call expression) for each message argument.
/// For [u8; 20], input! parses as &[u8; 20]; at call site we use *name to get [u8; 20].
/// 为官方 input! 生成参数：返回 (字节长, 解析方式, input! 中的类型 token, 调用 message 时的表达式)。
fn arg_for_input(
    arg_ty: &syn::Type,
    pat: &syn::Pat,
) -> Option<(usize, ParseKind, TokenStream2, TokenStream2)> {
    let name = match pat {
        syn::Pat::Ident(pi) => pi.ident.clone(),
        _ => return None,
    };
    if let syn::Type::Path(p) = arg_ty {
        if p.path.is_ident("u8") {
            return Some((1, ParseKind::U8, quote! { u8 }, quote! { #name }));
        }
        if p.path.is_ident("u32") {
            return Some((4, ParseKind::U32, quote! { u32 }, quote! { #name }));
        }
        if p.path.is_ident("i32") {
            return Some((4, ParseKind::I32, quote! { i32 }, quote! { #name }));
        }
        if p.path.is_ident("bool") {
            return Some((1, ParseKind::Bool, quote! { bool }, quote! { #name }));
        }
        if p.path.is_ident("u64") {
            return Some((8, ParseKind::U64, quote! { u64 }, quote! { #name }));
        }
    }
    if let syn::Type::Reference(r) = arg_ty {
        if let syn::Type::Array(arr) = r.elem.as_ref() {
            if let syn::Type::Path(inner) = *arr.elem.clone() {
                if inner.path.is_ident("u8") {
                    if let syn::Expr::Lit(lit) = &arr.len {
                        if let Lit::Int(n) = &lit.lit {
                            let len = n.base10_parse::<usize>().ok()?;
                            if len == 20 {
                                return Some((20, ParseKind::AccountId20, quote! { &[u8; 20] }, quote! { #name }));
                            }
                            if len == 32 {
                                return Some((32, ParseKind::H256, quote! { &[u8; 32] }, quote! { &#name }));
                            }
                        }
                    }
                }
            }
        }
    }
    if let syn::Type::Array(arr) = arg_ty {
        if let syn::Type::Path(inner) = *arr.elem.clone() {
            if inner.path.is_ident("u8") {
                if let syn::Expr::Lit(lit) = &arr.len {
                    if let Lit::Int(n) = &lit.lit {
                        let len = n.base10_parse::<usize>().ok()?;
                        if len == 20 {
                            return Some((20, ParseKind::AccountId20, quote! { &[u8; 20] }, quote! { *#name }));
                        }
                        if len == 32 {
                            return Some((32, ParseKind::H256, quote! { [u8; 32] }, quote! { #name }));
                        }
                    }
                }
            }
        }
    }
    None
}

/// When cfg(test): parses arguments from __input[__off..] by hand, without input!
/// (so we don't rely on HostFnImpl, which is not available on host during tests.)
/// 当 cfg(test)（cargo test）时，从 __input[__off..] 手动解析参数，不依赖 input!（避免 HostFnImpl 在 host 上不可用）。
fn manual_parse_from_input(
    input_vars: &[(syn::Ident, usize, ParseKind, TokenStream2)],
) -> TokenStream2 {
    let mut stmts = Vec::<TokenStream2>::new();
    let mut off: usize = 4; // after 4-byte selector
    for (name, size, kind, _call_expr) in input_vars {
        let stmt = match kind {
            ParseKind::U8 => quote! {
                let #name = __input[#off];
            },
            ParseKind::U32 => quote! {
                let #name = u32::from_le_bytes(__input[#off..#off + 4].try_into().unwrap());
            },
            ParseKind::I32 => quote! {
                let #name = i32::from_le_bytes(__input[#off..#off + 4].try_into().unwrap());
            },
            ParseKind::Bool => quote! {
                let #name = __input[#off] != 0;
            },
            ParseKind::U64 => quote! {
                let #name = u64::from_le_bytes(__input[#off..#off + 8].try_into().unwrap());
            },
            ParseKind::AccountId20 => quote! {
                let mut #name = [0u8; 20];
                #name.copy_from_slice(&__input[#off..#off + 20]);
            },
            ParseKind::H256 => quote! {
                let mut #name = [0u8; 32];
                #name.copy_from_slice(&__input[#off..#off + 32]);
            },
        };
        stmts.push(stmt);
        off += size;
    }
    quote! {
        let mut __off = 4usize;
        #(#stmts)*
    }
}

/// Generates code that encodes the return value `__ret` and passes it to `wrevive_api::env().return_value`.
/// - `()`: no return, pass empty slice;
/// - `u32`: encode as 32-byte buffer (first 4 bytes LE), common ABI convention;
/// - `[u8; 20]`: pass 20-byte reference directly;
/// - other: currently treated as no return, empty slice (extensible later).
/// 根据 message 的返回类型，生成将返回值 `__ret` 编码并通过 `wrevive_api::env().return_value` 返回的代码。
/// - `()`：无返回值，传空切片；
/// - `u32`：编码为 32 字节 buffer（前 4 字节小端），与常见 ABI 约定一致；
/// - `[u8; 20]`：直接传 20 字节引用；
/// - 其他类型：当前视为无返回，传空切片（可后续扩展）。
fn return_encode(ret_ty: &ReturnType) -> TokenStream2 {
    match ret_ty {
        ReturnType::Default => quote! {
            wrevive_api::env().return_value(ReturnFlags::empty(), &[]);
        },
        ReturnType::Type(_, ty) => {
            if let syn::Type::Path(p) = ty.as_ref() {
                if p.path.is_ident("u32") {
                    return quote! {
                        let __buf: [u8; 32] = {
                            let mut b = [0u8; 32];
                            b[0..4].copy_from_slice(&__ret.to_le_bytes());
                            b
                        };
                        wrevive_api::env().return_value(ReturnFlags::empty(), &__buf);
                    };
                }
                if p.path.is_ident("i32") {
                    return quote! {
                        let __buf: [u8; 32] = {
                            let mut b = [0u8; 32];
                            b[0..4].copy_from_slice(&__ret.to_le_bytes());
                            b
                        };
                        wrevive_api::env().return_value(ReturnFlags::empty(), &__buf);
                    };
                }
                if p.path.is_ident("bool") {
                    return quote! {
                        let __byte = [if __ret { 1u8 } else { 0u8 }];
                        wrevive_api::env().return_value(ReturnFlags::empty(), &__byte);
                    };
                }
                if p.path.is_ident("u8") {
                    return quote! {
                        let __byte = [__ret];
                        wrevive_api::env().return_value(ReturnFlags::empty(), &__byte);
                    };
                }
                if p.path.is_ident("u64") {
                    return quote! {
                        let __buf: [u8; 32] = {
                            let mut b = [0u8; 32];
                            b[0..8].copy_from_slice(&__ret.to_le_bytes());
                            b
                        };
                        wrevive_api::env().return_value(ReturnFlags::empty(), &__buf);
                    };
                }
            }
            if let syn::Type::Array(arr) = ty.as_ref() {
                if let syn::Type::Path(inner) = *arr.elem.clone() {
                    if inner.path.is_ident("u8") {
                        if let syn::Expr::Lit(lit) = &arr.len {
                            if let Lit::Int(n) = &lit.lit {
                                if n.base10_parse::<usize>().ok() == Some(20) {
                                    return quote! {
                                        wrevive_api::env().return_value(ReturnFlags::empty(), &__ret);
                                    };
                                }
                                if n.base10_parse::<usize>().ok() == Some(32) {
                                    return quote! {
                                        wrevive_api::env().return_value(ReturnFlags::empty(), &__ret);
                                    };
                                }
                            }
                        }
                    }
                }
            }
            quote! {
                wrevive_api::env().return_value(ReturnFlags::empty(), &[]);
            }
        }
    }
}


// =============================================================================
// Procedural macro entry points
// 过程宏入口
// =============================================================================

/// **Main macro**: `#[revive_contract]` must be applied to a `mod`. It:
/// 1. Keeps the mod and its items;
/// 2. Emits `deploy()` and `call()` as `extern "C"` at crate root;
/// 3. Writes ABI to target/contract/abi.json at compile time.
/// **主宏**：`#[revive_contract]` 只能挂在 `mod` 上，展开后：
/// 1. 保留该 mod 及其内部项；
/// 2. 在 crate 根生成 `deploy()` 和 `call()` 两个 `extern "C"` 函数；
/// 3. 编译时把 ABI 写入 target/contract/abi.json。
#[proc_macro_attribute]
pub fn revive_contract(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let contract_name = env::var("CONTRACT_LIB_NAME").unwrap_or_else(|_| "contract".into());

    let item = parse_macro_input!(item as Item);
    let Item::Mod(mut module) = item else {
        return syn::Error::new_spanned(item, "revive_contract must be applied to a mod / revive_contract 只能用于 mod")
            .to_compile_error()
            .into();
    };

    let mod_name = module.ident.clone();
    let mod_content = match &mut module.content {
        Some((_, items)) => items,
        None => {
            return syn::Error::new_spanned(module, "revive_contract mod must have a body / revive_contract mod 必须有 body")
                .to_compile_error()
                .into();
        }
    };

    // Classify: exactly one constructor, messages with selectors, other items (kept as-is)
    // 分类：构造函数（恰好一个）、带 selector 的 message、其他项（原样保留）
    let mut constructor_fn: Option<ItemFn> = None;
    let mut message_fns: Vec<(ItemFn, [u8; 4])> = Vec::new();
    let mut other_items: Vec<Item> = Vec::new();

    for item in std::mem::take(mod_content) {
        match item {
            Item::Fn(mut f) => {
                let is_constructor = is_revive_constructor(&f.attrs);
                let selector = parse_selector_from_attrs(&f.attrs);
                if is_constructor {
                    f.attrs = strip_revive_attrs(&f.attrs);
                    constructor_fn = Some(f);
                } else if is_revive_message(&f.attrs) {
                    let sel = selector.unwrap_or_else(|| {
                        selector_from_name(&f.sig.ident.to_string())
                    });
                    f.attrs = strip_revive_attrs(&f.attrs);
                    message_fns.push((f, sel));
                } else {
                    other_items.push(Item::Fn(f));
                }
            }
            other => other_items.push(other),
        }
    }

    let constructor_fn = match constructor_fn {
        Some(f) => f,
        None => {
            return syn::Error::new_spanned(&module, "exactly one #[revive(constructor)] function required / 需要恰好一个 #[revive(constructor)] 函数")
                .to_compile_error()
                .into();
        }
    };
    let constructor_name = constructor_fn.sig.ident.clone();

    // Emit ABI to target/contract/{contract_name}.json
    // 生成 ABI 到 target/contract/abi.json
    emit_abi(
        &contract_name,
        &constructor_name.to_string(),
        &message_fns,
    );

    // Put constructor, messages, and other items back into the mod (deploy/call are emitted outside)
    // 把用户的 constructor、message、其他项重新放回 mod（deploy/call 生成在 mod 外）
    mod_content.push(Item::Fn(constructor_fn));
    for (f, _) in &message_fns {
        mod_content.push(Item::Fn(f.clone()));
    }
    mod_content.extend(other_items);

    // Build match arms for call(): cfg(test) chooses manual parse vs input! (set by cargo test)
    // 为 call() 生成 match 分支：cfg(test) 时用手动解析，否则用 input!（由 cargo test 自动设置）
    let match_arms: Vec<TokenStream2> = message_fns
        .iter()
        .map(|(f, sel)| {
            let fn_name = &f.sig.ident;
            let sig = &f.sig;
            let mut min_len: usize = 4;
            let mut input_vars_off: Vec<(syn::Ident, usize, ParseKind, TokenStream2)> = Vec::new();
            let mut input_vars_ink: Vec<(syn::Ident, TokenStream2)> = Vec::new();
            let mut call_exprs = Vec::new();
            for arg in &sig.inputs {
                let FnArg::Typed(pt) = arg else { continue };
                if let Some((size, parse_kind, type_tt, call_expr)) = arg_for_input(pt.ty.as_ref(), &pt.pat) {
                    min_len += size;
                    let name = match pt.pat.as_ref() {
                        syn::Pat::Ident(pi) => pi.ident.clone(),
                        _ => continue,
                    };
                    input_vars_off.push((name.clone(), size, parse_kind, call_expr.clone()));
                    input_vars_ink.push((name, type_tt));
                    call_exprs.push(call_expr);
                }
            }
            let manual_parse = manual_parse_from_input(&input_vars_off);
            // Manual parse: [u8; 20] is already by-value, pass name; with input! we pass *name for &[u8; 20]
            // 手动解析时 [u8; 20] 变量已是 by-value，传 name；input! 时为 &[u8; 20] 传 *name
            let call_exprs_manual: Vec<TokenStream2> = input_vars_off
                .iter()
                .map(|(name, _, _, _)| quote! { #name })
                .collect();
            let input_parse_ink = if input_vars_ink.is_empty() {
                quote! { input!(__input, _skip: u32, ); }
            } else {
                let names = input_vars_ink.iter().map(|(n, _)| n);
                let types = input_vars_ink.iter().map(|(_, t)| t);
                quote! { input!(__input, _skip: u32, #(#names: #types),* , ); }
            };
            let ret_ty = &sig.output;
            let encode_and_return = return_encode(ret_ty);
            let sel_u32 = u32::from_be_bytes(*sel);
            quote! {
                #sel_u32 => {
                    if __input_len >= #min_len {
                        #[cfg(test)]
                        #manual_parse
                        #[cfg(not(test))]
                        #input_parse_ink
                        #[cfg(test)]
                        let __ret = #mod_name::#fn_name(#(#call_exprs_manual),*);
                        #[cfg(not(test))]
                        let __ret = #mod_name::#fn_name(#(#call_exprs),*);
                        #encode_and_return
                    } else {
                        wrevive_api::env().return_value(ReturnFlags::REVERT, &[]);
                    }
                }
            }
        })
        .collect();

    // Emit deploy(): entry called by PolkaVM on instantiation, forwards to user constructor
    // 生成 deploy()：PolkaVM 实例化时调用的入口，直接转发到用户定义的构造函数
    let deploy_fn: Item = syn::parse2(quote! {
        #[no_mangle]
        pub extern "C" fn deploy() {
            #mod_name::#constructor_name();
        }
    })
    .unwrap();

    // Emit call(): read call data, dispatch by first 4-byte selector, encode return value
    // 生成 call()：读取 call data，按前 4 字节 selector 分发到对应 message，并编码返回值
    let call_fn: Item = syn::parse2(quote! {
        #[no_mangle]
        pub extern "C" fn call() {
            let __input_len = wrevive_api::env().call_data_size().min(1024) as usize;
            let __input_vec = if __input_len > 0 {
                wrevive_api::env().call_data_copy(0, __input_len)
            } else {
                #[cfg(test)]
                let empty = vec![];
                #[cfg(not(test))]
                let empty = alloc::vec![];
                empty
            };
            let __input: &[u8] = &__input_vec;
            if __input_len >= 4 {
                let __sel = u32::from_be_bytes([__input[0], __input[1], __input[2], __input[3]]);
                match __sel {
                    #(#match_arms),*
                    _ => wrevive_api::env().return_value(ReturnFlags::REVERT, &[]),
                }
            } else {
                wrevive_api::env().return_value(ReturnFlags::REVERT, &[]);
            }
        }
    })
    .unwrap();

    // Expansion: original mod + deploy() + call()
    // 展开结果：原 mod + deploy() + call()
    quote! {
        #module

        #deploy_fn

        #call_fn
    }
    .into()
}

/// **Pass-through macro**: `#[revive(constructor)]` and `#[revive(message, selector = ...)]` are only markers;
/// `#[revive_contract]` reads them when parsing the mod; here we do not expand, just return the item as-is.
/// **透传宏**：`#[revive(constructor)]` 与 `#[revive(message, selector = ...)]` 仅作为标记，
/// 由 `#[revive_contract]` 在解析 mod 时读取，此处不做展开，原样返回 item。
#[proc_macro_attribute]
pub fn revive(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// **Scale derive**: like `#[ink::scale_derive(Encode, Decode, TypeInfo)]`, expands to
/// `#[derive(::wrevive_api::Encode, ::wrevive_api::Decode, ::wrevive_api::TypeInfo)]` on the item.
/// Use on structs/enums whose instances are stored as Mapping values (set/get require Scale).
#[proc_macro_attribute]
pub fn scale_derive(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: TokenStream2 = item.into();
    quote! {
        #[derive(::wrevive_api::Encode, ::wrevive_api::Decode, ::wrevive_api::TypeInfo)]
        #item
    }
    .into()
}
