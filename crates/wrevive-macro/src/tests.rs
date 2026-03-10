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

}