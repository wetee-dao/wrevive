//! Documentation processing module: extract and process doc comments from Rust source.
//! 注释处理模块：从 Rust 源码中提取和处理文档注释。
//!
//! Provides functionality to extract doc comments from function attributes,
//! parse parameter docs, and generate documentation in various formats.
//! 提供从函数属性中提取文档注释、解析参数文档、以及生成各种格式文档注释的功能。

use std::collections::HashMap;
use syn::{Attribute, ExprLit, Lit, Meta, MetaNameValue};

/// Extracts doc comments from function attributes.
/// 从函数属性中提取文档注释。
///
/// # Arguments / 参数
/// * `attrs` - Function attribute list / 函数的属性列表
///
/// # Returns / 返回值
/// Returns a vector of doc comment strings in original order.
/// 返回文档注释字符串的向量，保持原始顺序。
pub fn extract_docs(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| {
            if let Meta::NameValue(MetaNameValue { 
                value: syn::Expr::Lit(ExprLit { 
                    lit: Lit::Str(lit_str), .. 
                }), .. 
            }) = &attr.meta {
                Some(lit_str.value())
            } else {
                None
            }
        })
        .collect()
}

/// Regex for matching parameter documentation.
/// 参数文档匹配的正则表达式。
///
/// Matches format: `@param param_name description`
/// 匹配格式：`@param param_name description`
static PARAM_DOC_REGEX: once_cell::sync::Lazy<regex::Regex> = 
    once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"@param\s+(\w+)\s+(.+)").unwrap()
    });

/// Parses parameter documentation from doc comments.
/// 从文档注释中解析参数文档。
///
/// Supports `@param param_name description` format.
/// 支持 `@param param_name description` 格式的参数文档。
///
/// # Arguments / 参数
/// * `docs` - List of doc comment strings / 文档注释字符串列表
///
/// # Returns / 返回值
/// Returns a map from parameter name to description.
/// 返回参数名到描述的映射。
pub fn parse_param_docs(docs: &[String]) -> HashMap<String, String> {
    let mut param_docs = HashMap::new();
    for doc in docs {
        if let Some(captures) = PARAM_DOC_REGEX.captures(doc) {
            let param_name = captures.get(1).unwrap().as_str().to_string();
            let description = captures.get(2).unwrap().as_str().to_string();
            param_docs.insert(param_name, description);
        }
    }
    param_docs
}

/// Cleans and formats doc comments.
/// 清理和格式化文档注释。
///
/// Removes excess whitespace and unifies documentation format.
/// 移除多余的空白字符，统一文档格式。
///
/// # Arguments / 参数
/// * `docs` - List of raw doc comment strings / 原始文档注释字符串列表
///
/// # Returns / 返回值
/// Returns the cleaned list of doc comment strings.
/// 返回清理后的文档注释字符串列表。
pub fn clean_docs(docs: &[String]) -> Vec<String> {
    docs.iter()
        .map(|doc| {
            // 移除首尾空白字符
            let cleaned = doc.trim();
            // 如果是空行，保持为空
            if cleaned.is_empty() {
                String::new()
            } else {
                cleaned.to_string()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_docs() {
        let attrs = vec![
            parse_quote!(#[doc = "这是第一行注释"]),
            parse_quote!(#[doc = "这是第二行注释"]),
            parse_quote!(#[derive(Debug)]), // 不是 doc 属性
        ];
        
        let docs = extract_docs(&attrs);
        assert_eq!(docs, vec!["这是第一行注释", "这是第二行注释"]);
    }

    #[test]
    fn test_parse_param_docs() {
        let docs = vec![
            "设置合约的所有者".to_string(),
            "@param new_owner 新的所有者地址".to_string(),
            "@param reason 修改原因".to_string(),
            "只有管理员可以调用".to_string(),
        ];
        
        let param_docs = parse_param_docs(&docs);
        assert_eq!(param_docs.get("new_owner"), Some(&"新的所有者地址".to_string()));
        assert_eq!(param_docs.get("reason"), Some(&"修改原因".to_string()));
        assert_eq!(param_docs.len(), 2);
    }

}
