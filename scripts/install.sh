#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# cx-switch 安装脚本
# 用法: curl -fsSL https://raw.githubusercontent.com/jay6697117/cx-switch/main/scripts/install.sh | bash
# ============================================================

usage() {
  cat <<'EOF'
从 GitHub Releases 安装 cx-switch。

用法:
  ./scripts/install.sh [选项]

选项:
  --repo <owner/repo>  GitHub 仓库（默认: jay6697117/cx-switch）
  --version <tag>      版本号或 'latest'（默认: latest）
  --install-dir <dir>  安装目录（默认: $HOME/.local/bin）
  --no-add-to-path     跳过自动添加到 shell profile
  -h, --help           显示帮助
EOF
}

# 默认配置
INSTALL_DIR="${HOME}/.local/bin"
VERSION="latest"
REPO="jay6697117/cx-switch"
ADD_TO_PATH=1
SHELL_NAME="$(basename "${SHELL:-}")"
PROFILE_FILE=""

# 颜色输出
if [[ -t 1 && -z "${NO_COLOR:-}" ]]; then
  C_RESET=$'\033[0m'
  C_BOLD=$'\033[1m'
  C_GREEN=$'\033[32m'
  C_YELLOW=$'\033[33m'
  C_CYAN=$'\033[36m'
  C_RED=$'\033[31m'
else
  C_RESET=""
  C_BOLD=""
  C_GREEN=""
  C_YELLOW=""
  C_CYAN=""
  C_RED=""
fi

print_color() {
  local color="$1"
  shift
  printf "%b\n" "${color}$*${C_RESET}"
}

print_success() { print_color "${C_BOLD}${C_GREEN}" "$*"; }
print_warn()    { print_color "${C_BOLD}${C_YELLOW}" "$*"; }
print_info()    { print_color "${C_CYAN}" "$*"; }
print_cmd()     { print_color "${C_BOLD}${C_CYAN}" "$*"; }
print_error()   { print_color "${C_BOLD}${C_RED}" "$*"; }

# 检测操作系统和架构，返回对应的压缩包文件名
detect_asset() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "${os}" in
    Linux)  os="Linux" ;;
    Darwin) os="macOS" ;;
    *)
      print_error "不支持的操作系统: ${os}"
      exit 1
      ;;
  esac

  case "${arch}" in
    x86_64|amd64)   arch="X64" ;;
    arm64|aarch64)   arch="ARM64" ;;
    *)
      print_error "不支持的架构: ${arch}"
      exit 1
      ;;
  esac

  echo "cx-switch-${os}-${arch}.tar.gz"
}

# 去掉路径尾部多余的斜杠
normalize_path_entry() {
  local value="${1:-}"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  if [[ "${value}" == "/" ]]; then
    printf "/"
    return
  fi
  while [[ "${value}" == */ && "${value}" != "/" ]]; do
    value="${value%/}"
  done
  printf "%s" "${value}"
}

# 检查 PATH 中是否已包含指定目录
path_contains_dir() {
  local target normalized_target
  target="${1}"
  normalized_target="$(normalize_path_entry "${target}")"
  IFS=':' read -r -a _segments <<< "${PATH:-}"
  for segment in "${_segments[@]}"; do
    if [[ "$(normalize_path_entry "${segment}")" == "${normalized_target}" ]]; then
      return 0
    fi
  done
  return 1
}

# 检测用户的 shell profile 文件
detect_profile_file() {
  local candidate

  if [[ "${SHELL_NAME}" == "fish" ]]; then
    printf "%s" "${HOME}/.config/fish/config.fish"
    return
  fi

  if [[ "${SHELL_NAME}" == "zsh" ]]; then
    for candidate in "${HOME}/.zshrc" "${HOME}/.zprofile" "${HOME}/.profile"; do
      if [[ -f "${candidate}" ]]; then
        printf "%s" "${candidate}"
        return
      fi
    done
    printf "%s" "${HOME}/.zshrc"
    return
  fi

  for candidate in "${HOME}/.bashrc" "${HOME}/.bash_profile" "${HOME}/.profile"; do
    if [[ -f "${candidate}" ]]; then
      printf "%s" "${candidate}"
      return
    fi
  done
  printf "%s" "${HOME}/.bashrc"
}

# 获取 shell 显示名称
shell_display_name() {
  case "${SHELL_NAME}" in
    fish|zsh|bash) printf "%s" "${SHELL_NAME}" ;;
    *) printf "shell" ;;
  esac
}

