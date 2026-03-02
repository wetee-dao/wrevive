//! storage! / mapping! / list! / list_2d! 宏：prefix 使用 Blake2s256 取前 4 字节。

use crate::prefix;
use proc_macro::TokenStream;
use quote::quote;
use syn::Lit;

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
                "storage! expects a string or byte string literal, e.g. storage!(b\"value\") or storage!(\"value\")",
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
                "mapping! expects a string or byte string literal, e.g. mapping!(b\"balance\") or mapping!(\"balance\")",
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
                "list! expects a string or byte string literal, e.g. list!(b\"mylist\")",
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
                "list_2d! expects a string or byte string literal, e.g. list_2d!(b\"dl\")",
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
