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
//! Apply `#[revive_contract]` or `#[revive_contract(encoding = "codec")]` / `#[revive_contract(encoding = "sol")]` on a module.
//! 在模块上标注 `#[revive_contract]` 或 `#[revive_contract(encoding = "codec")]` / `#[revive_contract(encoding = "sol")]`。
//!
//! **Encoding 模式 / Encoding mode:** 可在合约级或每个函数级指定。
//! - **codec**（默认）：参数与返回值用 SCALE（parity-scale-codec）编解码；与 ink! 一致。
//! - **sol**：参数用 `pvm_contract_types::SolDecode` 读取，返回值用 `SolEncode` 格式化（Solidity ABI）；合约需依赖 `pvm-contract-types`。
//! - 合约级：`#[revive_contract(encoding = "sol")]`；函数级：`#[revive(message, sol)]` / `#[revive(message, encoding = "sol")]` 或 `#[revive(constructor, codec)]`。函数级覆盖合约级默认。
//!
//! The module must contain:
//! 模块内包含：
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
use std::path::{Path, PathBuf};
use syn::{
    parse_macro_input,
    punctuated::Punctuated,
    Attribute, Expr, FnArg, GenericArgument, Item, ItemFn, Lit, Meta, PathArguments, ReturnType,
    Token,
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

/// Encoding mode for contract messages: codec (SCALE) or sol (Solidity ABI).
/// 合约 message 的编码模式：codec（SCALE）或 sol（Solidity ABI）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum EncodingMode {
    Codec,
    Sol,
}

/// Parses `encoding = "codec"` or `encoding = "sol"` from `#[revive_contract(...)]` attribute.
/// Default is Codec when absent or unrecognized.
fn parse_encoding_from_contract_attr(attr: &TokenStream) -> EncodingMode {
    use syn::parse::Parser;
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let attr2 = TokenStream2::from(attr.clone());
    let Ok(nested) = parser.parse2(attr2) else {
        return EncodingMode::Codec;
    };
    for meta in nested {
        let Meta::NameValue(nv) = meta else { continue };
        if !nv.path.is_ident("encoding") {
            continue;
        }
        let syn::Expr::Lit(expr_lit) = &nv.value else { continue };
        let Lit::Str(s) = &expr_lit.lit else { continue };
        if s.value() == "sol" {
            return EncodingMode::Sol;
        }
        if s.value() == "codec" {
            return EncodingMode::Codec;
        }
    }
    EncodingMode::Codec
}

/// Parses `encoding = "codec"` / `encoding = "sol"` or path `codec` / `sol` from `#[revive(message, ...)]` or `#[revive(constructor, ...)]`.
/// 从函数的 revive 属性中解析 encoding，未指定时使用 default。
fn parse_encoding_from_fn_attrs(attrs: &[Attribute], default: EncodingMode) -> EncodingMode {
    for attr in attrs {
        if !attr.path().is_ident("revive") {
            continue;
        }
        let syn::Meta::List(list) = &attr.meta else { continue };
        let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) else {
            continue;
        };
        for meta in nested {
            match &meta {
                Meta::NameValue(nv) if nv.path.is_ident("encoding") => {
                    let syn::Expr::Lit(expr_lit) = &nv.value else { continue };
                    let Lit::Str(s) = &expr_lit.lit else { continue };
                    if s.value() == "sol" {
                        return EncodingMode::Sol;
                    }
                    if s.value() == "codec" {
                        return EncodingMode::Codec;
                    }
                }
                Meta::Path(p) if p.is_ident("sol") => return EncodingMode::Sol,
                Meta::Path(p) if p.is_ident("codec") => return EncodingMode::Codec,
                _ => {}
            }
        }
    }
    default
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

/// 从 syn::Path 取类型显示名（多段路径取最后一段，如 `crate::BalanceInfo` → `BalanceInfo`）。
fn path_to_display_name(path: &syn::Path) -> String {
    path.segments
        .iter()
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_else(|| "ScaleBytes".into())
}

