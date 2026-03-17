# ============================================================
# cx-switch Windows 安装脚本
# 用法: irm https://raw.githubusercontent.com/jay6697117/cx-switch/main/scripts/install.ps1 | iex
# ============================================================

param(
  [string]$Repo = "jay6697117/cx-switch",
  [string]$Version = "latest",
  [string]$InstallDir = "$env:LOCALAPPDATA\cx-switch\bin",
  [switch]$NoAddToPath
)

$ErrorActionPreference = "Stop"

function Write-Info {
  param([string]$Message)
  Write-Host $Message -ForegroundColor Cyan
}

function Write-Success {
  param([string]$Message)
  Write-Host $Message -ForegroundColor Green
}

function Write-Warn {
  param([string]$Message)
  Write-Host $Message -ForegroundColor Yellow
}

function Normalize-PathEntry {
  param([string]$PathEntry)
  if ([string]::IsNullOrWhiteSpace($PathEntry)) {
    return ""
  }
  $normalized = $PathEntry.Trim()
  if ($normalized.Length -gt 3) {
    $normalized = $normalized.TrimEnd('\')
  }
  return $normalized
}

function Get-PathSegments {
  param([string]$PathValue)
  if ([string]::IsNullOrWhiteSpace($PathValue)) {
    return @()
  }
  return @(
    $PathValue -split ';' |
      ForEach-Object { Normalize-PathEntry $_ } |
      Where-Object { $_ -ne "" }
  )
}

# 检测系统架构
function Detect-Asset {
  $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
  $archText = switch ($arch) {
    "X64" { "X64" }
    "Arm64" { "ARM64" }
    default { throw "不支持的架构: $arch" }
  }
  return "cx-switch-Windows-$archText.zip"
}

# 检查依赖
if (-not (Get-Command Invoke-WebRequest -ErrorAction SilentlyContinue)) {
  throw "需要 Invoke-WebRequest 命令。"
}

$Asset = Detect-Asset

# 构建下载 URL
$DownloadUrl = if ($Version -eq "latest") {
  "https://github.com/$Repo/releases/latest/download/$Asset"
} else {
  "https://github.com/$Repo/releases/download/$Version/$Asset"
}

# 下载并安装
$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("cx-switch-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -Path $TempDir -ItemType Directory -Force | Out-Null

try {
  $ArchivePath = Join-Path $TempDir $Asset

  Write-Host ""
  Write-Info "📦 正在下载 cx-switch..."
  Write-Info "   $DownloadUrl"
  Write-Host ""

  Invoke-WebRequest -Uri $DownloadUrl -OutFile $ArchivePath

  # 解压
  Expand-Archive -Path $ArchivePath -DestinationPath $TempDir -Force
  $SourceBin = Join-Path $TempDir "cx-switch.exe"
  if (-not (Test-Path $SourceBin)) {
    throw "下载的压缩包中不包含 cx-switch.exe"
  }

  # 安装
  New-Item -Path $InstallDir -ItemType Directory -Force | Out-Null
  $DestBin = Join-Path $InstallDir "cx-switch.exe"
  Copy-Item -Path $SourceBin -Destination $DestBin -Force

  Write-Host ""
  Write-Success "✅ cx-switch 安装成功！"
  Write-Info "   路径: $DestBin"
  Write-Host ""
} finally {
  Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
}

# PATH 处理
$normalizedInstallDir = Normalize-PathEntry $InstallDir
$currentSegments = Get-PathSegments $env:Path

# 添加到当前终端 PATH
if ($currentSegments -notcontains $normalizedInstallDir) {
  $env:Path = if ([string]::IsNullOrWhiteSpace($env:Path)) { $InstallDir } else { "$InstallDir;$env:Path" }
}

# 持久化到用户 PATH
$persistPath = -not $NoAddToPath
if ($persistPath) {
  $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
  $userSegments = Get-PathSegments $userPath
  if ($userSegments -notcontains $normalizedInstallDir) {
    $newPath = if ([string]::IsNullOrWhiteSpace($userPath)) { $InstallDir } else { "$userPath;$InstallDir" }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
  }
  Write-Success "✅ 已就绪，可在 PowerShell 中使用（已添加到用户 PATH）。"
} else {
  Write-Success "✅ 已就绪，可在当前终端中使用。"
  Write-Info "不带 -NoAddToPath 重新运行安装脚本可自动配置到用户 PATH。"
}

Write-Host ""
Write-Info "🚀 开始使用："
Write-Info "  cx-switch --version"
Write-Info "  cx-switch list"
Write-Info "  cx-switch --help"
Write-Host ""
