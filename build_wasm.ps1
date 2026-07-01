param(
  [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"

Write-Host "Building for wasm32-unknown-unknown ($Profile)..." -ForegroundColor Cyan

if ($Profile -eq "release") {
  cargo build --target wasm32-unknown-unknown --release
  $target_dir = "target\wasm32-unknown-unknown\release"
} else {
  cargo build --target wasm32-unknown-unknown
  $target_dir = "target\wasm32-unknown-unknown\debug"
}

if ($LASTEXITCODE -ne 0) {
  Write-Host "Build failed!" -ForegroundColor Red
  exit $LASTEXITCODE
}

$wasm_file = "$target_dir\siege_factory.wasm"
if (-not (Test-Path $wasm_file)) {
  Write-Host "WASM file not found at $wasm_file" -ForegroundColor Red
  exit 1
}

Write-Host "Running wasm-bindgen..." -ForegroundColor Cyan
wasm-bindgen --out-dir web --target web $wasm_file

if ($LASTEXITCODE -ne 0) {
  Write-Host "wasm-bindgen failed!" -ForegroundColor Red
  exit $LASTEXITCODE
}

Copy-Item "index.html" "web\index.html" -Force

Write-Host "Done! Open web/index.html in a browser or serve with: npx serve web" -ForegroundColor Green
