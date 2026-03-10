# 代码生成注释优化需求文档

## 需求背景
wrevive 项目是一个 Rust 合约工具包，提供 ink! 风格的过程宏用于 PolkaVM (pallet-revive) 合约开发。当前代码生成功能已经完善，但生成的代码注释不够详细，影响了代码的可读性和维护性。

## 问题分析
通过对当前代码生成的分析，发现以下注释不够完善的地方：

### 1. ABI 生成注释问题
- **当前状态**: ABI JSON 中的 `docs` 字段为空数组 `"docs": []`
- **影响**: 生成的 ABI 缺少函数和参数的说明文档，导致前端工具链无法生成有意义的 API 文档

### 2. Go 代码生成注释问题
- **当前状态**: 生成的 Go 调用代码缺少函数级别的注释
- **影响**: 开发者使用生成的 Go SDK 时，无法了解每个函数的用途、参数含义和返回值说明

### 3. Rust 生成代码注释问题
- **当前状态**: 通过 `#[revive_contract]` 宏生成的 `deploy()` 和 `call()` 函数缺少详细注释
- **影响**: 合约开发者难以理解生成代码的内部逻辑

## 需求场景具体处理逻辑

### 场景1: 从源码注释提取到 ABI
当 Rust 合约中的函数带有文档注释时，需要提取这些注释并生成到 ABI 的 `docs` 字段中。

**示例源码**:
```rust
/// 设置合约的所有者地址
/// 只有当前所有者可以调用此函数
/// @param new_owner 新的所有者地址
#[revive(message, write)]
pub fn set_owner(new_owner: Address) -> Result<(), Error> {
    // ...
}
```

**期望的 ABI 输出**:
```json
{
  "args": [
    {
      "label": "new_owner",
      "type": {...},
      "docs": ["新的所有者地址"]
    }
  ],
  "docs": [
    "设置合约的所有者地址",
    "只有当前所有者可以调用此函数"
  ]
}
```

### 场景2: Go 代码函数注释生成
基于 Rust 源码注释，为生成的 Go 函数添加详细的文档注释。

**期望的 Go 输出**:
```go
// SetOwner 设置合约的所有者地址
// 只有当前所有者可以调用此函数
// new_owner: 新的所有者地址
// 返回: 执行结果，如果失败返回错误
func (c *WreviveExample) ExecSetOwner(
    new_owner types.H160, __ink_params chain.ExecParams,
) error {
    // ...
}
```

### 场景3: Rust 生成代码注释优化
为宏生成的 Rust 代码添加更详细的注释，解释代码逻辑和用途。

## 架构技术方案

### 1. 注释提取模块设计
在 `wrevive-macro` crate 中新增注释处理功能：

**新增文件**: `crates/wrevive-macro/src/docs.rs`
- 提取 Rust 函数的文档注释
- 解析参数文档标记（如 `@param`）
- 支持 JSDoc 风格的注释格式

### 2. ABI 生成增强
修改 `crates/wrevive-macro/src/abi.rs`：
- 在构造函数和消息函数处理时，提取并包含注释信息
- 支持 ink! 风格的文档注释格式
- 处理多行注释和参数文档

### 3. Go 代码生成增强
修改 Go 代码生成逻辑（可能在 `cargo-wrevive` 或外部工具中）：
- 从 ABI 中读取 `docs` 信息
- 生成符合 Go 文档规范的注释
- 支持参数和返回值的文档说明

### 4. Rust 生成代码注释优化
修改 `crates/wrevive-macro/src/contract.rs`：
- 为 `deploy()` 函数生成详细注释
- 为 `call()` 函数添加选择器分发逻辑说明
- 为 interface 模块添加使用说明

## 影响文件

### 修改的文件
1. `crates/wrevive-macro/src/abi.rs` - ABI 生成逻辑
2. `crates/wrevive-macro/src/contract.rs` - 合约代码生成
3. `crates/wrevive-macro/src/interface.rs` - 接口生成
4. `crates/wrevive-macro/src/lib.rs` - 新增 docs 模块导入

### 新增的文件
1. `crates/wrevive-macro/src/docs.rs` - 注释处理模块
2. 可能需要修改 Go 代码生成工具（如果存在于项目中）

### 测试文件
1. `examples/wrevive-contract/src/contract.rs` - 添加注释示例
2. 对应的测试用例验证生成的注释

