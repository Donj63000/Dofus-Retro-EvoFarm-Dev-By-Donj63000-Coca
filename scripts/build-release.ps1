$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$projectRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $projectRoot "Cargo.toml"
$targetTriple = "x86_64-pc-windows-msvc"
$targetDirectory = if ($env:CARGO_TARGET_DIR) {
    if ([System.IO.Path]::IsPathRooted($env:CARGO_TARGET_DIR)) { $env:CARGO_TARGET_DIR }
    else { Join-Path $projectRoot $env:CARGO_TARGET_DIR }
} else { Join-Path $projectRoot "target" }
$sourceExe = Join-Path $targetDirectory "$targetTriple\release\evofarm.exe"
$targetExe = Join-Path $projectRoot "EvoFarm.exe"
$zipPath = Join-Path $projectRoot "EvoFarm-Windows-x64.zip"
$pendingZipPath = Join-Path $projectRoot "EvoFarm-Windows-x64.build.zip"
$sha256Path = Join-Path $projectRoot "SHA256SUMS"

# Je distribue uniquement le programme portable et sa documentation, sans donnees personnelles.
$zipItems = @("EvoFarm.exe", "README.txt")
foreach ($optionalDocument in @("LICENSE", "LICENSE.txt", "LICENSE.md")) {
    if (Test-Path -LiteralPath (Join-Path $projectRoot $optionalDocument) -PathType Leaf) {
        $zipItems += $optionalDocument
    }
}
foreach ($item in @("logo.png", "fond.png", "README.txt")) {
    if (-not (Test-Path -LiteralPath (Join-Path $projectRoot $item) -PathType Leaf)) {
        throw "Un fichier requis pour la distribution est absent : $item"
    }
}

# Je refuse une signature partiellement configuree au lieu de publier un resultat ambigu.
$signingSettings = @($env:SIGNTOOL_EXE, $env:SIGN_CERT_PATH, $env:SIGN_CERT_PASSWORD)
$configuredSigningSettings = @($signingSettings | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }).Count
if ($configuredSigningSettings -ne 0 -and $configuredSigningSettings -ne 3) {
    throw "Signature incomplete : renseigner SIGNTOOL_EXE, SIGN_CERT_PATH et SIGN_CERT_PASSWORD ensemble."
}

# Je bloque uniquement les executables que cette distribution doit reconstruire ou remplacer.
$protectedExecutablePaths = @(
    [System.IO.Path]::GetFullPath($sourceExe),
    [System.IO.Path]::GetFullPath($targetExe)
)
foreach ($candidateProcess in @(Get-Process EvoFarm -ErrorAction SilentlyContinue)) {
    try {
        $runningPath = $candidateProcess.Path
        if ([string]::IsNullOrWhiteSpace($runningPath)) { continue }
        $normalizedRunningPath = [System.IO.Path]::GetFullPath($runningPath)
    }
    catch {
        # Je laisse Cargo et la copie signaler un verrou si le chemin du processus est inaccessible.
        continue
    }
    if ($protectedExecutablePaths -icontains $normalizedRunningPath) {
        throw "EvoFarm.exe est en cours d'utilisation : $normalizedRunningPath. Fermez cette instance avant de relancer le packaging."
    }
}

Push-Location -LiteralPath $projectRoot
try {
    & cargo build --manifest-path $manifestPath --locked --release --target $targetTriple
    if ($LASTEXITCODE -ne 0) {
        throw "La compilation d'EvoFarm a echoue (code $LASTEXITCODE). Aucun artefact n'a ete publie."
    }
    if (-not (Test-Path -LiteralPath $sourceExe -PathType Leaf) -or (Get-Item -LiteralPath $sourceExe).Length -eq 0) {
        throw "La compilation n'a pas produit l'executable attendu : $sourceExe"
    }

    # Je signe le produit compile avant de remplacer les fichiers de distribution precedents.
    if ($configuredSigningSettings -eq 3) {
        & $env:SIGNTOOL_EXE sign /fd SHA256 /f $env:SIGN_CERT_PATH /p $env:SIGN_CERT_PASSWORD /tr https://timestamp.digicert.com /td SHA256 $sourceExe
        if ($LASTEXITCODE -ne 0) { throw "La signature Authenticode a echoue (code $LASTEXITCODE)." }
        & $env:SIGNTOOL_EXE verify /pa $sourceExe
        if ($LASTEXITCODE -ne 0) { throw "La verification Authenticode a echoue (code $LASTEXITCODE)." }
    }
    else {
        Write-Host "Aucun certificat configure : executable non signe, empreintes SHA-256 fournies."
    }

    Copy-Item -LiteralPath $sourceExe -Destination $targetExe -Force

    # Je remplace l'archive precedente uniquement lorsque la nouvelle archive est complete.
    & tar.exe -a -cf $pendingZipPath @zipItems
    if ($LASTEXITCODE -ne 0) {
        throw "La creation de l'archive a echoue (code $LASTEXITCODE)."
    }
    if (-not (Test-Path -LiteralPath $pendingZipPath -PathType Leaf) -or (Get-Item -LiteralPath $pendingZipPath).Length -eq 0) {
        throw "L'archive de distribution est absente ou vide."
    }
    Move-Item -LiteralPath $pendingZipPath -Destination $zipPath -Force

    $checksums = foreach ($artifact in @($targetExe, $zipPath)) {
        $hash = (Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $([System.IO.Path]::GetFileName($artifact))"
    }
    [System.IO.File]::WriteAllLines($sha256Path, $checksums, [System.Text.Encoding]::ASCII)
}
finally {
    Pop-Location
    if (Test-Path -LiteralPath $pendingZipPath -PathType Leaf) {
        Remove-Item -LiteralPath $pendingZipPath -Force
    }
}

Write-Host "Executable cree : $targetExe"
Write-Host "Archive portable creee : $zipPath"
Write-Host "Empreintes creees : $sha256Path"
