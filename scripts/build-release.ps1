$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $projectRoot "Cargo.toml"
$sourceExe = Join-Path $projectRoot "target\release\evofarm.exe"
$targetExe = Join-Path $projectRoot "EvoFarm.exe"
$sha256Path = Join-Path $projectRoot "EvoFarm.exe.sha256"
$zipPath = Join-Path $projectRoot "EvoFarm.zip"

$zipItems = @(
    "src",
    "classes",
    "fond",
    "icone",
    "scripts",
    "Cargo.toml",
    "Cargo.lock",
    "build.rs",
    "README.txt",
    ".gitignore",
    "EvoFarm.exe",
    "EvoFarm.exe.sha256"
)

& cargo build --manifest-path $manifestPath --release --locked

if (-not (Test-Path $sourceExe)) {
    throw "Build succeeded but $sourceExe was not found."
}

$runningProcess = Get-Process EvoFarm -ErrorAction SilentlyContinue
if ($runningProcess) {
    throw "EvoFarm.exe est en cours d'utilisation. Fermez l'application avant de relancer le packaging."
}

Copy-Item $sourceExe $targetExe -Force
Write-Host "Created $targetExe"

if (-not $env:SIGNTOOL_EXE) {
    throw "SIGNTOOL_EXE manquant."
}
if (-not $env:SIGN_CERT_PATH) {
    throw "SIGN_CERT_PATH manquant."
}
if (-not $env:SIGN_CERT_PASSWORD) {
    throw "SIGN_CERT_PASSWORD manquant."
}

& $env:SIGNTOOL_EXE sign `
    /fd SHA256 `
    /f $env:SIGN_CERT_PATH `
    /p $env:SIGN_CERT_PASSWORD `
    /tr http://timestamp.digicert.com `
    /td SHA256 `
    $targetExe

& $env:SIGNTOOL_EXE verify $targetExe
Write-Host "Signed and verified $targetExe"

$hash = (Get-FileHash $targetExe -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -Path $sha256Path -Value "$hash  EvoFarm.exe" -Encoding Ascii -NoNewline
Write-Host "Created $sha256Path"

if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}

Push-Location $projectRoot
try {
    & tar.exe -a -cf $zipPath @zipItems
}
finally {
    Pop-Location
}

Write-Host "Created $zipPath"
