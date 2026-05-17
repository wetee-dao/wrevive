//! 属性解析：encoding、revive(constructor/message)、selector、path 显示名等。

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use sha3::{Digest, Keccak256};
use syn::{
    Attribute, GenericArgument, Lit, Meta, PathArguments, Token, Type, punctuated::Punctuated,
};

/// Encoding mode for contract messages: codec (SCALE) or sol (Solidity ABI).
/// 合约 message 的编码模式：codec（SCALE）或 sol（Solidity ABI）。

/// Computes the 4-byte message selector from the function name only (for Codec mode).
/// Uses Keccak-256 of the bare function name (e.g. `keccak256("transfer")`).
///
/// # 中文
/// 从函数名计算 4 字节消息选择器（Codec 模式）。
pub fn selector_from_name(name: &str) -> [u8; 4] {
    let hash = Keccak256::digest(name.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]]
}

/// Computes the 4-byte message selector, with encoding-aware inputs.
///
/// - **Codec** mode: keccak256 of the function name only (e.g. `keccak256("transfer")`).
/// - **Sol** mode: keccak256 of the canonical Solidity signature
///   (e.g. `keccak256("transfer(address,uint256)")`) — identical to how
///   Solidity, MetaMask, and `cargo-pvm-contract` compute selectors.
///
/// # 中文
/// - Codec 模式：对函数名做 keccak256（如 `"transfer"`）。
/// - Sol 模式：对 Solidity 规范签名做 keccak256（如 `"transfer(address,uint256)"`），
///   与 Solidity / MetaMask / cargo-pvm-contract 完全一致。
pub fn compute_selector(name: &str, params: &[Type], enc: EncodingMode) -> [u8; 4] {
    match enc {
        EncodingMode::Codec => selector_from_name(name),
        EncodingMode::Sol => {
            let sol_sig = canonical_solidity_signature(name, params);
            let hash = Keccak256::digest(sol_sig.as_bytes());
            [hash[0], hash[1], hash[2], hash[3]]
        }
    }
}

/// Build the canonical Solidity signature string: `name(type1,type2,...)`.
///
/// # 中文
/// 构建 Solidity 规范签名字符串。
fn canonical_solidity_signature(name: &str, params: &[Type]) -> String {
    let param_strs: Vec<String> = params.iter().map(rust_type_to_solidity).collect();
    format!("{}({})", name, param_strs.join(","))
}

