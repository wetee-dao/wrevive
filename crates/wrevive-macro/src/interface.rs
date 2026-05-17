//! 合约间调用接口生成：selector 常量、encode_*、call_raw、call_*、constructor 的 encode_* 与 instantiate_*。

use crate::attrs::{self, EncodingMode};
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
    message_fns: &[(ItemFn, [u8; 4], EncodingMode, bool)],
    constructor_fn: Option<(&ItemFn, EncodingMode)>,
) -> TokenStream2 {
    // 构造函数 encode_* 必须与 deploy() 一致：4 字节 selector + SCALE 参数（contract.rs 跳过 __input[..4]）。
    let mut ctor_selector_const: TokenStream2 = quote! {};
    let selector_consts: Vec<TokenStream2> = message_fns
        .iter()
        .map(|(f, sel, _, _)| {
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
        .filter(|(_, _, enc, _)| *enc == EncodingMode::Codec)
        .map(|(f, sel, _, _)| {
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
                let params: Vec<_> = input_vars.iter().map(|(n, t)| quote! { #n: &#t }).collect();
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
        .filter(|(_, _, enc, _)| *enc == EncodingMode::Codec)
        .map(|(f, _, _, _)| {
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
            let (call_params, encode_invocation): (TokenStream2, TokenStream2) = if input_vars
                .is_empty()
            {
                (quote! { callee: &Address }, quote! { #encode_fn_ident() })
            } else if input_vars.len() == 1 {
                let (arg_name, arg_ty) = &input_vars[0];
                (
                    quote! { callee: &Address, #arg_name: &#arg_ty },
                    quote! { #encode_fn_ident(#arg_name) },
                )
            } else {
                let params: Vec<_> = input_vars.iter().map(|(n, t)| quote! { #n: &#t }).collect();
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
                /// Call the remote contract's message, returns decoded return value.
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

    // constructor：encode_<name>（selector + SCALE，与 deploy 入口一致）+ instantiate_<name>
    let (ctor_encode_fn, ctor_instantiate_fn): (TokenStream2, TokenStream2) = match constructor_fn {
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
            } else {
                let sel = attrs::selector_from_name(&ctor_name.to_string());
                let (sa, sb, sc, sd) = (sel[0], sel[1], sel[2], sel[3]);
                let const_ident = syn::Ident::new(
                    &format!(
                        "SELECTOR_{}",
                        to_selector_constant_name(&ctor_name.to_string())
                    ),
                    ctor_name.span(),
                );
                ctor_selector_const = quote! {
                    /// Constructor selector (Keccak-256 of function name; same 4 bytes `deploy()` skips before decoding).
                    pub const #const_ident: [u8; 4] = [#sa, #sb, #sc, #sd];
                };
                if ctor_inputs.len() == 1 {
                    let (n, t) = &ctor_inputs[0];
                    (
                        quote! { #n: &#t },
                        quote! {
                            let mut out = alloc::vec::Vec::new();
                            out.extend_from_slice(&[#sa, #sb, #sc, #sd]);
                            out.extend_from_slice(&wrevive_api::Encode::encode(#n));
                            out
                        },
                    )
                } else {
                    let params: Vec<TokenStream2> = ctor_inputs
                        .iter()
                        .map(|(n, t)| quote! { #n: &#t })
                        .collect();
                    let args: Vec<TokenStream2> =
                        ctor_inputs.iter().map(|(n, _)| quote! { #n }).collect();
                    (
                        quote! { #(#params),* },
                        quote! {
                            let mut out = alloc::vec::Vec::new();
                            out.extend_from_slice(&[#sa, #sb, #sc, #sd]);
                            out.extend_from_slice(
                                &wrevive_api::Encode::encode(&(#(#args),*)),
                            );
                            out
                        },
                    )
                }
            };
            let encode_fn_name = format_ident!("encode_{}", ctor_name);
            let ctor_encode = quote! {
                /// Encode constructor call data: 4-byte selector + SCALE args (matches `deploy()` input after `#[revive_contract]` codegen).
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
                            deposit_limit: &wrevive_api::U256,
                            value: &wrevive_api::U256
                        },
                        quote! { #encode_fn_name() },
                    )
                } else if ctor_inputs.len() == 1 {
                    let (n, t) = &ctor_inputs[0];
                    (
                        quote! {
                            #instantiate_code_hash: &wrevive_api::H256,
                            #n: &#t,
                            deposit_limit: &wrevive_api::U256,
                            value: &wrevive_api::U256
                        },
                        quote! { #encode_fn_name(#n) },
                    )
                } else {
                    let params: Vec<TokenStream2> = ctor_inputs
                        .iter()
                        .map(|(n, t)| quote! { #n: &#t })
                        .collect();
                    let args: Vec<TokenStream2> =
                        ctor_inputs.iter().map(|(n, _)| quote! { #n }).collect();
                    (
                        quote! {
                            #instantiate_code_hash: &wrevive_api::H256,
                            #(#params),*,
                            deposit_limit: &wrevive_api::U256,
                            value: &wrevive_api::U256
                        },
                        quote! { #encode_fn_name(#(#args),*) },
                    )
                };
            let instantiate_fn_name = format_ident!("instantiate_{}", ctor_name);
            let ctor_instantiate = quote! {
                /// Instantiate contract (call constructor), returns (new contract address, decoded constructor return value).
                ///
                /// Note: `deposit_limit` is the storage deposit limit. Pass `U256::MAX` for "no specific limit" (recommended default),
                /// aligning with pallet-revive examples.
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
                        u64::MAX,
                        u64::MAX,
                        deposit_limit.as_bytes(),
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
        /// Contract-to-contract call interface module
        ///
        /// This module is automatically generated by #[revive_contract] macro to provide:
        /// 1. SELECTOR_* constants: 4-byte selectors for each message function
        /// 2. encode_* functions: encode function arguments to SCALE format
        /// 3. call_raw function: low-level contract call interface
        /// 4. call_* functions: convenient high-level call interfaces (encode+call+decode)
        /// 5. constructor's encode_* / instantiate_* functions
        ///
        /// # Usage Examples
        /// ```rust
        /// use crate::contract::api;
        ///
        /// // Method 1: Use high-level interface (recommended)
        /// let result = api::set_value(&callee_address, &42)?;
        ///
        /// // Method 2: Manual encoding + call
        /// let input = api::encode_set_value(&42);
        /// let raw_result = api::call_raw(&callee_address, &input)?;
        /// let decoded_result: Result<(), Error> = Decode::decode(&mut &raw_result[..])?;
        /// ```
        ///
        /// # Notes
        /// - Only functions using Codec encoding will generate encode_* and call_* functions
        /// - Sol encoded functions require manual encoding/decoding handling
        /// - Failed calls return ReturnErrorCode, which needs to be handled
        pub mod api {
            use super::*;
            use wrevive_api::*;

            /// Message function selector constants
            ///
            /// Each constant corresponds to a 4-byte selector of a message function,
            /// used to construct contract call input data.
            #(#selector_consts)*
            #ctor_selector_const

            /// Low-level contract call function
            ///
            /// Calls the specified contract with encoded input (4-byte selector + encoded parameters),
            /// returns the raw bytes from the contract.
            ///
            /// # Parameters
            /// * `callee` - Target contract address
            /// * `input` - Encoded call data (selector + parameter encoding)
            ///
            /// # Return Value
            /// Returns the raw return bytes from the contract, needs manual decoding
            ///
            /// # Error Handling
            /// Returns ReturnErrorCode on call failure
            #[inline(always)]
            pub fn call_raw(
                callee: &Address,
                input: &[u8],
            ) -> Result<Vec<u8>, ReturnErrorCode> {
                let r = wrevive_api::env().call(
                    pallet_revive_uapi::CallFlags::empty(),
                    callee,
                    u64::MAX,
                    u64::MAX,
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

            /// Parameter encoding functions
            ///
            /// For each message function using Codec encoding, generates corresponding encode_* functions,
            /// used to encode function arguments into SCALE format byte sequences.
            #(#encode_fns)*

            /// High-level call functions
            ///
            /// For each message function using Codec encoding, generates corresponding call_* functions,
            /// providing one-stop interfaces for encoding, calling, and decoding.
            #(#call_fns)*

            /// Constructor parameter encoding functions
            #ctor_encode_fn

            /// Contract instantiation functions
            ///
            /// Used to deploy new contract instances, handling constructor calls and return value decoding
            #ctor_instantiate_fn
        }
    }
}
