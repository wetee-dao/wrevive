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
use std::path::{Path, PathBuf};
use syn::{
    parse_macro_input,
    punctuated::Punctuated,
    Attribute, FnArg, GenericArgument, Item, ItemFn, Lit, Meta, PathArguments, ReturnType, Token,
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

/// 从 syn::Path 取类型显示名（多段路径取最后一段，如 `crate::BalanceInfo` → `BalanceInfo`）。
fn path_to_display_name(path: &syn::Path) -> String {
    path.segments
        .iter()
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_else(|| "ScaleBytes".into())
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

/// Generates ink!-style ABI JSON from contract name, constructor name, and message list.
/// Writes to `{CARGO_TARGET_DIR}/contract/{contract_name}.json` at compile time (used by frontend/JS).
/// 根据合约名、构造函数名与 message 列表，生成 ink! 风格的 ABI JSON，
/// 写入 `{CARGO_TARGET_DIR}/contract/{contract_name}.json`（编译时由宏调用，供前端/JS 使用）。
fn emit_abi(
    contract_name: &str,
    constructor_name: &str,
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
// Layout: [ selector(4) | arg1 | arg2 | ... ]
// 布局： [ selector(4) | arg1 | arg2 | ... ]
// - u32: 4 bytes, little-endian, i.e. __input[4..8]
// - [u8; 20] (AccountId): 20 bytes, i.e. __input[4..24]

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

/// Generates code that encodes the return value `__ret` and passes it to `wrevive_api::env().return_value`.
/// 支持 ()、T、Result<T,E>（Ok 编码返回，Err 则 REVERT）、Option<T>（Some 编码返回，None 空字节）。
fn return_encode(ret_ty: &ReturnType) -> TokenStream2 {
    match ret_ty {
        ReturnType::Default => quote! {
            wrevive_api::env().return_value(ReturnFlags::empty(), &[]);
        },
        ReturnType::Type(_, ty) => {
            if let Some((_inner_ty, is_result)) = unwrap_result_or_option(ty) {
                if is_result {
                    quote! {
                        match __ret {
                            Ok(__v) => {
                                let __encoded = wrevive_api::Encode::encode(&__v);
                                wrevive_api::env().return_value(ReturnFlags::empty(), &__encoded);
                            }
                            Err(_) => {
                                wrevive_api::env().return_value(ReturnFlags::REVERT, &[]);
                            }
                        }
                    }
                } else {
                    quote! {
                        match __ret {
                            Some(__v) => {
                                let __encoded = wrevive_api::Encode::encode(&__v);
                                wrevive_api::env().return_value(ReturnFlags::empty(), &__encoded);
                            }
                            None => {
                                wrevive_api::env().return_value(ReturnFlags::empty(), &[]);
                            }
                        }
                    }
                }
            } else {
                quote! {
                    {
                        let __encoded = wrevive_api::Encode::encode(&__ret);
                        wrevive_api::env().return_value(ReturnFlags::empty(), &__encoded);
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
pub fn revive_contract(_attr: TokenStream, item: TokenStream) -> TokenStream {
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

    // deploy() 体：若构造函数返回 Result，则 Err 时 REVERT（需在移动 constructor_fn 前生成）
    let deploy_body: TokenStream2 = match &constructor_fn.sig.output {
        ReturnType::Type(_, ty) if unwrap_result_or_option(ty).map(|(_, r)| r).unwrap_or(false) => {
            quote! {
                match #mod_name::#constructor_name() {
                    Ok(_) => {}
                    Err(_) => {
                        wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    }
                }
            }
        }
        _ => quote! { #mod_name::#constructor_name(); },
    };
    let deploy_fn: Item = syn::parse2(quote! {
        #[polkavm_derive::polkavm_export]
        pub extern "C" fn deploy() {
            #deploy_body
        }
    })
    .unwrap();

    // Put constructor, messages, and other items back into the mod (deploy/call are emitted outside)
    // 把用户的 constructor、message、其他项重新放回 mod（deploy/call 生成在 mod 外）
    mod_content.push(Item::Fn(constructor_fn));
    for (f, _) in &message_fns {
        mod_content.push(Item::Fn(f.clone()));
    }
    mod_content.extend(other_items);

    // Build match arms for call(): all arguments use SCALE decode (unified approach)
    // 为 call() 生成 match 分支：所有参数统一使用 SCALE 解码（测试和正式环境一致）
    let match_arms: Vec<TokenStream2> = message_fns
        .iter()
        .map(|(f, sel)| {
            let fn_name = &f.sig.ident;
            let sig = &f.sig;
            let min_len: usize = 4; // 至少需要 selector 的 4 字节
            
            // 收集所有参数的类型和名称，统一使用 SCALE 解码
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
            
            // 统一使用 SCALE 解码（测试和正式环境一致）
            let input_parse = if input_vars.is_empty() {
                quote! {}
            } else {
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
            };
            let ret_ty = &sig.output;
            let encode_and_return = return_encode(ret_ty);
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

    // 在 crate 根自动添加必要的导入（SCALE 编码/解码）
    // Automatically add necessary imports at crate root (for SCALE encode/decode)
    let use_scale_codec: Item = syn::parse2(quote! {
        use wrevive_api::Decode;
    })
    .unwrap();

    // Expansion: imports + original mod + deploy() + call()
    // 展开结果：导入 + 原 mod + deploy() + call()
    quote! {
        #use_scale_codec

        #module

        #deploy_fn

        #call_fn
    }
    .into()
}
