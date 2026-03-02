//! 属性解析：encoding、revive(constructor/message)、selector、path 显示名等。

use blake2::Blake2s256;
use digest::Digest;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    punctuated::Punctuated,
    Attribute, Lit, Meta, Token,
};

/// Encoding mode for contract messages: codec (SCALE) or sol (Solidity ABI).
/// 合约 message 的编码模式：codec（SCALE）或 sol（Solidity ABI）。
/// Computes the 4-byte message selector from the function name (ink!-compatible).
pub fn selector_from_name(name: &str) -> [u8; 4] {
    let hash = Blake2s256::digest(name.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]]
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
pub fn parse_encoding_from_fn_attrs(attrs: &[Attribute], default: EncodingMode) -> EncodingMode {
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
pub fn parse_selector_from_attrs(attrs: &[Attribute]) -> Option<[u8; 4]> {
    for attr in attrs {
        if !attr.path().is_ident("revive") {
            continue;
        }
        let syn::Meta::List(list) = &attr.meta else { continue };
        let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) else {
            continue;
        };
        for meta in nested {
            let Meta::NameValue(nv) = meta else { continue };
            if !nv.path.is_ident("selector") {
                continue;
            }
            let syn::Expr::Lit(expr_lit) = &nv.value else { continue };
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
        let syn::Meta::List(list) = &attr.meta else { continue };
        let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) else {
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

/// Returns true if the function is marked with `#[revive(message)]` or `#[revive(message, selector = ...)]`.
pub fn is_revive_message(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("revive") {
            continue;
        }
        let syn::Meta::List(list) = &attr.meta else { continue };
        let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) else {
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
