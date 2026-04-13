#!/bin/bash
set -e

# 算盘 (suanpan) 安装脚本
# 使用方法: curl -sSL https://raw.githubusercontent.com/yinguobing/suanpan/main/install.sh | bash

REPO="yinguobing/suanpan"
INSTALL_DIR="/usr/local/bin"

# 检测操作系统和架构
detect_target() {
    local os arch
    
    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="macos" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *)          echo "不支持的操作系统: $(uname -s)" >&2; exit 1 ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        arm64|aarch64)  arch="aarch64" ;;
        *)              echo "不支持的架构: $(uname -m)" >&2; exit 1 ;;
    esac
    
    # 构建目标名称
    if [ "$os" = "linux" ]; then
        # Linux 优先使用 musl 静态链接版本
        echo "x86_64-unknown-linux-musl"
    elif [ "$os" = "macos" ]; then
        if [ "$arch" = "aarch64" ]; then
            echo "aarch64-apple-darwin"
        else
            echo "x86_64-apple-darwin"
        fi
    else
        echo "x86_64-pc-windows-msvc"
    fi
}

# 获取最新版本号
get_latest_version() {
    curl -s "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}

# 下载并安装
download_and_install() {
    local target version url tmp_dir
    
    target=$(detect_target)
    version=$(get_latest_version)
    
    if [ -z "$version" ]; then
        echo "错误: 无法获取最新版本信息" >&2
        exit 1
    fi
    
    echo "检测到目标平台: ${target}"
    echo "最新版本: ${version}"
    
    # 构建下载 URL
    local archive_ext="tar.gz"
    if [[ "$target" == *"windows"* ]]; then
        archive_ext="zip"
    fi
    
    url="https://github.com/${REPO}/releases/download/${version}/suanpan-${version}-${target}.${archive_ext}"
    
    # 创建临时目录
    tmp_dir=$(mktemp -d)
    trap "rm -rf ${tmp_dir}" EXIT
    
    echo "下载中: ${url}"
    
    # 下载
    if ! curl -sSL "$url" -o "${tmp_dir}/suanpan.${archive_ext}"; then
        echo "错误: 下载失败" >&2
        exit 1
    fi
    
    # 解压
    echo "解压中..."
    cd "$tmp_dir"
    if [ "$archive_ext" = "zip" ]; then
        unzip -q "suanpan.${archive_ext}"
        mv suanpan.exe suanpan 2>/dev/null || true
    else
        tar -xzf "suanpan.${archive_ext}"
    fi
    
    # 检查二进制文件
    if [ ! -f "suanpan" ]; then
        echo "错误: 解压后未找到 suanpan 二进制文件" >&2
        exit 1
    fi
    
    # 安装
    echo "安装到 ${INSTALL_DIR}..."
    if [ -w "$INSTALL_DIR" ]; then
        mv suanpan "$INSTALL_DIR/"
    else
        echo "需要 sudo 权限安装到 ${INSTALL_DIR}"
        sudo mv suanpan "$INSTALL_DIR/"
    fi
    
    # 验证安装
    if command -v suanpan >/dev/null 2>&1; then
        echo ""
        echo "✓ 安装成功!"
        suanpan --version
        echo ""
        echo "使用 'suanpan --help' 查看帮助信息"
    else
        echo "警告: suanpan 已安装但不在 PATH 中" >&2
        echo "请确保 ${INSTALL_DIR} 在你的 PATH 环境变量中"
    fi
}

# 主函数
main() {
    echo "=== 算盘 (suanpan) 安装脚本 ==="
    echo ""
    
    # 检查依赖
    if ! command -v curl >/dev/null 2>&1; then
        echo "错误: 需要 curl 但未安装" >&2
        exit 1
    fi
    
    download_and_install
}

main "$@"