/// Map a Rust type to its Solidity canonical type name.
/// Mirrors `SolType::from_rust_type` in `cargo-pvm-contract`.
///
/// # 中文
/// 将 Rust 类型映射为 Solidity 规范类型字符串。
pub(crate) fn rust_type_to_solidity(ty: &Type) -> String {
    // Strip reference
    let ty = if let Type::Reference(r) = ty {
        &*r.elem
    } else {
        ty
    };

    // Handle Vec<T> → T[]
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            if seg.ident == "Vec" {
                if let PathArguments::AngleBracketed(a) = &seg.arguments {
                    if let Some(GenericArgument::Type(inner)) = a.args.first() {
                        return format!("{}[]", rust_type_to_solidity(inner));
                    }
                }
            }
        }
    }

    // Handle [T; N] → T[N]  (and [u8; N] → bytesN)
    if let Type::Array(arr) = ty {
        if let syn::Expr::Lit(lit) = &arr.len {
            if let Lit::Int(n) = &lit.lit {
                if let Ok(size) = n.base10_parse::<usize>() {
                    let inner = rust_type_to_solidity(&arr.elem);
                    if inner == "uint8" {
                        return format!("bytes{}", size);
                    }
                    return format!("{}[{}]", inner, size);
                }
            }
        }
    }

    // Handle tuples (T1, T2, ...)
    if let Type::Tuple(t) = ty {
        let elems: Vec<String> = t.elems.iter().map(rust_type_to_solidity).collect();
        return format!("({})", elems.join(","));
    }

    // Normalize the type name (strip whitespace, handle common aliases)
    let type_str = quote!(#ty).to_string().replace(' ', "");

    match type_str.as_str() {
        "Address" | "wrevive_api::Address" => "address".into(),
        "U256" | "wrevive_api::U256" | "ruint::aliases::U256" => "uint256".into(),
        "u128" => "uint128".into(),
        "u64" => "uint64".into(),
        "u32" => "uint32".into(),
        "u16" => "uint16".into(),
        "u8" => "uint8".into(),
        "i128" => "int128".into(),
        "i64" => "int64".into(),
        "i32" => "int32".into(),
        "i16" => "int16".into(),
        "i8" => "int8".into(),
        "bool" => "bool".into(),
        "String" | "alloc::string::String" => "string".into(),
        "Bytes" | "wrevive_api::Bytes" => "bytes".into(),
        "BlockNumber" | "wrevive_api::BlockNumber" => "uint32".into(),
        "AccountId" | "wrevive_api::AccountId" => "bytes32".into(),
        "H256" | "wrevive_api::H256" => "bytes32".into(),
        _ => {
            // For unknown/custom types, use the last path segment as the type name
            if let Type::Path(p) = ty {
                if let Some(seg) = p.path.segments.last() {
                    return seg.ident.to_string();
                }
            }
            type_str
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EncodingMode {
    Codec,
    Sol,
}

/// Parses `encoding = "codec"` or `encoding = "sol"` from `#[revive_contract(...)]` attribute.
/// Default is Codec when absent or unrecognized.
pub fn parse_encoding_from_contract_attr(attr: &TokenStream) -> EncodingMode {
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
        let syn::Expr::Lit(expr_lit) = &nv.value else {
            continue;
        };
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

/// Parses `encoding = "codec"` / `encoding = "sol"` or path `codec` / `sol` from
/// `#[revive(message, ...)]` or `#[revive(constructor, ...)]`.
pub fn parse_encoding_from_fn_attrs(attrs: &[Attribute], default: EncodingMode) -> EncodingMode {
    for attr in attrs {
        if !attr.path().is_ident("revive") {
            continue;
        }
        let syn::Meta::List(list) = &attr.meta else {
            continue;
        };
        let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };
        for meta in nested {
            match &meta {
                Meta::NameValue(nv) if nv.path.is_ident("encoding") => {
                    let syn::Expr::Lit(expr_lit) = &nv.value else {
                        continue;
                    };
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
pub fn parse_selector_from_attrs(attrs: &[Attribute]) -> Option<[u8; 4]> {
    for attr in attrs {
        if !attr.path().is_ident("revive") {
            continue;
        }
        let syn::Meta::List(list) = &attr.meta else {
            continue;
        };
        let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };
        for meta in nested {
            let Meta::NameValue(nv) = meta else { continue };
            if !nv.path.is_ident("selector") {
                continue;
            }
            let syn::Expr::Lit(expr_lit) = &nv.value else {
                continue;
            };
            let Lit::Str(s) = &expr_lit.lit else { continue };
            let hex_str = s.value();
            let hex = hex_str.trim_start_matches("0x");
            if hex.len() != 8 {
                continue;
            }
            let mut arr = [0u8; 4];
            for (i, c) in hex.chars().enumerate() {
                let v = c.to_digit(16)?;
                arr[i / 2] = (arr[i / 2] << 4) | v as u8;
            }
            return Some(arr);
        }
    }
    None
}

/// Returns true if the function is marked with `#[revive(constructor)]`.
pub fn is_revive_constructor(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("revive") {
            continue;
        }
        let syn::Meta::List(list) = &attr.meta else {
            continue;
        };
        let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };
        for meta in nested {
            if let Meta::Path(p) = meta {
                if p.is_ident("constructor") {
                    return true;
                }
            }
        }
    }
    false
}

/// Returns true if the function is marked with `#[revive(message)]` or
/// `#[revive(message, selector = ...)]`.
pub fn is_revive_message(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("revive") {
            continue;
        }
        let syn::Meta::List(list) = &attr.meta else {
            continue;
        };
        let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };
        for meta in nested {
            if let Meta::Path(p) = meta {
                if p.is_ident("message") {
                    return true;
                }
            }
        }
    }
    false
}

/// Returns true if the function is marked with `#[revive(message, write)]`.
/// 用于 ABI：有 write 标签则该 message 的 "mutates" 固定为 true。
pub fn has_revive_mutates(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("revive") {
            continue;
        }
        let syn::Meta::List(list) = &attr.meta else {
            continue;
        };
        let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };
        for meta in nested {
            if let Meta::Path(p) = meta {
                if p.is_ident("write") {
                    return true;
                }
            }
        }
    }
    false
}

/// Returns true if the function is marked with `#[revive(fallback)]`.
/// 当请求 selector 未匹配任何 message 时，调用 fallback。
pub fn is_revive_fallback(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("revive") {
            continue;
        }
        let syn::Meta::List(list) = &attr.meta else {
            continue;
        };
        let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };
        for meta in nested {
            if let Meta::Path(p) = meta {
                if p.is_ident("fallback") {
                    return true;
                }
            }
        }
    }
    false
}

/// Removes all `#[revive(...)]` attributes so they are not processed again.
pub fn strip_revive_attrs(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|a| !a.path().is_ident("revive"))
        .cloned()
        .collect()
}

/// 从 syn::Path 取类型显示名（多段路径取最后一段）。
pub fn path_to_display_name(path: &syn::Path) -> String {
    path.segments
        .iter()
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_else(|| "ScaleBytes".into())
}