/// 若 expr 是 `X::new(b"prefix")` 或 `X::new(&b"prefix")`（X 为 Storage/Mapping 等），返回 Some(prefix 字节)；否则返回 None。
/// 也识别宏展开后的 `X::new(&[b0, b1, b2, b3])`（storage!/mapping! 生成的 4 字节 prefix）。
fn extract_prefix_from_new_call(expr: &Expr) -> Option<Vec<u8>> {
    let call = match expr {
        Expr::Call(c) => c,
        _ => return None,
    };
    let path = match &*call.func {
        Expr::Path(p) => &p.path,
        _ => return None,
    };
    if path.segments.last()?.ident != "new" {
        return None;
    }
    let first_arg = call.args.first()?;
    let inner = match first_arg {
        Expr::Reference(r) => &*r.expr,
        other => other,
    };
    // 字面量：b"prefix" 或 "prefix"（字节串）
    if let Expr::Lit(l) = inner {
        return match &l.lit {
            Lit::ByteStr(bs) => Some(bs.value()),
            Lit::Str(s) => Some(s.value().into_bytes()),
            _ => None,
        };
    }
    // 宏展开：&[b0, b1, b2, b3]（storage!/mapping! 生成的 4 字节）
    if let Expr::Array(arr) = inner {
        let mut bytes = Vec::with_capacity(arr.elems.len());
        for e in &arr.elems {
            if let Expr::Lit(l) = e {
                if let Lit::Byte(b) = &l.lit {
                    bytes.push(b.value());
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        return Some(bytes);
    }
    None
}

/// 若 expr 是未展开的 `storage!(...)` 或 `mapping!(...)`，用与宏相同的规则（Blake2s256 取前 4 字节）返回 prefix，否则 None。
/// 这样在 revive_contract 展开时（早于 storage!/mapping! 展开）也能做 prefix 重复检查。
fn extract_prefix_from_storage_mapping_macro(expr: &Expr) -> Option<Vec<u8>> {
    let mac = match expr {
        Expr::Macro(m) => m,
        _ => return None,
    };
    let seg = mac.mac.path.segments.last()?;
    let name = seg.ident.to_string();
    if name != "storage" && name != "mapping" {
        return None;
    }
    let lit: Lit = syn::parse2(mac.mac.tokens.clone()).ok()?;
    let bytes = lit_to_prefix_bytes(&lit)?;
    Some(blake2s_prefix_4_bytes(&bytes).to_vec())
}

/// 若 expr 是未展开的 `list!(...)`，返回 Blake2s256 取前 4 字节的两个 prefix：_id、_items。
fn extract_prefixes_from_list_macro(expr: &Expr) -> Option<Vec<Vec<u8>>> {
    let mac = match expr {
        Expr::Macro(m) => m,
        _ => return None,
    };
    let seg = mac.mac.path.segments.last()?;
    if seg.ident != "list" {
        return None;
    }
    let lit: Lit = syn::parse2(mac.mac.tokens.clone()).ok()?;
    let bytes = lit_to_prefix_bytes(&lit)?;
    let p_id = blake2s_prefix_4_bytes(&bytes).to_vec();
    let p_items = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_items"].concat()).to_vec();
    Some(vec![p_id, p_items])
}

/// 若 expr 是未展开的 `list_2d!(...)`，返回 Blake2s256 取前 4 字节的四个 prefix：_k1、_len、_k2、_store。
fn extract_prefixes_from_list_2d_macro(expr: &Expr) -> Option<Vec<Vec<u8>>> {
    let mac = match expr {
        Expr::Macro(m) => m,
        _ => return None,
    };
    let seg = mac.mac.path.segments.last()?;
    if seg.ident != "list_2d" {
        return None;
    }
    let lit: Lit = syn::parse2(mac.mac.tokens.clone()).ok()?;
    let bytes = lit_to_prefix_bytes(&lit)?;
    let p_k1 = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_k1"].concat()).to_vec();
    let p_len = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_len"].concat()).to_vec();
    let p_k2 = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_k2"].concat()).to_vec();
    let p_store = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_store"].concat()).to_vec();
    Some(vec![p_k1, p_len, p_k2, p_store])
}

/// 从合约 mod 的 Item 中提取所有 Storage/Mapping/List/List2D 的 prefix（含 list!/list_2d! 的多个），用于去重。
/// 返回 [(prefix, static_name), ...]，list! 产生 2 条，list_2d! 产生 4 条，storage!/mapping! 产生 1 条。
fn prefixes_from_item(item: &Item) -> Option<Vec<(Vec<u8>, String)>> {
    let s = match item {
        Item::Static(s) => s,
        _ => return None,
    };
    let name = s.ident.to_string();
    if let Some(p) = extract_prefix_from_new_call(&s.expr) {
        return Some(vec![(p, name)]);
    }
    if let Some(p) = extract_prefix_from_storage_mapping_macro(&s.expr) {
        return Some(vec![(p, name)]);
    }
    if let Some(ps) = extract_prefixes_from_list_macro(&s.expr) {
        return Some(ps.into_iter().map(|p| (p, name.clone())).collect());
    }
    if let Some(ps) = extract_prefixes_from_list_2d_macro(&s.expr) {
        return Some(ps.into_iter().map(|p| (p, name.clone())).collect());
    }
    None
}

/// Maps a Rust type to an ABI type name and optional length for abi.json.
/// - 基础类型: u8/u16/u32/u64/u128, i8/.., bool
/// - 数组: [u8; N] → ("AccountId", Some(N))
/// - 自定义结构体（Path）: 取路径最后一段作为 displayName，表示支持 Codec 的自定义类型
/// - 引用: &T / &mut T 递归到 T
/// - 其他（Option/Vec/泛型等）: ("ScaleBytes", None)，表示 SCALE 编码字节
fn type_to_abi(ty: &syn::Type) -> Option<(String, Option<u32>)> {
    // 引用类型：递归到内层
    if let syn::Type::Reference(r) = ty {
        return type_to_abi(&r.elem);
    }
    // 路径类型：基础类型或自定义结构体
    if let syn::Type::Path(p) = ty {
        if let Some(id) = p.path.get_ident() {
            let name = id.to_string();
            match name.as_str() {
                "u8" | "u16" | "u32" | "u64" | "u128"
                | "i8" | "i16" | "i32" | "i64" | "i128"
                | "bool" => return Some((name, None)),
                _ => {}
            }
        }
        // 自定义结构体（多段路径或单段非基础类型）：用类型名作为 displayName
        return Some((path_to_display_name(&p.path), None));
    }
    // 定长数组 [u8; N]
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
    // Option<T>, Vec<T>, 其他泛型等：统一标为 ScaleBytes，表示 SCALE 编码
    Some(("ScaleBytes".into(), None))
}

/// Formats the 4-byte selector as a hex string, e.g. `"0x60fe47b1"`, for the ABI selector field.
/// 将 4 字节 selector 格式化为十六进制字符串，如 `"0x60fe47b1"`，用于 ABI 的 selector 字段。
fn selector_hex(sel: [u8; 4]) -> String {
    format!("0x{:02x}{:02x}{:02x}{:02x}", sel[0], sel[1], sel[2], sel[3])
}

/// Generates ink!-style ABI JSON from contract name, constructor, and message list.
/// Writes to `{CARGO_TARGET_DIR}/contract/{contract_name}.json` at compile time (used by frontend/JS).
/// 根据合约名、构造函数与 message 列表，生成 ink! 风格的 ABI JSON，
/// 写入 `{CARGO_TARGET_DIR}/contract/{contract_name}.json`（编译时由宏调用，供前端/JS 使用）。
fn emit_abi(
    contract_name: &str,
    constructor_fn: &ItemFn,
    message_fns: &[(ItemFn, [u8; 4])],
) {
    // Resolve output dir: CARGO_TARGET_DIR first, else try to find workspace root, else {CARGO_MANIFEST_DIR}/target
    // 确定输出目录：优先 CARGO_TARGET_DIR，否则尝试查找 workspace 根目录，否则 {CARGO_MANIFEST_DIR}/target
    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(d) => d,
        Err(_) => return,
    };
    
    // Helper function to find workspace root
    // 辅助函数：查找 workspace 根目录
    fn find_workspace_root(start: &Path) -> Option<PathBuf> {
        let mut current = start;
        loop {
            let workspace_toml = current.join("Cargo.toml");
            if workspace_toml.exists() {
                if let Ok(content) = fs::read_to_string(&workspace_toml) {
                    if content.contains("[workspace]") {
                        return Some(current.to_path_buf());
                    }
                }
            }
            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }
        None
    }
    
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| {
        // Try to find workspace root by looking for Cargo.toml with [workspace] section
        // 尝试通过查找包含 [workspace] 的 Cargo.toml 来找到 workspace 根目录
        let manifest_path = Path::new(&manifest_dir);
        if let Some(workspace_root) = find_workspace_root(manifest_path) {
            workspace_root.join("target").to_string_lossy().into_owned()
        } else {
            // Fallback to {CARGO_MANIFEST_DIR}/target
            // 回退到 {CARGO_MANIFEST_DIR}/target
            manifest_path.join("target").to_string_lossy().into_owned()
        }
    });
    // ABI 写入 target/contract/{contract_name}.json（与文档一致）
    let contract_dir = Path::new(&target_dir).join("contract");
    if fs::create_dir_all(&contract_dir).is_err() {
        return;
    }
    let out_path = contract_dir.join(format!("{}.json", contract_name));

    // Constructor: args from constructor_fn parameters (same format as messages).
    // 构造函数：参数列表与 message 一致，从 constructor_fn 的形参生成。
    let mut constructor_args = Vec::new();
    for arg in &constructor_fn.sig.inputs {
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
            constructor_args.push(arg_obj);
        }
    }
    let constructors = vec![serde_json::json!({
        "label": constructor_fn.sig.ident.to_string(),
        "selector": "0x00000000",
        "payable": false,
        "args": constructor_args,
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
        // Return type: no return → Null; 基础类型/自定义 Codec 类型/ScaleBytes
        // 返回类型：无返回值 -> Null；否则为 type_to_abi 的 displayName（含自定义结构体）
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

    let json_str = serde_json::to_string_pretty(&abi).unwrap_or_default();
    let _ = fs::write(&out_path, &json_str);
}

// =============================================================================
// Call data layout and decode/encode (aligned with ABI)
// call() 内参数字节布局与解码/编码（与 ABI 约定一致）
// =============================================================================
// Layout: [ selector(4) | SCALE-encoded arg1 | arg2 | ... ]. All args use SCALE for consistency.
// 布局： [ selector(4) | SCALE 编码的 arg1 | arg2 | ... ]，参数统一 SCALE 编码。

/// 若返回类型为 Result<T,E> 或 Option<T>，返回 Some((内层类型 T, true=Result/false=Option))；否则返回 None。
fn unwrap_result_or_option(ty: &syn::Type) -> Option<(&syn::Type, bool)> {
    let path = match ty {
        syn::Type::Path(p) => &p.path,
        _ => return None,
    };
    let seg = path.segments.iter().last()?;
    let name = seg.ident.to_string();
    let args = match &seg.arguments {
        PathArguments::AngleBracketed(a) => &a.args,
        _ => return None,
    };
    let first_ty = args.iter().find_map(|a| match a {
        GenericArgument::Type(t) => Some(t),
        _ => None,
    })?;
    if name == "Result" {
        Some((first_ty, true))
    } else if name == "Option" {
        Some((first_ty, false))
    } else {
        None
    }
}

/// 是否为单元类型 ()。
fn is_unit_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty())
}

/// Generates code that encodes the return value `__ret` and passes it to `wrevive_api::env().return_value`.
/// 支持 ()、T、Result<T,E>、Option<T>；codec 用 Encode；sol 时 Result 只编码 Ok 值，Err 时 REVERT。
fn return_encode(ret_ty: &ReturnType, encoding: EncodingMode) -> TokenStream2 {
    match ret_ty {
        ReturnType::Default => quote! {
            wrevive_api::env().return_value(ReturnFlags::empty(), &[]);
        },
        ReturnType::Type(_, ty) => {
            let is_result = unwrap_result_or_option(ty).map(|(_, r)| r).unwrap_or(false);
            let flags_expr: TokenStream2 = if is_result {
                quote! {
                    match &__ret {
                        Ok(_) => wrevive_api::ReturnFlags::empty(),
                        Err(_) => wrevive_api::ReturnFlags::REVERT,
                    }
                }
            } else {
                quote! { wrevive_api::ReturnFlags::empty() }
            };
            match encoding {
                EncodingMode::Codec => quote! {
                    let __encoded = wrevive_api::Encode::encode(&__ret);
                    let __flags = #flags_expr;
                    wrevive_api::env().return_value(__flags, &__encoded);
                },
                EncodingMode::Sol => {
                    // Result<T,E> 在 sol 下不整体 SolEncode，而是 Ok 时只编码 T，Err 时 REVERT（与 pvm 风格一致）
                    if is_result {
                        let inner_ty = unwrap_result_or_option(ty).map(|(t, _)| t).unwrap_or(ty);
                        let ok_encode = if is_unit_type(inner_ty) {
                            quote! {
                                wrevive_api::env().return_value(wrevive_api::ReturnFlags::empty(), &[]);
                            }
                        } else {
                            quote! {
                                let __len = pvm_contract_types::SolEncode::encode_len(ok_val);
                                let mut __buf = alloc::vec![0u8; __len];
                                pvm_contract_types::SolEncode::encode_to(ok_val, &mut __buf);
                                wrevive_api::env().return_value(wrevive_api::ReturnFlags::empty(), &__buf);
                            }
                        };
                        quote! {
                            match &__ret {
                                Ok(ok_val) => { #ok_encode }
                                Err(_) => wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]),
                            }
                        }
                    } else {
                        quote! {
                            let __len = pvm_contract_types::SolEncode::encode_len(&__ret);
                            let mut __buf = alloc::vec![0u8; __len];
                            pvm_contract_types::SolEncode::encode_to(&__ret, &mut __buf);
                            let __flags = #flags_expr;
                            wrevive_api::env().return_value(__flags, &__buf);
                        }
                    }
                }
            }
        }
    }
}