# 将安装目录写入 shell profile
persist_path_to_profile() {
  local profile path_line
  profile="$(detect_profile_file)"
  PROFILE_FILE="${profile}"
  mkdir -p "$(dirname "${profile}")"
  touch "${profile}"

  # 如果已经存在就跳过
  if grep -Fq "${INSTALL_DIR}" "${profile}"; then
    return
  fi

  if [[ "${SHELL_NAME}" == "fish" ]]; then
    {
      echo ""
      echo "# Added by cx-switch installer"
      echo "if not contains -- \"${INSTALL_DIR}\" \$PATH"
      echo "    set -gx PATH \"${INSTALL_DIR}\" \$PATH"
      echo "end"
    } >> "${profile}"
  else
    path_line="export PATH=\"${INSTALL_DIR}:\$PATH\""
    {
      echo ""
      echo "# Added by cx-switch installer"
      echo "${path_line}"
    } >> "${profile}"
  fi
}

# 打印重启 shell 提示
print_shell_restart_hint() {
  case "${SHELL_NAME}" in
    fish)
      print_warn "重启 shell："
      print_cmd "  exec fish"
      ;;
    zsh)
      print_warn "重启 shell："
      print_cmd "  exec zsh -l"
      ;;
    bash)
      print_warn "重启 shell："
      print_cmd "  exec bash"
      ;;
    *)
      print_warn "重新打开终端即可使用。"
      ;;
  esac
}

# ============================================================
# 主流程
# ============================================================

# 解析命令行参数
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      REPO="$2"
      shift 2
      ;;
    --version)
      VERSION="$2"
      shift 2
      ;;
    --install-dir)
      INSTALL_DIR="$2"
      shift 2
      ;;
    --add-to-path)
      ADD_TO_PATH=1
      shift
      ;;
    --no-add-to-path)
      ADD_TO_PATH=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      print_error "未知选项: $1"
      usage
      exit 1
      ;;
  esac
done

# 检查 curl 是否可用
if ! command -v curl >/dev/null 2>&1; then
  print_error "需要 curl，请先安装。"
  exit 1
fi

# 检测系统并获取对应的压缩包文件名
ASSET="$(detect_asset)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

# 构建下载 URL
URL=""
if [[ "${VERSION}" == "latest" ]]; then
  URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
else
  URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
fi

# 下载
echo ""
print_info "📦 正在下载 cx-switch..."
print_info "   ${URL}"
echo ""
curl -fL "${URL}" -o "${TMP_DIR}/${ASSET}"

# 解压
tar -xzf "${TMP_DIR}/${ASSET}" -C "${TMP_DIR}"
BIN_PATH="${TMP_DIR}/cx-switch"

if [[ ! -f "${BIN_PATH}" ]]; then
  print_error "下载的压缩包中不包含 cx-switch 二进制文件。"
  exit 1
fi

# 安装到目标目录
mkdir -p "${INSTALL_DIR}"
DEST_BIN="${INSTALL_DIR}/cx-switch"

if command -v install >/dev/null 2>&1; then
  install -m 0755 "${BIN_PATH}" "${DEST_BIN}"
else
  cp "${BIN_PATH}" "${DEST_BIN}"
  chmod 0755 "${DEST_BIN}"
fi

# 完成提示
echo ""
print_success "✅ cx-switch 安装成功！"
print_info "   路径: ${DEST_BIN}"
echo ""

# PATH 处理
CURRENT_PATH_MISSING=0
if path_contains_dir "${INSTALL_DIR}"; then
  :
else
  CURRENT_PATH_MISSING=1
fi

if [[ "${ADD_TO_PATH}" -eq 1 ]]; then
  persist_path_to_profile
fi

if [[ "${ADD_TO_PATH}" -eq 1 && -n "${PROFILE_FILE}" ]]; then
  if [[ "${CURRENT_PATH_MISSING}" -eq 0 ]]; then
    print_success "✅ 已就绪，可直接使用（通过 ${PROFILE_FILE} 加载）。"
  else
    print_success "✅ 已配置到 ${PROFILE_FILE}，新终端会话中可直接使用。"
  fi
elif [[ "${CURRENT_PATH_MISSING}" -eq 0 ]]; then
  print_success "✅ 已就绪，可在当前终端中使用。"
else
  print_warn "⚠️  当前终端尚未就绪。"
fi

if [[ "${CURRENT_PATH_MISSING}" -eq 1 ]]; then
  echo ""
  print_warn "在当前终端中立即使用："
  if [[ "${SHELL_NAME}" == "fish" ]]; then
    print_cmd "  set -gx PATH \"${INSTALL_DIR}\" \$PATH"
  else
    print_cmd "  export PATH=\"${INSTALL_DIR}:\$PATH\""
  fi

  if [[ "${ADD_TO_PATH}" -eq 1 && -n "${PROFILE_FILE}" ]]; then
    echo ""
    print_warn "或重新加载 shell 配置："
    print_cmd "  source \"${PROFILE_FILE}\""
    echo ""
    print_shell_restart_hint
  fi
fi

echo ""
print_info "🚀 开始使用："
print_cmd "  cx-switch --version"
print_cmd "  cx-switch list"
print_cmd "  cx-switch --help"
echo ""
