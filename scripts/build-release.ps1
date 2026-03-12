$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $projectRoot "Cargo.toml"
$sourceExe = Join-Path $projectRoot "target\\release\\dofus_rentabilite.exe"
$targetExe = Join-Path $projectRoot "EvoFarm.exe"
$zipPath = Join-Path $projectRoot "evofarm.zip"
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
    "EvoFarm.exe"
)

& cargo build --manifest-path $manifestPath --release

if (-not (Test-Path $sourceExe)) {
    throw "Build succeeded but $sourceExe was not found."
}

$runningProcess = Get-Process EvoFarm -ErrorAction SilentlyContinue
if ($runningProcess) {
    throw "EvoFarm.exe est en cours d'utilisation. Fermez l'application avant de relancer le packaging."
}

Copy-Item $sourceExe $targetExe -Force
Write-Host "Created $targetExe"

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