// =============================================================================
// Procedural macro entry points
// 过程宏入口
// =============================================================================

/// 从当前包的 Cargo.toml 中读取第一个 `[[bin]]` 的 `name`，用于 ABI 文件名（编译 lib 时 CARGO_BIN_NAME 未设置）。
fn bin_name_from_manifest() -> Option<String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").ok()?;
    let content = fs::read_to_string(Path::new(&manifest_dir).join("Cargo.toml")).ok()?;
    let rest = content.find("[[bin]]")?;
    let after_bin = &content[rest + "[[bin]]".len()..];
    // 找 name = "xxx" 或 name = 'xxx'
    let name_start = after_bin.find("name")?;
    let after_name = &after_bin[name_start + 4..];
    let eq = after_name.find('=')?;
    let after_eq = after_name[eq + 1..].trim_start();
    let quote = after_eq.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let end = after_eq[1..].find(quote)?;
    Some(after_eq[1..1 + end].to_string())
}

/// **Main macro**: `#[revive_contract]` must be applied to a `mod`. It:
/// 1. Keeps the mod and its items;
/// 2. Emits `deploy()` and `call()` as `extern "C"` at crate root;
/// 3. Writes ABI to target/contract/{name}.json at compile time (name from bin or CONTRACT_NAME).
/// **主宏**：`#[revive_contract]` 只能挂在 `mod` 上，展开后：
/// 1. 保留该 mod 及其内部项；
/// 2. 在 crate 根生成 `deploy()` 和 `call()` 两个 `extern "C"` 函数；
/// 3. 编译时把 ABI 写入 target/contract/{name}.json（name 取自 [[bin]] 或 CONTRACT_NAME）。
#[proc_macro_attribute]
pub fn revive_contract(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析 encoding = "codec" | "sol"，默认 codec
    let encoding_mode = parse_encoding_from_contract_attr(&attr);

    // ABI 名：CARGO_BIN_NAME（编 binary 时）-> CONTRACT_NAME -> Cargo.toml 首个 [[bin]] name -> "contract"
    let contract_name = env::var("CARGO_BIN_NAME")
        .or_else(|_| env::var("CONTRACT_NAME"))
        .unwrap_or_else(|_| bin_name_from_manifest().unwrap_or_else(|| "contract".into()));

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

    // 编译期检查：Storage/Mapping/List/List2D 的 prefix 不能重复（list!/list_2d! 的多个 prefix 均参与）
    let prefix_to_names: std::collections::HashMap<Vec<u8>, Vec<String>> = mod_content
        .iter()
        .filter_map(prefixes_from_item)
        .flat_map(|pairs| pairs.into_iter())
        .fold(std::collections::HashMap::new(), |mut acc, (prefix, name)| {
            acc.entry(prefix).or_default().push(name);
            acc
        });
    for (prefix, names) in &prefix_to_names {
        if names.len() > 1 {
            let msg = format!(
                "duplicate storage prefix: {:?} used by: {} (Storage/Mapping/List/List2D prefix must be unique)",
                prefix,
                names.join(", ")
            );
            return syn::Error::new_spanned(&module, msg).to_compile_error().into();
        }
    }

    // Classify: exactly one constructor (with per-fn encoding), messages with selectors and per-fn encoding
    // 分类：构造函数（带各自 encoding）、message（带 selector 与各自 encoding）
    let mut constructor_fn: Option<(ItemFn, EncodingMode)> = None;
    let mut message_fns: Vec<(ItemFn, [u8; 4], EncodingMode)> = Vec::new();
    let mut other_items: Vec<Item> = Vec::new();

    for item in std::mem::take(mod_content) {
        match item {
            Item::Fn(mut f) => {
                let is_constructor = is_revive_constructor(&f.attrs);
                let selector = parse_selector_from_attrs(&f.attrs);
                if is_constructor {
                    let enc = parse_encoding_from_fn_attrs(&f.attrs, encoding_mode);
                    f.attrs = strip_revive_attrs(&f.attrs);
                    constructor_fn = Some((f, enc));
                } else if is_revive_message(&f.attrs) {
                    let enc = parse_encoding_from_fn_attrs(&f.attrs, encoding_mode);
                    let sel = selector.unwrap_or_else(|| {
                        selector_from_name(&f.sig.ident.to_string())
                    });
                    f.attrs = strip_revive_attrs(&f.attrs);
                    message_fns.push((f, sel, enc));
                } else {
                    other_items.push(Item::Fn(f));
                }
            }
            other => other_items.push(other),
        }
    }

    let (constructor_fn, constructor_encoding) = match constructor_fn {
        Some((f, enc)) => (f, enc),
        None => {
            return syn::Error::new_spanned(&module, "exactly one #[revive(constructor)] function required / 需要恰好一个 #[revive(constructor)] 函数")
                .to_compile_error()
                .into();
        }
    };
    let constructor_name = constructor_fn.sig.ident.clone();

    // Emit ABI to target/{contract_name}.json
    let message_fns_abi: Vec<(ItemFn, [u8; 4])> = message_fns.iter().map(|(f, sel, _)| (f.clone(), *sel)).collect();
    emit_abi(&contract_name, &constructor_fn, &message_fns_abi);

    // Constructor parameter parsing: codec = SCALE-decode; sol = SolDecode::decode_at with running offset.
    // 构造函数参数解析：codec 用 SCALE 解码，sol 用 SolDecode::decode_at 与偏移。
    let mut constructor_input_vars: Vec<(syn::Ident, TokenStream2)> = Vec::new();
    let mut constructor_call_exprs = Vec::new();
    for arg in &constructor_fn.sig.inputs {
        let FnArg::Typed(pt) = arg else { continue };
        let name = match pt.pat.as_ref() {
            syn::Pat::Ident(pi) => pi.ident.clone(),
            _ => continue,
        };
        let ty = pt.ty.as_ref();
        let type_tt = quote! { #ty };
        constructor_input_vars.push((name.clone(), type_tt));
        constructor_call_exprs.push(quote! { #name });
    }

    let constructor_parse: TokenStream2 = if constructor_input_vars.is_empty() {
        quote! {}
    } else {
        let input_setup = quote! {
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
        };
        match constructor_encoding {
            EncodingMode::Codec => {
                let mut scale_stmts = Vec::new();
                let scale_input = quote! { __scale_input };
                for (name, type_tt) in &constructor_input_vars {
                    scale_stmts.push(quote! {
                        let #name: #type_tt = match <#type_tt as wrevive_api::Decode>::decode(&mut #scale_input) {
                            Ok(val) => val,
                            Err(_) => {
                                wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                                return;
                            }
                        };
                    });
                }
                quote! {
                    #input_setup
                    let mut __scale_input = __input;
                    #(#scale_stmts)*
                }
            }
            EncodingMode::Sol => {
                let mut sol_stmts = Vec::new();
                for (name, type_tt) in &constructor_input_vars {
                    sol_stmts.push(quote! {
                        let #name: #type_tt = <#type_tt as pvm_contract_types::SolDecode>::decode_at(__input, __sol_off);
                        __sol_off += pvm_contract_types::SolEncode::encode_len(&#name);
                    });
                }
                quote! {
                    #input_setup
                    let mut __sol_off: usize = 0;
                    #(#sol_stmts)*
                }
            }
        }
    };

    // deploy() 体：解析构造函数参数、调用构造函数、按 encoding 编码返回。
    let constructor_return_encode = match &constructor_fn.sig.output {
        ReturnType::Default => quote! { wrevive_api::env().return_value(wrevive_api::ReturnFlags::empty(), &[]); },
        ReturnType::Type(_, ty) => {
            let is_result = unwrap_result_or_option(ty).map(|(_, r)| r).unwrap_or(false);
            let flags = if is_result {
                quote! { if let Err(_) = &__ret { wrevive_api::ReturnFlags::REVERT } else { wrevive_api::ReturnFlags::empty() } }
            } else {
                quote! { wrevive_api::ReturnFlags::empty() }
            };
            match constructor_encoding {
                EncodingMode::Codec => quote! {
                    let __encoded = wrevive_api::Encode::encode(&__ret);
                    wrevive_api::env().return_value(#flags, &__encoded);
                },
                EncodingMode::Sol => quote! {
                    let __len = pvm_contract_types::SolEncode::encode_len(&__ret);
                    let mut __buf = alloc::vec![0u8; __len];
                    pvm_contract_types::SolEncode::encode_to(&__ret, &mut __buf);
                    wrevive_api::env().return_value(#flags, &__buf);
                },
            }
        }
    };
    let deploy_body: TokenStream2 = quote! {
        #constructor_parse
        let __ret = #mod_name::#constructor_name(#(#constructor_call_exprs),*);
        #constructor_return_encode
    };
    let deploy_fn: Item = syn::parse2(quote! {
        #[polkavm_derive::polkavm_export]
        #[allow(unreachable_code)]
        pub extern "C" fn deploy() {
            #deploy_body
        }
    })
    .unwrap();

    // Put constructor, messages, and other items back into the mod (deploy/call are emitted outside)
    // 把用户的 constructor、message、其他项重新放回 mod（deploy/call 生成在 mod 外）
    mod_content.push(Item::Fn(constructor_fn));
    for (f, _, _) in &message_fns {
        mod_content.push(Item::Fn(f.clone()));
    }
    mod_content.extend(other_items);

    // Build match arms for call(): decode __input[4..] by each message's encoding (codec or sol).
    // 为 call() 生成 match 分支：每个 message 按自己的 encoding 解码与编码。
    let match_arms: Vec<TokenStream2> = message_fns
        .iter()
        .map(|(f, sel, fn_enc)| {
            let fn_name = &f.sig.ident;
            let sig = &f.sig;
            let min_len: usize = 4; // selector 占 4 字节 / selector is 4 bytes

            let mut input_vars: Vec<(syn::Ident, TokenStream2)> = Vec::new();
            let mut call_exprs = Vec::new();
            for arg in &sig.inputs {
                let FnArg::Typed(pt) = arg else { continue };
                let name = match pt.pat.as_ref() {
                    syn::Pat::Ident(pi) => pi.ident.clone(),
                    _ => continue,
                };
                let ty = pt.ty.as_ref();
                let type_tt = quote! { #ty };
                input_vars.push((name.clone(), type_tt));
                call_exprs.push(quote! { #name });
            }

            let input_parse = if input_vars.is_empty() {
                quote! {}
            } else {
                match fn_enc {
                    EncodingMode::Codec => {
                        let mut scale_stmts = Vec::new();
                        let scale_input = quote! { __scale_input };
                        for (name, type_tt) in &input_vars {
                            scale_stmts.push(quote! {
                                let #name: #type_tt = match <#type_tt as wrevive_api::Decode>::decode(&mut #scale_input) {
                                    Ok(val) => val,
                                    Err(_) => {
                                        wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                                        return;
                                    }
                                };
                            });
                        }
                        quote! {
                            let mut __scale_input = &__input[4..];
                            #(#scale_stmts)*
                        }
                    }
                    EncodingMode::Sol => {
                        let mut sol_stmts = Vec::new();
                        for (name, type_tt) in &input_vars {
                            sol_stmts.push(quote! {
                                let #name: #type_tt = <#type_tt as pvm_contract_types::SolDecode>::decode_at(&__input[4..], __sol_off);
                                __sol_off += pvm_contract_types::SolEncode::encode_len(&#name);
                            });
                        }
                        quote! {
                            let mut __sol_off: usize = 0;
                            #(#sol_stmts)*
                        }
                    }
                }
            };
            let ret_ty = &sig.output;
            let encode_and_return = return_encode(ret_ty, *fn_enc);
            let sel_u32 = u32::from_be_bytes(*sel);
            quote! {
                #sel_u32 => {
                    if __input_len >= #min_len {
                        #input_parse
                        let __ret = #mod_name::#fn_name(#(#call_exprs),*);
                        #encode_and_return
                    } else {
                        wrevive_api::env().return_value(ReturnFlags::REVERT, &[]);
                    }
                }
            }
        })
        .collect();

    // Emit call(): read call data, dispatch by first 4-byte selector, encode return value
    // 生成 call()：读取 call data，按前 4 字节 selector 分发到对应 message，并编码返回值
    let call_fn: Item = syn::parse2(quote! {
        #[polkavm_derive::polkavm_export]
        #[allow(unreachable_code)]
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

    // 在 crate 根自动添加必要的导入：有任一 codec 则 use Decode，有任一 sol 则 use SolDecode/SolEncode（合约需依赖 pvm-contract-types）
    let any_sol = constructor_encoding == EncodingMode::Sol || message_fns.iter().any(|(_, _, enc)| *enc == EncodingMode::Sol);
    let use_encoding: TokenStream2 = if any_sol {
        quote! {
            use wrevive_api::Decode;
            use pvm_contract_types::{SolDecode, SolEncode};
        }
    } else {
        quote! { use wrevive_api::Decode; }
    };

    // Expansion: imports + original mod + deploy() + call()
    // 展开结果：导入 + 原 mod + deploy() + call()
    quote! {
        #use_encoding

        #module

        #deploy_fn

        #call_fn
    }
    .into()
}

// =============================================================================
// storage! / mapping! — prefix 使用 Blake2s256 取前 4 字节
// =============================================================================

/// 从字符串或字节串字面量得到字节（供 storage!/mapping! 哈希用）。
fn lit_to_prefix_bytes(lit: &Lit) -> Option<Vec<u8>> {
    match lit {
        Lit::Str(s) => Some(s.value().into_bytes()),
        Lit::ByteStr(bs) => Some(bs.value()),
        _ => None,
    }
}

/// 将 prefix 字节用 Blake2s256 哈希后取前 4 字节，生成 `&[u8; 4]` 形式的 token。
fn blake2s_prefix_4_bytes(bytes: &[u8]) -> [u8; 4] {
    let hash = Blake2s256::digest(bytes);
    [hash[0], hash[1], hash[2], hash[3]]
}

/// `storage!(b"value")` 或 `storage!("value")` → `Storage::new(&[b0, b1, b2, b3])`，
/// 其中 4 字节为 Blake2s256(prefix) 的前 4 字节。类型由上下文推断，如 `static V: Storage<u32> = storage!(b"value");`
#[proc_macro]
pub fn storage(input: TokenStream) -> TokenStream {
    let lit = match syn::parse::<Lit>(input) {
        Ok(l) => l,
        Err(e) => return e.to_compile_error().into(),
    };
    let bytes = match lit_to_prefix_bytes(&lit) {
        Some(b) => b,
        None => {
            return syn::Error::new_spanned(lit, "storage! expects a string or byte string literal, e.g. storage!(b\"value\") or storage!(\"value\")")
                .to_compile_error()
                .into();
        }
    };
    let prefix = blake2s_prefix_4_bytes(&bytes);
    let (b0, b1, b2, b3) = (prefix[0], prefix[1], prefix[2], prefix[3]);
    quote! {
        wrevive_api::Storage::new(&[#b0, #b1, #b2, #b3])
    }
    .into()
}

/// `mapping!(b"balance")` 或 `mapping!("balance")` → `Mapping::new(&[b0, b1, b2, b3])`，
/// 其中 4 字节为 Blake2s256(prefix) 的前 4 字节。类型由上下文推断，如 `static M: Mapping<K, V> = mapping!(b"balance");`
#[proc_macro]
pub fn mapping(input: TokenStream) -> TokenStream {
    let lit = match syn::parse::<Lit>(input) {
        Ok(l) => l,
        Err(e) => return e.to_compile_error().into(),
    };
    let bytes = match lit_to_prefix_bytes(&lit) {
        Some(b) => b,
        None => {
            return syn::Error::new_spanned(lit, "mapping! expects a string or byte string literal, e.g. mapping!(b\"balance\") or mapping!(\"balance\")")
                .to_compile_error()
                .into();
        }
    };
    let prefix = blake2s_prefix_4_bytes(&bytes);
    let (b0, b1, b2, b3) = (prefix[0], prefix[1], prefix[2], prefix[3]);
    quote! {
        wrevive_api::Mapping::new(&[#b0, #b1, #b2, #b3])
    }
    .into()
}

// =============================================================================
// list! / list_2d! — 单一 prefix，Blake2s 取前 4 字节生成多个 prefix，参与去重
// =============================================================================

/// `list!(b"mylist")` 或 `list!("mylist")` → `List::new(&[4字节], &[4字节])`，两段均为 Blake2s(prefix) / Blake2s(prefix+"_items") 前 4 字节。
#[proc_macro]
pub fn list(input: TokenStream) -> TokenStream {
    let lit = match syn::parse::<Lit>(input) {
        Ok(l) => l,
        Err(e) => return e.to_compile_error().into(),
    };
    let bytes = match lit_to_prefix_bytes(&lit) {
        Some(b) => b,
        None => {
            return syn::Error::new_spanned(lit, "list! expects a string or byte string literal, e.g. list!(b\"mylist\")")
                .to_compile_error()
                .into();
        }
    };
    let p_id = blake2s_prefix_4_bytes(&bytes);
    let p_items = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_items"].concat());
    let (a0, a1, a2, a3) = (p_id[0], p_id[1], p_id[2], p_id[3]);
    let (b0, b1, b2, b3) = (p_items[0], p_items[1], p_items[2], p_items[3]);
    quote! {
        wrevive_api::List::new(&[#a0, #a1, #a2, #a3], &[#b0, #b1, #b2, #b3])
    }
    .into()
}

/// `list_2d!(b"dl")` 或 `list_2d!("dl")` → `List2D::new(&[4字节], ...)`，四段为 Blake2s(prefix+"_k1/_len/_k2/_store") 前 4 字节。
#[proc_macro]
pub fn list_2d(input: TokenStream) -> TokenStream {
    let lit = match syn::parse::<Lit>(input) {
        Ok(l) => l,
        Err(e) => return e.to_compile_error().into(),
    };
    let bytes = match lit_to_prefix_bytes(&lit) {
        Some(b) => b,
        None => {
            return syn::Error::new_spanned(lit, "list_2d! expects a string or byte string literal, e.g. list_2d!(b\"dl\")")
                .to_compile_error()
                .into();
        }
    };
    let p_k1 = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_k1"].concat());
    let p_len = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_len"].concat());
    let p_k2 = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_k2"].concat());
    let p_store = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_store"].concat());
    let (a0, a1, a2, a3) = (p_k1[0], p_k1[1], p_k1[2], p_k1[3]);
    let (b0, b1, b2, b3) = (p_len[0], p_len[1], p_len[2], p_len[3]);
    let (c0, c1, c2, c3) = (p_k2[0], p_k2[1], p_k2[2], p_k2[3]);
    let (d0, d1, d2, d3) = (p_store[0], p_store[1], p_store[2], p_store[3]);
    quote! {
        wrevive_api::List2D::new(
            &[#a0, #a1, #a2, #a3],
            &[#b0, #b1, #b2, #b3],
            &[#c0, #c1, #c2, #c3],
            &[#d0, #d1, #d2, #d3],
        )
    }
    .into()
}