## 实现细节

### 1. 注释提取算法
```rust
/// 从函数属性中提取文档注释
pub fn extract_docs(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit { 
                    lit: syn::Lit::Str(lit_str), .. 
                }) = &meta.value {
                    Some(lit_str.value())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}
```

### 2. 参数文档解析
支持 `@param param_name description` 格式的参数文档：
```rust
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
```

### 3. Go 注释模板
```go
// {{func_name}} {{func_description}}
{{#each params}}
// {{name}}: {{description}}
{{/each}}
// 返回: {{return_description}}
func (c *{{contract_name}}) {{go_func_name}}(
    {{#each params}}{{name}} {{type}}, {{/each}}__ink_params chain.{{param_type}},
) {{return_type}} {
    // 生成的方法体
}
```

### 4. Rust 生成代码注释示例
```rust
quote! {
    /// 合约部署入口函数
    /// 
    /// 此函数由 #[revive_contract] 宏自动生成，用于：
    /// 1. 解码部署参数（selector + SCALE 编码的构造函数参数）
    /// 2. 调用用户定义的构造函数
    /// 3. 将构造函数返回值编码并设置到合约返回数据中
    /// 
    /// # 参数格式
    /// - input_data: selector(4字节) + SCALE 编码的构造函数参数
    /// 
    /// # 返回值
    /// - 成功: 构造函数返回值的 SCALE 编码
    /// - 失败: REVERT 标志 + 错误信息
    #[polkavm_derive::polkavm_export]
    #[allow(unreachable_code)]
    pub extern "C" fn deploy() {
        #deploy_body
    }
}
```

## 边界条件与异常处理

### 1. 注释格式兼容性
- **问题**: 不同开发者可能使用不同的注释风格
- **解决方案**: 支持多种注释格式的解析，包括：
  - 标准的 Rust 文档注释 `///`
  - 内联文档注释 `//!`
  - JSDoc 风格的 `@param` 标记
  - Ink! 风格的特殊注释

### 2. 多语言注释支持
- **问题**: 注释可能包含中文、英文等多种语言
- **解决方案**: 保持原始注释内容，在生成的代码中保持编码一致性

### 3. 注释长度限制
- **问题**: 某些目标格式可能对注释长度有限制
- **解决方案**: 
  - 对过长的注释进行智能截断
  - 提供配置选项控制注释的最大长度
  - 保留重要信息，优先参数说明

### 4. 特殊字符处理
- **问题**: 注释中的特殊字符可能影响生成的代码
- **解决方案**: 
  - 对 HTML 特殊字符进行转义（用于 JSON 输出）
  - 对 Go 字符串字面量中的特殊字符进行转义
  - 对 Rust 字符串字面量中的特殊字符进行转义

## 数据流动路径

```
Rust 源码注释
    ↓ [docs.rs: extract_docs()]
提取的注释信息
    ↓ [abi.rs: emit_abi()]
ABI JSON (docs 字段)
    ↓ [Go 代码生成工具]
Go SDK 函数注释
    ↓ [contract.rs: revive_contract_impl()]
Rust 生成代码注释
```

## 预期成果

### 1. 完善的 ABI 文档
- 所有构造函数和消息函数都包含详细的 `docs` 字段
- 参数级别的文档说明
- 符合 ink! ABI 规范的文档格式

### 2. 高质量的 Go SDK
- 每个生成的函数都有完整的文档注释
- 参数说明清晰
- 返回值描述准确

### 3. 可读的 Rust 生成代码
- 生成的 `deploy()` 和 `call()` 函数有详细注释
- 接口模块有使用说明
- 代码逻辑更容易理解

### 4. 开发者体验提升
- 合约开发者可以通过注释自然地生成文档
- 前端开发者可以获得有意义的 API 文档
- 减少额外编写文档的工作量

## 测试验证

### 1. 单元测试
- 注释提取功能的单元测试
- 不同注释格式的兼容性测试
- 边界条件处理的测试

### 2. 集成测试
- 完整的合约编译和 ABI 生成测试
- Go 代码生成的端到端测试
- 生成代码的编译和运行测试

### 3. 示例验证
- 使用示例合约验证注释生成效果
- 检查生成的 ABI JSON 格式正确性
- 验证 Go 代码的可编译性

这个优化将显著提升 wrevive 项目的开发者体验，使生成的代码更加专业和易于使用。
