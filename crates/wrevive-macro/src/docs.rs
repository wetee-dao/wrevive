//! 注释处理模块：从 Rust 源码中提取和处理文档注释
//! 
//! 此模块提供了从函数属性中提取文档注释、解析参数文档、
//! 以及生成各种格式的文档注释的功能。

use std::collections::HashMap;
use syn::{Attribute, ExprLit, Lit, Meta, MetaNameValue};

/// 从函数属性中提取文档注释
/// 
/// # 参数
/// * `attrs` - 函数的属性列表
/// 
/// # 返回值
/// 返回文档注释字符串的向量，保持原始顺序
/// 
/// # 示例
/// ```rust,ignore
/// use syn::parse_quote;
/// let attrs = vec![
///     parse_quote!(#[doc = "这是第一行注释"]),
///     parse_quote!(#[doc = "这是第二行注释"]),
/// ];
/// let docs = extract_docs(&attrs);
/// assert_eq!(docs, vec!["这是第一行注释", "这是第二行注释"]);
/// ```
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

/// 参数文档匹配的正则表达式
/// 匹配格式：@param param_name description
static PARAM_DOC_REGEX: once_cell::sync::Lazy<regex::Regex> = 
    once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"@param\s+(\w+)\s+(.+)").unwrap()
    });

/// 返回值文档匹配的正则表达式
/// 匹配格式：@return description 或 @returns description
static RETURN_DOC_REGEX: once_cell::sync::Lazy<regex::Regex> = 
    once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"@returns?\s+(.+)").unwrap()
    });

/// 从文档注释中解析参数文档
/// 
/// 支持 @param param_name description 格式的参数文档
/// 
/// # 参数
/// * `docs` - 文档注释字符串列表
/// 
/// # 返回值
/// 返回参数名到描述的映射
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

/// 从文档注释中解析返回值文档
/// 
/// 支持 @return description 或 @returns description 格式的返回值文档
/// 
/// # 参数
/// * `docs` - 文档注释字符串列表
/// 
/// # 返回值
/// 返回返回值描述字符串，如果没有找到则返回 None
pub fn parse_return_docs(docs: &[String]) -> Option<String> {
    for doc in docs {
        if let Some(captures) = RETURN_DOC_REGEX.captures(doc) {
            return Some(captures.get(1).unwrap().as_str().to_string());
        }
    }
    None
}

/// 清理和格式化文档注释
/// 
/// 移除多余的空白字符，统一文档格式
/// 
/// # 参数
/// * `docs` - 原始文档注释字符串列表
/// 
/// # 返回值
/// 返回清理后的文档注释字符串列表
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

/// 将文档注释转换为 Go 风格的注释
/// 
/// # 参数
/// * `docs` - 文档注释字符串列表
/// * `func_name` - 函数名
/// * `param_docs` - 参数文档映射
/// * `return_doc` - 返回值文档
/// 
/// # 返回值
/// 返回 Go 风格的注释字符串
pub fn to_go_comment(
    docs: &[String], 
    func_name: &str,
    param_docs: &HashMap<String, String>,
    return_doc: &Option<String>
) -> String {
    let mut comment = String::new();
    
    // 添加函数总览注释
    if !docs.is_empty() {
        for doc in docs {
            comment.push_str(&format!("// {}\n", doc));
        }
    } else {
        comment.push_str(&format!("// {} 是一个自动生成的合约调用函数\n", func_name));
    }
    
    // 添加参数注释
    for (param_name, description) in param_docs {
        comment.push_str(&format!("// {}: {}\n", param_name, description));
    }
    
    // 添加返回值注释
    if let Some(return_desc) = return_doc {
        comment.push_str(&format!("// 返回: {}\n", return_desc));
    }
    
    comment
}

/// 将文档注释转换为 Rust 风格的注释
/// 
/// # 参数
/// * `docs` - 文档注释字符串列表
/// 
/// # 返回值
/// 返回 Rust 风格的注释字符串
pub fn to_rust_comment(docs: &[String]) -> String {
    if docs.is_empty() {
        return String::new();
    }
    
    let mut comment = String::new();
    for doc in docs {
        comment.push_str(&format!("/// {}\n", doc));
    }
    comment
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

    #[test]
    fn test_parse_return_docs() {
        let docs1 = vec![
            "设置合约的所有者".to_string(),
            "@return 设置成功返回 true".to_string(),
        ];
        
        let docs2 = vec![
            "获取合约余额".to_string(),
            "@returns 当前余额值".to_string(),
        ];
        
        assert_eq!(parse_return_docs(&docs1), Some("设置成功返回 true".to_string()));
        assert_eq!(parse_return_docs(&docs2), Some("当前余额值".to_string()));
    }


    #[test]
    fn test_to_go_comment() {
        let docs = vec!["设置合约的所有者".to_string()];
        let mut param_docs = HashMap::new();
        param_docs.insert("new_owner".to_string(), "新的所有者地址".to_string());
        let return_doc = Some("操作成功".to_string());
        
        let comment = to_go_comment(&docs, "SetOwner", &param_docs, &return_doc);
        assert!(comment.contains("// 设置合约的所有者"));
        assert!(comment.contains("// new_owner: 新的所有者地址"));
        assert!(comment.contains("// 返回: 操作成功"));
    }

    #[test]
    fn test_to_rust_comment() {
        let docs = vec!["设置合约的所有者".to_string(), "只有管理员可以调用".to_string()];
        let comment = to_rust_comment(&docs);
        assert!(comment.contains("/// 设置合约的所有者"));
        assert!(comment.contains("/// 只有管理员可以调用"));
    }
}
