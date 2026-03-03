//! 合约间调用接口生成：selector 常量、encode_*、call_raw、call_*、constructor 的 encode_* 与 instantiate_*。

use crate::attrs::EncodingMode;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Pat, ReturnType};

/// 将 message 函数名转为 SCREAMING_SNAKE_CASE（用于 SELECTOR_XXX 常量名）。
fn to_selector_constant_name(name: &str) -> String {
    name.to_uppercase()
}

/// 生成合约间调用的 interface 子模块：SELECTOR_*、encode_*、call_raw、call_*、constructor 的 encode_* / instantiate_*。
/// 仅对 encoding 为 Codec 的 message/constructor 生成 encode 与调用封装。
pub fn gen_interface_module(
    message_fns: &[(ItemFn, [u8; 4], EncodingMode)],
    constructor_fn: Option<(&ItemFn, EncodingMode)>,
) -> TokenStream2 {
    let selector_consts: Vec<TokenStream2> = message_fns
        .iter()
        .map(|(f, sel, _)| {
            let name = f.sig.ident.to_string();
            let const_name = syn::Ident::new(
                &format!("SELECTOR_{}", to_selector_constant_name(&name)),
                f.sig.ident.span(),
            );
            let (a, b, c, d) = (sel[0], sel[1], sel[2], sel[3]);
            quote! {
                pub const #const_name: [u8; 4] = [#a, #b, #c, #d];
            }
        })
        .collect();

    let encode_fns: Vec<TokenStream2> = message_fns
        .iter()
        .filter(|(_, _, enc)| *enc == EncodingMode::Codec)
        .map(|(f, sel, _)| {
            let fn_name = &f.sig.ident;
            let mut input_vars = Vec::new();
            let mut arg_tys = Vec::new();
            for arg in &f.sig.inputs {
                let FnArg::Typed(pt) = arg else { continue };
                let name = match pt.pat.as_ref() {
                    Pat::Ident(pi) => pi.ident.clone(),
                    _ => continue,
                };
                let ty = pt.ty.as_ref();
                input_vars.push((name, quote! { #ty }));
                arg_tys.push(quote! { #ty });
            }
            let (a, b, c, d) = (sel[0], sel[1], sel[2], sel[3]);
            let encode_body = if input_vars.is_empty() {
                quote! {
                    let mut out = alloc::vec::Vec::with_capacity(4);
                    out.extend_from_slice(&[#a, #b, #c, #d]);
                    out
                }
            } else if input_vars.len() == 1 {
                let (arg_name, _) = &input_vars[0];
                quote! {
                    let mut out = alloc::vec::Vec::with_capacity(4 + 32);
                    out.extend_from_slice(&[#a, #b, #c, #d]);
                    out.extend_from_slice(&wrevive_api::Encode::encode(#arg_name));
                    out
                }
            } else {
                let arg_names: Vec<_> = input_vars.iter().map(|(n, _)| n).collect();
                quote! {
                    let mut out = alloc::vec::Vec::with_capacity(4 + 64);
                    out.extend_from_slice(&[#a, #b, #c, #d]);
                    out.extend_from_slice(&wrevive_api::Encode::encode(&(#(#arg_names),*)));
                    out
                }
            };
            let (params, _args): (TokenStream2, TokenStream2) = if input_vars.is_empty() {
                (quote! {}, quote! {})
            } else if input_vars.len() == 1 {
                let (arg_name, arg_ty) = &input_vars[0];
                (quote! { #arg_name: &#arg_ty }, quote! {})
            } else {
                let params: Vec<_> = input_vars
                    .iter()
                    .map(|(n, t)| quote! { #n: &#t })
                    .collect();
                (quote! { #(#params),* }, quote! {})
            };
            let encode_fn_ident = format_ident!("encode_{}", fn_name);
            quote! {
                #[inline(always)]
                pub fn #encode_fn_ident(#params) -> alloc::vec::Vec<u8> {
                    #encode_body
                }
            }
        })
        .collect();

    // 与 message 同名的调用函数：封装 encode + call_raw + decode，返回已解码的 message 返回值
    let call_fns: Vec<TokenStream2> = message_fns
        .iter()
        .filter(|(_, _, enc)| *enc == EncodingMode::Codec)
        .map(|(f, _, _)| {
            let fn_name = &f.sig.ident;
            let encode_fn_ident = format_ident!("encode_{}", fn_name);
            let mut input_vars = Vec::new();
            for arg in &f.sig.inputs {
                let FnArg::Typed(pt) = arg else { continue };
                let name = match pt.pat.as_ref() {
                    Pat::Ident(pi) => pi.ident.clone(),
                    _ => continue,
                };
                let ty = pt.ty.as_ref();
                input_vars.push((name, quote! { #ty }));
            }
            let (call_params, encode_invocation): (TokenStream2, TokenStream2) = if input_vars.is_empty() {
                (quote! { callee: &Address }, quote! { #encode_fn_ident() })
            } else if input_vars.len() == 1 {
                let (arg_name, arg_ty) = &input_vars[0];
                (
                    quote! { callee: &Address, #arg_name: &#arg_ty },
                    quote! { #encode_fn_ident(#arg_name) },
                )
            } else {
                let params: Vec<_> = input_vars
                    .iter()
                    .map(|(n, t)| quote! { #n: &#t })
                    .collect();
                let args: Vec<_> = input_vars.iter().map(|(n, _)| quote! { #n }).collect();
                (
                    quote! { callee: &Address, #(#params),* },
                    quote! { #encode_fn_ident(#(#args),*) },
                )
            };
            let ret_ty = match &f.sig.output {
                ReturnType::Default => quote! { () },
                ReturnType::Type(_, ty) => quote! { #ty },
            };
            quote! {
                /// 调用远端合约的该 message，返回已解码的返回值。
                #[inline(always)]
                pub fn #fn_name(#call_params) -> Result<#ret_ty, ReturnErrorCode> {
                    let input = #encode_invocation;
                    let raw = call_raw(callee, &input)?;
                    let mut cur = &raw[..];
                    <#ret_ty as wrevive_api::Decode>::decode(&mut cur)
                        .map_err(|_| ReturnErrorCode::CalleeTrapped)
                }
            }
        })
        .collect();

    // constructor：encode_<name>（无 selector）+ instantiate_<name>（封装 env::instantiate + 解码返回值）
    let (ctor_encode_fn, ctor_instantiate_fn): (TokenStream2, TokenStream2) =
        match constructor_fn {
            Some((ctor, enc)) if enc == EncodingMode::Codec => {
            let ctor_name = &ctor.sig.ident;
            let mut ctor_inputs = Vec::new();
            for arg in &ctor.sig.inputs {
                let FnArg::Typed(pt) = arg else { continue };
                let name = match pt.pat.as_ref() {
                    Pat::Ident(pi) => pi.ident.clone(),
                    _ => continue,
                };
                let ty = pt.ty.as_ref();
                ctor_inputs.push((name, quote! { #ty }));
            }
            let (encode_params, encode_body) = if ctor_inputs.is_empty() {
                (quote! {}, quote! { alloc::vec::Vec::new() })
            } else if ctor_inputs.len() == 1 {
                let (n, t) = &ctor_inputs[0];
                (
                    quote! { #n: &#t },
                    quote! { wrevive_api::Encode::encode(#n) },
                )
            } else {
                let params: Vec<TokenStream2> = ctor_inputs
                    .iter()
                    .map(|(n, t)| quote! { #n: &#t })
                    .collect();
                let args: Vec<TokenStream2> = ctor_inputs.iter().map(|(n, _)| quote! { #n }).collect();
                (
                    quote! { #(#params),* },
                    quote! { wrevive_api::Encode::encode(&(#(#args),*)) },
                )
            };
            let encode_fn_name = format_ident!("encode_{}", ctor_name);
            let ctor_encode = quote! {
                /// 编码 constructor 参数（无 selector），用于 env::instantiate 的 input_data。
                #[inline(always)]
                pub fn #encode_fn_name(#encode_params) -> alloc::vec::Vec<u8> {
                    #encode_body
                }
            };
            let ret_ty = match &ctor.sig.output {
                ReturnType::Default => quote! { () },
                ReturnType::Type(_, ty) => quote! { #ty },
            };
            // 用于 env::instantiate 的 code hash 使用独立参数名，避免与构造函数参数中的 code_hash 重名导致 E0415
            let instantiate_code_hash = format_ident!("__instantiate_code_hash");
            let (instantiate_params, encode_invocation): (TokenStream2, TokenStream2) =
                if ctor_inputs.is_empty() {
                    (
                        quote! {
                            #instantiate_code_hash: &wrevive_api::H256,
                            value: &wrevive_api::U256,
                            deposit: &wrevive_api::U256
                        },
                        quote! { #encode_fn_name() },
                    )
                } else if ctor_inputs.len() == 1 {
                    let (n, t) = &ctor_inputs[0];
                    (
                        quote! {
                            #instantiate_code_hash: &wrevive_api::H256,
                            #n: &#t,
                            value: &wrevive_api::U256,
                            deposit: &wrevive_api::U256
                        },
                        quote! { #encode_fn_name(#n) },
                    )
                } else {
                    let params: Vec<TokenStream2> = ctor_inputs
                        .iter()
                        .map(|(n, t)| quote! { #n: &#t })
                        .collect();
                    let args: Vec<TokenStream2> = ctor_inputs.iter().map(|(n, _)| quote! { #n }).collect();
                    (
                        quote! {
                            #instantiate_code_hash: &wrevive_api::H256,
                            #(#params),*,
                            value: &wrevive_api::U256,
                            deposit: &wrevive_api::U256
                        },
                        quote! { #encode_fn_name(#(#args),*) },
                    )
                };
            let instantiate_fn_name = format_ident!("instantiate_{}", ctor_name);
            let ctor_instantiate = quote! {
                /// 实例化合约（调用 constructor），返回 (新合约地址, 已解码的 constructor 返回值)。
                #[inline(always)]
                pub fn #instantiate_fn_name(#instantiate_params) -> Result<(wrevive_api::Address, #ret_ty), ReturnErrorCode> {
                    let input_data = #encode_invocation;
                    let mut addr = [0u8; 20];
                    let mut out_buf = [0u8; 256];
                    let mut out_slice = out_buf.as_mut_slice();
                    let mut cursor = &mut out_slice;
                    wrevive_api::env().instantiate(
                        pallet_revive_uapi::CallFlags::empty(),
                        #instantiate_code_hash.as_bytes(),
                        10_000_000,
                        10_000_000,
                        deposit.as_bytes(),
                        value.as_bytes(),
                        &input_data,
                        &mut addr,
                        Some(&mut cursor),
                    )?;
                    let ret = <#ret_ty as wrevive_api::Decode>::decode(&mut &out_buf[..])
                        .map_err(|_| ReturnErrorCode::CalleeTrapped)?;
                    Ok((wrevive_api::Address::from(addr), ret))
                }
            };
            (quote! { #ctor_encode }, quote! { #ctor_instantiate })
            }
            _ => (quote! {}, quote! {}),
        };

    quote! {
        /// 合约间调用接口：selector 常量、encode_*、call_raw、call_*、constructor 的 encode_* / instantiate_*。
        pub mod api {
            use super::*;
            use wrevive_api::{Address, Decode, Encode, ReturnErrorCode};

            #(#selector_consts)*

            /// 调用指定合约，传入已编码的 input（4 字节 selector + 编码参数），返回合约返回的原始字节。
            #[inline(always)]
            pub fn call_raw(
                callee: &Address,
                input: &[u8],
            ) -> Result<alloc::vec::Vec<u8>, ReturnErrorCode> {
                let r = wrevive_api::env().call(
                    pallet_revive_uapi::CallFlags::empty(),
                    callee,
                    10_000_000,
                    10_000_000,
                    &wrevive_api::U256::ZERO,
                    &wrevive_api::U256::ZERO,
                    input,
                    None,
                );
                r.map_err(|e| e)?;
                let size = wrevive_api::env().return_data_size() as usize;
                let mut buf = alloc::vec![0u8; size];
                let mut slice = buf.as_mut_slice();
                wrevive_api::env().return_data_copy(&mut slice, 0);
                Ok(buf)
            }

            #(#encode_fns)*

            #(#call_fns)*

            #ctor_encode_fn

            #ctor_instantiate_fn
        }
    }
}
