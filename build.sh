#!/bin/bash
set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 要构建的示例合约包（可改为其他 examples 下的包名）
CONTRACT_PKG="${1:-wrevive-example-contract}"
CONTRACT_TOML="examples/${CONTRACT_PKG}/Cargo.toml"
if [ ! -f "$CONTRACT_TOML" ]; then
    echo -e "${RED}错误: 找不到 $CONTRACT_TOML${NC}"
    exit 1
fi
# 从该包的 Cargo.toml 读取 [lib] name（用于 ELF 路径）
CONTRACT_LIB_NAME=$(sed -n '/^\[lib\]/,/^\[/p' "$CONTRACT_TOML" 2>/dev/null | grep '^name\s*=' | head -1 | sed 's/.*"\(.*\)".*/\1/')
[ -z "$CONTRACT_LIB_NAME" ] && CONTRACT_LIB_NAME=$(sed -n 's/^name\s*=\s*"\(.*\)"/\1/p' "$CONTRACT_TOML" | head -1 | tr '-' '_')
[ -z "$CONTRACT_LIB_NAME" ] && CONTRACT_LIB_NAME="contract"

echo -e "${GREEN}=== 构建 PolkaVM 合约 ===${NC}"
echo -e "  包: $CONTRACT_PKG (ELF: $CONTRACT_LIB_NAME)"

# 设置目标文件路径
TARGET_JSON="riscv64emac-unknown-none-polkavm.json"
TARGET_NAME="riscv64emac-unknown-none-polkavm"

# 检查目标文件是否存在
if [ ! -f "$TARGET_JSON" ]; then
    echo -e "${RED}错误: 找不到目标文件 $TARGET_JSON${NC}"
    exit 1
fi

# 检查 Rust 工具链
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}错误: 未找到 Rust 工具链${NC}"
    exit 1
fi

echo -e "${YELLOW}步骤 1/3: 编译合约...${NC}"

# 使用 build.rs 写入的 target（来自 polkavm-linker），产出 cdylib .elf
RUSTC_BOOTSTRAP=1 cargo +stable build \
  -p "$CONTRACT_PKG" \
  --release \
  --no-default-features \
  --target "$TARGET_JSON" \
  -Z build-std=core > build.log 2>&1
CARGO_EXIT=$?
cat build.log
if [ "$CARGO_EXIT" -ne 0 ]; then
    echo -e "${RED}编译失败！${NC}"
    exit 1
fi


# 输出 ELF 路径：与 [lib] name 一致（polkavm-linker target 产出 .elf）
ELF_FILE="target/${TARGET_NAME}/release/${CONTRACT_LIB_NAME}.elf"

if [ ! -f "$ELF_FILE" ]; then
    echo -e "${RED}错误: 找不到编译输出文件${NC}"
    echo "查找路径: target/${TARGET_NAME}/release/"
    ls -la "target/${TARGET_NAME}/release/" || true
    exit 1
fi

echo -e "${GREEN}✓ 编译成功${NC}"
echo -e "  ELF 文件: $ELF_FILE"

echo -e "${YELLOW}步骤 2/3: 检查 polkatool...${NC}"

# 检查 polkatool（polkavm-linker 仅为库无二进制，需用 polkatool link）
if ! command -v polkatool &> /dev/null; then
    echo -e "${YELLOW}警告: 未找到 polkatool${NC}"
    echo -e "${YELLOW}安装命令: cargo install polkatool${NC}"
    echo -e "${YELLOW}跳过链接步骤，仅生成 ELF 文件${NC}"
    exit 0
fi

echo -e "${YELLOW}步骤 3/3: 链接和优化...${NC}"

# 创建输出目录
mkdir -p target/contract

# 链接（polkatool link 将 ELF 转为 .polkavm 字节码），输出文件使用 ABI 中的合约名
OUTPUT_FILE="target/contract/${CONTRACT_LIB_NAME}.polkavm"
polkatool link --strip -o "$OUTPUT_FILE" "$ELF_FILE"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ 链接成功${NC}"
    echo -e "  输出文件: $OUTPUT_FILE"
    
    # 显示文件大小
    ELF_SIZE=$(du -h "$ELF_FILE" | cut -f1)
    PVM_SIZE=$(du -h "$OUTPUT_FILE" | cut -f1)
    echo -e "  ELF 大小: $ELF_SIZE"
    echo -e "  PolkaVM 大小: $PVM_SIZE"
else
    echo -e "${RED}链接失败！${NC}"
    exit 1
fi

echo -e "${GREEN}=== 构建完成 ===${NC}"
