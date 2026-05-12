# wTools v0.4.1 Installer
# Run: powershell -ExecutionPolicy Bypass -File install.ps1

$ErrorActionPreference = "Stop"
$appName = "wTools"
$installDir = "$env:LOCALAPPDATA\$appName"
$startMenu = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
$shortcutPath = "$startMenu\$appName.lnk"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  wTools v0.4.0 安装程序" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Create install directory
Write-Host "[1/4] 安装到: $installDir" -ForegroundColor Yellow
if (Test-Path $installDir) {
    Write-Host "  检测到已有安装，正在覆盖..."
}
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

# Copy files
Write-Host "[2/4] 复制文件..." -ForegroundColor Yellow
$sourceDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Get-ChildItem -Path $sourceDir -Exclude "install.ps1","*.zip" | Copy-Item -Destination $installDir -Recurse -Force

# Create Start Menu shortcut
Write-Host "[3/4] 创建开始菜单快捷方式..." -ForegroundColor Yellow
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($shortcutPath)
$Shortcut.TargetPath = "$installDir\lib.exe"
$Shortcut.WorkingDirectory = $installDir
$Shortcut.IconLocation = "$installDir\assets\icon.ico"
$Shortcut.Save()

# Add to PATH (optional)
Write-Host "[4/4] 完成!" -ForegroundColor Yellow
Write-Host ""
Write-Host "安装完成!" -ForegroundColor Green
Write-Host "  程序目录: $installDir" -ForegroundColor White
Write-Host "  快捷方式: 开始菜单 -> wTools" -ForegroundColor White
Write-Host ""
Write-Host "使用说明:" -ForegroundColor Cyan
Write-Host "  - Alt+Space 唤醒搜索界面" -ForegroundColor White
Write-Host "  - 右击托盘图标设置开机自启" -ForegroundColor White
Write-Host "  - 按 Tab 切换搜索/剪贴板" -ForegroundColor White
Write-Host "  - 需要安装 Everything 用于文件搜索" -ForegroundColor White
Write-Host "    https://www.voidtools.com/" -ForegroundColor DarkGray
Write-Host ""

# Launch app after install
$choice = Read-Host "是否立即启动 wTools? (y/n)"
if ($choice -eq 'y') {
    Start-Process "$installDir\lib.exe"
}
