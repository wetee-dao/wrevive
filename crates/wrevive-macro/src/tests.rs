#[cfg(test)]
mod tests {
    use super::docs;
    use syn::parse_quote;

    #[test]
    fn test_extract_docs() {
        // 创建带注释的函数属性
        let attrs: Vec<syn::Attribute> = vec![
            parse_quote! {
                /// 这是一个测试函数
                /// 
                /// # Parameters
                /// @param value 测试值
                /// @param value test value
            }
        ];

        let docs = docs::extract_docs(&attrs);
        assert_eq!(docs.len(), 5);
        assert!(docs[0].contains("这是一个测试函数"));
        assert!(docs[2].contains("@param value"));
    }

    #[test]
    fn test_parse_param_docs() {
        let docs = vec![
            "函数说明".to_string(),
            "@param value 测试值".to_string(),
            "@param value test value".to_string(),
            "@param other 其他值".to_string(),
        ];

        let param_docs = docs::parse_param_docs(&docs);
        assert_eq!(param_docs.get("value"), Some(&"测试值".to_string()));
        assert_eq!(param_docs.get("other"), Some(&"其他值".to_string()));
    }

    #[test]
    fn test_to_go_comment() {
        let docs = vec!["这是一个测试函数".to_string()];
        let mut param_docs = std::collections::HashMap::new();
        param_docs.insert("value".to_string(), "测试值".to_string());
        let return_doc = Some("返回结果".to_string());

        let go_comment = docs::to_go_comment(&docs, "test_func", &param_docs, &return_doc);
        assert!(go_comment.contains("// test_func"));
        assert!(go_comment.contains("// 这是一个测试函数"));
        assert!(go_comment.contains("// @param value 测试值"));
        assert!(go_comment.contains("// @return 返回结果"));
    }

    #[test]
    fn test_to_rust_comment() {
        let docs = vec!["这是一个测试函数".to_string(), "多行注释".to_string()];
        let rust_comment = docs::to_rust_comment(&docs);
        assert!(rust_comment.contains("/// 这是一个测试函数"));
        assert!(rust_comment.contains("/// 多行注释"));
    }
}