//! call() 内参数字节布局与返回值编码（与 ABI 约定一致）。

use crate::attrs::EncodingMode;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{GenericArgument, PathArguments, ReturnType, Type};

/// 若返回类型为 Result<T,E> 或 Option<T>，返回 Some((内层类型 T, true=Result/false=Option))；否则返回 None。
pub fn unwrap_result_or_option(ty: &Type) -> Option<(&Type, bool)> {
    let path = match ty {
        Type::Path(p) => &p.path,
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
pub fn is_unit_type(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(t) if t.elems.is_empty())
}

/// 生成将返回值 __ret 编码并传给 env().return_value 的代码。
pub fn return_encode(ret_ty: &ReturnType, encoding: EncodingMode) -> TokenStream2 {
    match ret_ty {
        ReturnType::Default => quote! {
            wrevive_api::env().return_value(wrevive_api::ReturnFlags::empty(), &[]);
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
                        // On error: revert with empty data.
                        // For proper Solidity error encoding, use SolError/SolRevert from pvm_contract_types.
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
