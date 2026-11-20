param(
  [switch]$Install,
  [switch]$Pack
)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$PluginId = "dev.flowingspdg.advancedhttp"
$PluginDir = Join-Path $Root "plugin\$PluginId.sdPlugin"
Set-Location $Root

cargo run -p advanced-http-plugin --bin typegen
Push-Location (Join-Path $Root "pi")
if (-not (Test-Path "node_modules")) { npm install }
npm run build
Pop-Location

rustup target add x86_64-pc-windows-msvc
cargo build -p advanced-http-plugin --release --bin plugin --target x86_64-pc-windows-msvc
$Dest = Join-Path $PluginDir "bin"
New-Item -ItemType Directory -Force -Path $Dest | Out-Null
Copy-Item (Join-Path $Root "target\x86_64-pc-windows-msvc\release\plugin.exe") (Join-Path $Dest "plugin.exe") -Force

if ($Pack) {
  $Out = Join-Path $Root "artifacts\plugin"
  New-Item -ItemType Directory -Force -Path $Out | Out-Null
  if (Get-Command streamdeck -ErrorAction SilentlyContinue) {
    streamdeck pack $PluginDir --output $Out --force
  }
}

if ($Install) {
  $AppData = [Environment]::GetFolderPath("ApplicationData")
  $Installed = Join-Path $AppData "Elgato\StreamDeck\Plugins\$PluginId.sdPlugin"
  if (Test-Path $Installed) { Remove-Item -Recurse -Force $Installed }
  Copy-Item -Recurse $PluginDir $Installed
  Write-Host "Installed plugin to $Installed"
}
