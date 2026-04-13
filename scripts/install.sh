#!/bin/bash
# suanpan 一键安装脚本
# 用法: curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/yinguobing/suanpan/main/scripts/install.sh | sh

set -e

REPO="yinguobing/suanpan"
BINARY_NAME="suanpan"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检测操作系统
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        CYGWIN*|MINGW*|MSYS*) echo "windows";;
        *)          echo "unknown";;
    esac
}

# 检测架构
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64";;
        aarch64|arm64)  echo "aarch64";;
        *)              echo "unknown";;
    esac
}

# 获取最新版本
get_latest_version() {
    curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

# 主函数
main() {
    info "开始安装 ${BINARY_NAME}..."
    
    # 检测平台
    OS=$(detect_os)
    ARCH=$(detect_arch)
    
    if [ "$OS" = "unknown" ] || [ "$ARCH" = "unknown" ]; then
        error "不支持的平台: ${OS} ${ARCH}"
        exit 1
    fi
    
    info "检测到平台: ${OS} ${ARCH}"
    
    # Windows 不支持此脚本
    if [ "$OS" = "windows" ]; then
        error "Windows 平台请手动下载 release 页面中的 zip 文件"
        error "访问: https://github.com/${REPO}/releases/latest"
        exit 1
    fi
    
    # 获取最新版本
    info "获取最新版本..."
    VERSION=$(get_latest_version)
    
    if [ -z "$VERSION" ]; then
        error "无法获取最新版本信息"
        exit 1
    fi
    
    success "最新版本: ${VERSION}"
    
    # 构建下载 URL
    # 优先使用 musl 版本（静态链接，兼容性更好）
    TARGET="${ARCH}-unknown-linux-musl"
    if [ "$OS" = "macos" ]; then
        TARGET="${ARCH}-apple-darwin"
    fi
    
    FILENAME="${BINARY_NAME}-${VERSION}-${TARGET}.tar.gz"
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILENAME}"
    
    info "下载: ${URL}"
    
    # 创建临时目录
    TMP_DIR=$(mktemp -d)
    trap "rm -rf ${TMP_DIR}" EXIT
    
    # 下载
    if ! curl -fsSL "$URL" -o "${TMP_DIR}/${FILENAME}"; then
        # 如果 musl 版本不存在，尝试 gnu 版本
        if [ "$TARGET" = "${ARCH}-unknown-linux-musl" ]; then
            warn "musl 版本不存在，尝试 gnu 版本..."
            TARGET="${ARCH}-unknown-linux-gnu"
            FILENAME="${BINARY_NAME}-${VERSION}-${TARGET}.tar.gz"
            URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILENAME}"
            info "下载: ${URL}"
            if ! curl -fsSL "$URL" -o "${TMP_DIR}/${FILENAME}"; then
                error "下载失败"
                exit 1
            fi
        else
            error "下载失败"
            exit 1
        fi
    fi
    
    success "下载完成"
    
    # 解压
    info "解压..."
    tar -xzf "${TMP_DIR}/${FILENAME}" -C "$TMP_DIR"
    
    # 确定安装路径
    if [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="${HOME}/.local/bin"
        mkdir -p "$INSTALL_DIR"
    fi
    
    # 安装
    info "安装到 ${INSTALL_DIR}..."
    mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    
    success "安装完成!"
    
    # 检查 PATH
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        warn "${INSTALL_DIR} 不在 PATH 中"
        warn "请添加以下内容到 ~/.bashrc 或 ~/.zshrc:"
        warn "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
    
    # 验证安装
    if command -v "$BINARY_NAME" &> /dev/null; then
        success "$(${BINARY_NAME} --version 2>/dev/null || echo '${BINARY_NAME} 已安装')"
    else
        info "安装路径: ${INSTALL_DIR}/${BINARY_NAME}"
    fi
    
    info "使用 'suanpan --help' 查看帮助"
}

main "$@"
