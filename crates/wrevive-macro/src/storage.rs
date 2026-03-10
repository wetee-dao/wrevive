//! storage! / mapping! / list! / list_2d! 宏：prefix 使用 Blake2s256 取前 4 字节。

use crate::prefix;
use proc_macro::TokenStream;
use quote::quote;
use syn::Lit;

/// Implements storage! macro for creating Storage instances with Blake2s256 prefix.
/// 
/// # English
/// Creates a Storage instance using Blake2s256 hash of the input prefix.
/// Takes a string or byte string literal and generates a 4-byte prefix.
/// 
/// # 中文
/// 使用 Blake2s256 哈希前缀实现 storage! 宏。
/// 接受字符串或字节串字面量并生成 4 字节前缀。
pub fn storage_impl(input: TokenStream) -> TokenStream {
    let lit = match syn::parse::<Lit>(input) {
        Ok(l) => l,
        Err(e) => return e.to_compile_error().into(),
    };
    let bytes = match prefix::lit_to_prefix_bytes(&lit) {
        Some(b) => b,
        None => {
            return syn::Error::new_spanned(
                lit,
                "storage! expects a string or byte string literal, e.g. storage!(b\"value\") or storage!(\"value\") / storage! 需要字符串或字节串字面量，例如 storage!(b\"value\") 或 storage!(\"value\")",
            )
            .to_compile_error()
            .into();
        }
    };
    let prefix = prefix::blake2s_prefix_4_bytes(&bytes);
    let (b0, b1, b2, b3) = (prefix[0], prefix[1], prefix[2], prefix[3]);
    quote! {
        wrevive_api::Storage::new(&[#b0, #b1, #b2, #b3])
    }
    .into()
}

/// Implements mapping! macro for creating Mapping instances with Blake2s256 prefix.
/// 
/// # English
/// Creates a Mapping instance using Blake2s256 hash of the input prefix.
/// Takes a string or byte string literal and generates a 4-byte prefix.
/// 
/// # 中文
/// 使用 Blake2s256 哈希前缀实现 mapping! 宏。
/// 接受字符串或字节串字面量并生成 4 字节前缀。
pub fn mapping_impl(input: TokenStream) -> TokenStream {
    let lit = match syn::parse::<Lit>(input) {
        Ok(l) => l,
        Err(e) => return e.to_compile_error().into(),
    };
    let bytes = match prefix::lit_to_prefix_bytes(&lit) {
        Some(b) => b,
        None => {
            return syn::Error::new_spanned(
                lit,
                "mapping! expects a string or byte string literal, e.g. mapping!(b\"balance\") or mapping!(\"balance\") / mapping! 需要字符串或字节串字面量，例如 mapping!(b\"balance\") 或 mapping!(\"balance\")",
            )
            .to_compile_error()
            .into();
        }
    };
    let prefix = prefix::blake2s_prefix_4_bytes(&bytes);
    let (b0, b1, b2, b3) = (prefix[0], prefix[1], prefix[2], prefix[3]);
    quote! {
        wrevive_api::Mapping::new(&[#b0, #b1, #b2, #b3])
    }
    .into()
}

/// Implements list! macro for creating List instances with Blake2s256 prefix.
/// 
/// # English
/// Creates a List instance using Blake2s256 hash of input prefix.
/// Generates both ID and items prefixes for the List storage structure.
/// 
/// # 中文
/// 使用 Blake2s256 哈希前缀实现 list! 宏。
/// 为 List 存储结构生成 ID 和 items 前缀。
pub fn list_impl(input: TokenStream) -> TokenStream {
    let lit = match syn::parse::<Lit>(input) {
        Ok(l) => l,
        Err(e) => return e.to_compile_error().into(),
    };
    let bytes = match prefix::lit_to_prefix_bytes(&lit) {
        Some(b) => b,
        None => {
            return syn::Error::new_spanned(
                lit,
                "list! expects a string or byte string literal, e.g. list!(b\"mylist\") / list! 需要字符串或字节串字面量，例如 list!(b\"mylist\")",
            )
            .to_compile_error()
            .into();
        }
    };
    let p_id = prefix::blake2s_prefix_4_bytes(&bytes);
    let p_items = prefix::blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_items"].concat());
    let (a0, a1, a2, a3) = (p_id[0], p_id[1], p_id[2], p_id[3]);
    let (b0, b1, b2, b3) = (p_items[0], p_items[1], p_items[2], p_items[3]);
    quote! {
        wrevive_api::List::new(&[#a0, #a1, #a2, #a3], &[#b0, #b1, #b2, #b3])
    }
    .into()
}

/// Implements list_2d! macro for creating List2D instances with Blake2s256 prefix.
/// 
/// # English
/// Creates a List2D instance using Blake2s256 hash of input prefix.
/// Generates four prefixes for the 2D list storage structure: k1, len, k2, store.
/// 
/// # 中文
/// 使用 Blake2s256 哈希前缀实现 list_2d! 宏。
/// 为 2D 列表存储结构生成四个前缀：k1、len、k2、store。
pub fn list_2d_impl(input: TokenStream) -> TokenStream {
    let lit = match syn::parse::<Lit>(input) {
        Ok(l) => l,
        Err(e) => return e.to_compile_error().into(),
    };
    let bytes = match prefix::lit_to_prefix_bytes(&lit) {
        Some(b) => b,
        None => {
            return syn::Error::new_spanned(
                lit,
                "list_2d! expects a string or byte string literal, e.g. list_2d!(b\"dl\") / list_2d! 需要字符串或字节串字面量，例如 list_2d!(b\"dl\")",
            )
            .to_compile_error()
            .into();
        }
    };
    let p_k1 = prefix::blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_k1"].concat());
    let p_len = prefix::blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_len"].concat());
    let p_k2 = prefix::blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_k2"].concat());
    let p_store = prefix::blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_store"].concat());
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
