#!/bin/bash
# GaussDB-Rust Crates.io 发布脚本
# 
# 使用方法:
#   bash scripts/publish-to-crates.sh [--dry-run]
#
# 选项:
#   --dry-run    执行干运行，不实际发布

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查参数
DRY_RUN=false
if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN=true
    echo -e "${YELLOW}🔍 执行干运行模式${NC}"
fi

# 发布包列表（按依赖顺序）
PACKAGES=(
    "gaussdb-protocol"
    "gaussdb-derive" 
    "gaussdb-types"
    "tokio-gaussdb"
    "gaussdb"
    "gaussdb-native-tls"
    "gaussdb-openssl"
)

echo -e "${BLUE}🚀 开始 GaussDB-Rust 包发布流程${NC}"
echo "=================================================="

# 检查是否已登录 crates.io
echo -e "${BLUE}🔐 检查 crates.io 登录状态...${NC}"
if ! cargo login --help > /dev/null 2>&1; then
    echo -e "${RED}❌ 请先登录 crates.io: cargo login${NC}"
    exit 1
fi

# 最终检查
echo -e "${BLUE}🔍 执行发布前检查...${NC}"

# 检查工作区状态
if [[ -n $(git status --porcelain) ]]; then
    echo -e "${RED}❌ 工作区有未提交的更改，请先提交所有更改${NC}"
    exit 1
fi

# 检查编译状态
echo -e "${BLUE}🔨 检查编译状态...${NC}"
if ! cargo check --workspace; then
    echo -e "${RED}❌ 编译检查失败${NC}"
    exit 1
fi

# 检查测试状态
echo -e "${BLUE}🧪 运行核心测试...${NC}"
if ! cargo test --manifest-path tokio-gaussdb/Cargo.toml --no-default-features --lib; then
    echo -e "${RED}❌ 核心测试失败${NC}"
    exit 1
fi

# 检查文档生成
echo -e "${BLUE}📚 检查文档生成...${NC}"
if ! cargo doc --workspace --no-deps; then
    echo -e "${RED}❌ 文档生成失败${NC}"
    exit 1
fi

echo -e "${GREEN}✅ 所有检查通过！${NC}"
echo ""

# 发布包
for package in "${PACKAGES[@]}"; do
    echo -e "${BLUE}📦 准备发布: ${package}${NC}"
    
    # 检查包是否存在
    if [[ ! -d "$package" ]]; then
        echo -e "${YELLOW}⚠️  跳过不存在的包: $package${NC}"
        continue
    fi
    
    # 检查 Cargo.toml
    if [[ ! -f "$package/Cargo.toml" ]]; then
        echo -e "${YELLOW}⚠️  跳过没有 Cargo.toml 的包: $package${NC}"
        continue
    fi
    
    # 获取包版本
    VERSION=$(grep "^version" "$package/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
    echo -e "   版本: ${GREEN}$VERSION${NC}"
    
    # 检查包是否已发布
    if cargo search "$package" --limit 1 | grep -q "^$package "; then
        PUBLISHED_VERSION=$(cargo search "$package" --limit 1 | grep "^$package " | sed 's/.* = "\(.*\)".*/\1/')
        if [[ "$VERSION" == "$PUBLISHED_VERSION" ]]; then
            echo -e "${YELLOW}⚠️  版本 $VERSION 已存在，跳过发布${NC}"
            continue
        fi
    fi
    
    # 执行发布
    if [[ "$DRY_RUN" == "true" ]]; then
        echo -e "${YELLOW}🔍 干运行: cargo publish --manifest-path $package/Cargo.toml --dry-run${NC}"
        if ! cargo publish --manifest-path "$package/Cargo.toml" --dry-run; then
            echo -e "${RED}❌ 干运行失败: $package${NC}"
            exit 1
        fi
    else
        echo -e "${GREEN}🚀 发布: $package${NC}"
        if ! cargo publish --manifest-path "$package/Cargo.toml"; then
            echo -e "${RED}❌ 发布失败: $package${NC}"
            exit 1
        fi
        
        # 等待包在 crates.io 上可用
        echo -e "${BLUE}⏳ 等待包在 crates.io 上可用...${NC}"
        sleep 30
    fi
    
    echo -e "${GREEN}✅ 完成: $package${NC}"
    echo ""
done

echo "=================================================="
if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${GREEN}🎉 干运行完成！所有包都可以成功发布。${NC}"
    echo -e "${BLUE}💡 要执行实际发布，请运行: bash scripts/publish-to-crates.sh${NC}"
else
    echo -e "${GREEN}🎉 发布完成！所有包已成功发布到 crates.io${NC}"
    echo ""
    echo -e "${BLUE}📋 发布的包:${NC}"
    for package in "${PACKAGES[@]}"; do
        if [[ -d "$package" && -f "$package/Cargo.toml" ]]; then
            VERSION=$(grep "^version" "$package/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
            echo -e "   • ${GREEN}$package${NC} v$VERSION"
        fi
    done
    echo ""
    echo -e "${BLUE}🔗 查看发布的包:${NC}"
    echo "   https://crates.io/crates/gaussdb"
    echo "   https://crates.io/crates/tokio-gaussdb"
fi
