//! #[revive_contract] 主宏：解析 mod、生成 deploy/call、写入 ABI、生成合约间调用 interface。

use crate::abi;
use crate::attrs;
use crate::codegen;
use crate::interface;
use crate::manifest;
use crate::prefix;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_macro_input;
use syn::{FnArg, Item, ItemFn, Pat, ReturnType};

    /// Implementation logic called by proc_macro_attribute in lib.rs.
    /// 由 lib.rs 的 proc_macro_attribute 调用的实现逻辑。
    /// 
    /// # English
    /// Main entry point for the #[revive_contract] macro.
    /// Parses the module, validates constructors and messages,
    /// generates deploy/call entry points, and writes ABI.
    /// 
    /// # 中文
    /// #[revive_contract] 宏的主入口点。
    /// 解析模块，验证构造函数和消息，
    /// 生成 deploy/call 入口点，并写入 ABI。
pub fn revive_contract_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let encoding_mode = attrs::parse_encoding_from_contract_attr(&attr);

    let contract_name = std::env::var("CARGO_BIN_NAME")
        .or_else(|_| std::env::var("CONTRACT_NAME"))
        .unwrap_or_else(|_| manifest::bin_name_from_manifest().unwrap_or_else(|| "contract".into()));

    let item = parse_macro_input!(item as Item);
    let Item::Mod(mut module) = item else {
        return syn::Error::new_spanned(
            item,
            "revive_contract must be applied to a mod / revive_contract 只能用于 mod",
        )
        .to_compile_error()
        .into();
    };

    let mod_name = module.ident.clone();
    let mod_content = match &mut module.content {
        Some((_, items)) => items,
        None => {
            return syn::Error::new_spanned(
                module,
                "revive_contract mod must have a body / revive_contract mod 必须有 body",
            )
            .to_compile_error()
            .into();
        }
    };

    let prefix_to_names: std::collections::HashMap<Vec<u8>, Vec<String>> = mod_content
        .iter()
        .filter_map(prefix::prefixes_from_item)
        .flat_map(|pairs| pairs.into_iter())
        .fold(std::collections::HashMap::new(), |mut acc, (prefix, name)| {
            acc.entry(prefix).or_default().push(name);
            acc
        });
    for (prefix, names) in &prefix_to_names {
        if names.len() > 1 {
            let msg = format!(
                "duplicate storage prefix: {:?} used by: {} (Storage/Mapping/List/List2D prefix must be unique / 存储前缀重复，被以下项使用，前缀必须唯一)",
                prefix,
                names.join(", ")
            );
            return syn::Error::new_spanned(&module, msg)
                .to_compile_error()
                .into();
        }
    }

    let mut constructor_fn: Option<(ItemFn, attrs::EncodingMode)> = None;
    let mut message_fns: Vec<(ItemFn, [u8; 4], attrs::EncodingMode, bool)> = Vec::new();
    let mut fallback_fn: Option<ItemFn> = None;
    let mut other_items: Vec<Item> = Vec::new();

    for item in std::mem::take(mod_content) {
        match item {
            Item::Fn(mut f) => {
                let is_fallback = attrs::is_revive_fallback(&f.attrs);
                let is_constructor = attrs::is_revive_constructor(&f.attrs);
                let selector = attrs::parse_selector_from_attrs(&f.attrs);
                if is_fallback {
                    if selector.is_some() {
                        return syn::Error::new_spanned(
                            &f.sig,
                            "#[revive(fallback)] must not define selector / fallback 不能指定 selector",
                        )
                        .to_compile_error()
                        .into();
                    }
                    if fallback_fn.is_some() {
                        return syn::Error::new_spanned(
                            &f.sig,
                            "at most one #[revive(fallback)] function allowed / 最多只能定义一个 fallback 函数",
                        )
                        .to_compile_error()
                        .into();
                    }
                    if !f.sig.inputs.is_empty() {
                        return syn::Error::new_spanned(
                            &f.sig,
                            "#[revive(fallback)] must have no parameters / fallback 必须无参数",
                        )
                        .to_compile_error()
                        .into();
                    }
                    if !matches!(&f.sig.output, ReturnType::Default) {
                        return syn::Error::new_spanned(
                            &f.sig,
                            "#[revive(fallback)] must have no return value / fallback 必须无返回值",
                        )
                        .to_compile_error()
                        .into();
                    }
                    f.attrs = attrs::strip_revive_attrs(&f.attrs);
                    fallback_fn = Some(f);
                } else if is_constructor {
                    let enc = attrs::parse_encoding_from_fn_attrs(&f.attrs, encoding_mode);
                    f.attrs = attrs::strip_revive_attrs(&f.attrs);
                    constructor_fn = Some((f, enc));
                } else if attrs::is_revive_message(&f.attrs) {
                    let enc = attrs::parse_encoding_from_fn_attrs(&f.attrs, encoding_mode);
                    let explicit_mutates = attrs::has_revive_mutates(&f.attrs);
                    let sel = selector
                        .unwrap_or_else(|| attrs::selector_from_name(&f.sig.ident.to_string()));
                    f.attrs = attrs::strip_revive_attrs(&f.attrs);
                    message_fns.push((f, sel, enc, explicit_mutates));
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
            return syn::Error::new_spanned(
                &module,
                "exactly one #[revive(constructor)] function required / 需要恰好一个 #[revive(constructor)] 函数",
            )
            .to_compile_error()
            .into();
        }
    };
    let constructor_name = constructor_fn.sig.ident.clone();

    // 智能合约必须有执行结果：构造函数和所有 message 函数都必须有返回值
    if matches!(&constructor_fn.sig.output, ReturnType::Default) {
        return syn::Error::new_spanned(
            &constructor_fn.sig,
            "constructor must have a return value (e.g. Self or Result<..., E>) / 构造函数必须有返回值，智能合约必须有执行结果",
        )
        .to_compile_error()
        .into();
    }
    for (f, _, _, _) in &message_fns {
        if matches!(&f.sig.output, ReturnType::Default) {
            return syn::Error::new_spanned(
                &f.sig,
                format!(
                    "message function `{}` must have a return value / message 函数必须有返回值，智能合约必须有执行结果",
                    f.sig.ident
                ),
            )
            .to_compile_error()
            .into();
        }
    }

    let message_fns_abi: Vec<(ItemFn, [u8; 4], bool)> =
        message_fns.iter().map(|(f, sel, _, explicit_mutates)| (f.clone(), *sel, *explicit_mutates)).collect();
    if let Err(e) = abi::emit_abi(&contract_name, &constructor_fn, &message_fns_abi, &other_items) {
        return syn::Error::new_spanned(
            &module,
            format!("ABI generation failed, build aborted: {0} / ABI 生成失败，编译终止: {0}", e),
        )
        .to_compile_error()
        .into();
    }

    let mut constructor_input_vars: Vec<(syn::Ident, TokenStream2)> = Vec::new();
    let mut constructor_call_exprs = Vec::new();
    for arg in &constructor_fn.sig.inputs {
        let FnArg::Typed(pt) = arg else { continue };
        let name = match pt.pat.as_ref() {
            Pat::Ident(pi) => pi.ident.clone(),
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
            // 与 call() 一致：链上传入 selector(4 字节) + 编码参数，需跳过 selector 再解码
            if __input.len() < 4 {
                wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                return;
            }
            let __input = &__input[4..];
        };
        match constructor_encoding {
            attrs::EncodingMode::Codec => {
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
            attrs::EncodingMode::Sol => {
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

    let constructor_return_encode = match &constructor_fn.sig.output {
        ReturnType::Default => quote! {
            wrevive_api::env().return_value(wrevive_api::ReturnFlags::empty(), &[]);
        },
        ReturnType::Type(_, ty) => {
            let is_result = codegen::unwrap_result_or_option(ty)
                .map(|(_, r)| r)
                .unwrap_or(false);
            let flags = if is_result {
                quote! { if let Err(_) = &__ret { wrevive_api::ReturnFlags::REVERT } else { wrevive_api::ReturnFlags::empty() } }
            } else {
                quote! { wrevive_api::ReturnFlags::empty() }
            };
            match constructor_encoding {
                attrs::EncodingMode::Codec => quote! {
                    let __encoded = wrevive_api::Encode::encode(&__ret);
                    wrevive_api::env().return_value(#flags, &__encoded);
                },
                attrs::EncodingMode::Sol => quote! {
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
        /// Contract deployment entry function
        /// 
        /// This function is automatically generated by #[revive_contract] macro to:
        /// 1. Decode deployment parameters (selector + SCALE encoded constructor arguments)
        /// 2. Call user-defined constructor
        /// 3. Encode constructor return value and set it to contract return data
        /// 
        /// # Parameter Format
        /// - input_data: selector(4 bytes) + SCALE encoded constructor parameters
        /// 
        /// # Return Values
        /// - Success: SCALE encoded constructor return value
        /// - Failure: REVERT flag + error information
        #[polkavm_derive::polkavm_export]
        #[allow(unreachable_code)]
        pub extern "C" fn deploy() {
            #deploy_body
        }
    })
    .unwrap();

    // 保持 mod 内顺序：use/const 等在前，再 fallback/constructor/message/interface
    mod_content.extend(other_items);
    if let Some(fb) = &fallback_fn {
        mod_content.push(Item::Fn(fb.clone()));
    }
    mod_content.push(Item::Fn(constructor_fn.clone()));
    for (f, _, _, _) in &message_fns {
        mod_content.push(Item::Fn(f.clone()));
    }

    // 生成合约间调用接口子模块：SELECTOR_*、encode_*、call_raw、constructor 的 encode_* / instantiate_*
    let interface_ts = interface::gen_interface_module(&message_fns, Some((&constructor_fn, constructor_encoding)));
    let interface_item: Item = syn::parse2(interface_ts).expect("interface code generation failed / interface 代码生成失败");
    mod_content.push(interface_item);

    let match_arms: Vec<TokenStream2> = message_fns
        .iter()
        .map(|(f, sel, fn_enc, _)| {
            let fn_name = &f.sig.ident;
            let sig = &f.sig;
            let min_len: usize = 4;

            let mut input_vars: Vec<(syn::Ident, TokenStream2)> = Vec::new();
            let mut call_exprs = Vec::new();
            for arg in &sig.inputs {
                let FnArg::Typed(pt) = arg else { continue };
                let name = match pt.pat.as_ref() {
                    Pat::Ident(pi) => pi.ident.clone(),
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
                    attrs::EncodingMode::Codec => {
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
                    attrs::EncodingMode::Sol => {
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
            let encode_and_return = codegen::return_encode(ret_ty, *fn_enc);
            let sel_u32 = u32::from_be_bytes(*sel);
            let arm_inner = if min_len == 4 {
                quote! {
                    #input_parse
                    let __ret = #mod_name::#fn_name(#(#call_exprs),*);
                    #encode_and_return
                }
            } else {
                quote! {
                    if __input_len >= #min_len {
                        #input_parse
                        let __ret = #mod_name::#fn_name(#(#call_exprs),*);
                        #encode_and_return
                    } else {
                        wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]);
                    }
                }
            };
            quote! {
                #sel_u32 => {
                    #arm_inner
                },
            }
        })
        .collect();

    let fallback_ident = fallback_fn.as_ref().map(|f| f.sig.ident.clone());
    let unknown_selector_arm: TokenStream2 = match &fallback_ident {
        Some(fb) => quote! {
            _ => {
                #mod_name::#fb();
                return;
            },
        },
        None => quote! {
            _ => wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]),
        },
    };
    let too_short_input: TokenStream2 = match &fallback_ident {
        Some(fb) => quote! {
            #mod_name::#fb();
            return;
        },
        None => quote! {
            wrevive_api::env().return_value(wrevive_api::ReturnFlags::REVERT, &[]);
        },
    };

    let call_fn: Item = syn::parse2(quote! {
        /// Contract call entry function (message dispatcher)
        /// 
        /// This function is automatically generated by #[revive_contract] macro to:
        /// 1. Read call data (selector + SCALE encoded arguments)
        /// 2. Dispatch to corresponding message function based on selector
        /// 3. Decode arguments and call user-defined message function
        /// 4. Encode return value and set it to contract return data
        /// 
        /// # Parameter Format
        /// - input_data: selector(4 bytes) + SCALE encoded message function arguments
        /// 
        /// # Selector Dispatch Logic
        /// - Each message function has a unique 4-byte selector
        /// - Selector is generated using first 4 bytes of BLAKE2s256 hash of function name
        /// - Supports explicit selector specification: `#[revive(message, selector = 0x...)]`
        /// 
        /// # Return Values
        /// - Success: SCALE encoded message function return value
        /// - Failure: REVERT flag + error information
        /// - Unknown selector: call fallback function or directly REVERT
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
            if __input_len < 4 {
                #too_short_input
            }
            let __sel = u32::from_be_bytes([__input[0], __input[1], __input[2], __input[3]]);
            match __sel {
                #(#match_arms)*
                #unknown_selector_arm
            }
        }
    })
    .unwrap();

    let any_sol = constructor_encoding == attrs::EncodingMode::Sol
        || message_fns
            .iter()
            .any(|(_, _, enc, _)| *enc == attrs::EncodingMode::Sol);
    let use_encoding: TokenStream2 = if any_sol {
        quote! {
            use wrevive_api::Decode;
            use pvm_contract_types::{SolDecode, SolEncode};
        }
    } else {
        quote! { use wrevive_api::Decode; }
    };

    quote! {
        #use_encoding

        #module

        #deploy_fn

        #call_fn
    }
    .into()
}
