$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$nativeTarPath = (Get-Command tar.exe -CommandType Application).Source
$sourceScript = Join-Path $PSScriptRoot "build-release.ps1"
$tempParent = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
$fixtureRoot = Join-Path $tempParent ("EvoFarm-packaging-tests-" + [guid]::NewGuid().ToString("N"))

$global:EvoFarmPackagingTestState = @{ nativeTarPath = $nativeTarPath }
# Je neutralise la configuration de compilation et de signature pendant les tests isoles.
$preservedEnvironment = @{}
foreach ($variable in @("CARGO_TARGET_DIR", "SIGNTOOL_EXE", "SIGN_CERT_PATH", "SIGN_CERT_PASSWORD")) {
    $preservedEnvironment[$variable] = [Environment]::GetEnvironmentVariable($variable, "Process")
    [Environment]::SetEnvironmentVariable($variable, $null, "Process")
}

function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) { throw $Message }
}

# Je simule Cargo pour tester les erreurs et les anciens binaires sans compiler ni toucher au projet.
function cargo {
    $global:EvoFarmPackagingTestState.cargoArguments = @($args)
    if ($global:EvoFarmPackagingTestState.produceExecutable) {
        [System.IO.File]::WriteAllText($global:EvoFarmPackagingTestState.sourceExe, "new executable")
    }
    $global:LASTEXITCODE = $global:EvoFarmPackagingTestState.cargoExitCode
}

function Get-Process {
    param([string]$Name, [string]$ErrorAction)
    if ($global:EvoFarmPackagingTestState.processRunning) {
        $process = [pscustomobject]@{ ProcessName = $Name }
        if ($global:EvoFarmPackagingTestState.unreadablePath) {
            $process | Add-Member -MemberType ScriptProperty -Name Path -Value { throw "Acces refuse." }
        }
        else {
            $processPath = if ($global:EvoFarmPackagingTestState.processPath) {
                Join-Path $global:EvoFarmPackagingTestState.caseRoot $global:EvoFarmPackagingTestState.processPath
            } else { $null }
            $process | Add-Member -MemberType NoteProperty -Name Path -Value $processPath
        }
        $process
    }
}

function tar.exe {
    $global:EvoFarmPackagingTestState.tarWasCalled = $true
    if ($global:EvoFarmPackagingTestState.tarExitCode -ne 0) {
        [System.IO.File]::WriteAllText((Join-Path $global:EvoFarmPackagingTestState.caseRoot "EvoFarm-Windows-x64.build.zip"), "incomplete archive")
        $global:LASTEXITCODE = $global:EvoFarmPackagingTestState.tarExitCode
        return
    }
    & $global:EvoFarmPackagingTestState.nativeTarPath @args
    $global:LASTEXITCODE = $LASTEXITCODE
}

function EvoFarmTestSignTool {
    $global:EvoFarmPackagingTestState.signToolCalls += @($args[0])
    $global:LASTEXITCODE = if ($args[0] -eq "sign") {
        $global:EvoFarmPackagingTestState.signExitCode
    } else { $global:EvoFarmPackagingTestState.verifyExitCode }
}

function Invoke-PackagingCase {
    param(
        [string]$Name,
        [int]$CargoExit = 0,
        [int]$TarExit = 0,
        [bool]$ProduceExe = $true,
        [bool]$Running = $false,
        [string]$ProcessPath = "EvoFarm.exe",
        [bool]$UnreadablePath = $false,
        [string]$MissingItem = "",
        [string]$TargetDirectory = "",
        [bool]$IncludeLicense = $false,
        [string]$SigningMode = "",
        [int]$SignExit = 0,
        [int]$VerifyExit = 0,
        [string]$ExpectedError = ""
    )

    $global:EvoFarmPackagingTestState.caseRoot = Join-Path $fixtureRoot $Name
    $global:EvoFarmPackagingTestState.cargoExitCode = $CargoExit
    $global:EvoFarmPackagingTestState.tarExitCode = $TarExit
    $global:EvoFarmPackagingTestState.produceExecutable = $ProduceExe
    $global:EvoFarmPackagingTestState.processRunning = $Running
    $global:EvoFarmPackagingTestState.processPath = $ProcessPath
    $global:EvoFarmPackagingTestState.unreadablePath = $UnreadablePath
    $global:EvoFarmPackagingTestState.tarWasCalled = $false
    $global:EvoFarmPackagingTestState.cargoArguments = @()
    $global:EvoFarmPackagingTestState.signToolCalls = @()
    $global:EvoFarmPackagingTestState.signExitCode = $SignExit
    $global:EvoFarmPackagingTestState.verifyExitCode = $VerifyExit
    $env:CARGO_TARGET_DIR = if ($TargetDirectory -eq "absolute") {
        Join-Path $global:EvoFarmPackagingTestState.caseRoot "external-target"
    } else { $TargetDirectory }
    $caseTarget = if ($env:CARGO_TARGET_DIR) {
        if ([IO.Path]::IsPathRooted($env:CARGO_TARGET_DIR)) { $env:CARGO_TARGET_DIR }
        else { Join-Path $global:EvoFarmPackagingTestState.caseRoot $env:CARGO_TARGET_DIR }
    } else { Join-Path $global:EvoFarmPackagingTestState.caseRoot "target" }
    $global:EvoFarmPackagingTestState.sourceExe = Join-Path $caseTarget "x86_64-pc-windows-msvc\release\evofarm.exe"
    New-Item -ItemType Directory -Path (Split-Path -Parent $global:EvoFarmPackagingTestState.sourceExe) -Force | Out-Null
    $env:SIGNTOOL_EXE = if ($SigningMode) { "EvoFarmTestSignTool" } else { $null }
    $env:SIGN_CERT_PATH = if ($SigningMode -eq "complete") { "fixture-certificate.pfx" } else { $null }
    $env:SIGN_CERT_PASSWORD = if ($SigningMode -eq "complete") { "fixture-password" } else { $null }

    foreach ($directory in @("src", "classes", "scripts", "build_support", "tests", "target\x86_64-pc-windows-msvc\release")) {
        New-Item -ItemType Directory -Path (Join-Path $global:EvoFarmPackagingTestState.caseRoot $directory) -Force | Out-Null
    }
    Copy-Item -LiteralPath $sourceScript -Destination (Join-Path $global:EvoFarmPackagingTestState.caseRoot "scripts\build-release.ps1")
    foreach ($file in @("logo.png", "fond.png", "Cargo.toml", "Cargo.lock", "build.rs", "README.txt", ".gitignore", "user-data.json", "unused.png")) {
        [System.IO.File]::WriteAllText((Join-Path $global:EvoFarmPackagingTestState.caseRoot $file), "fixture")
    }
    [System.IO.File]::WriteAllText((Join-Path $global:EvoFarmPackagingTestState.caseRoot "EvoFarm.exe"), "previous executable")
    [System.IO.File]::WriteAllText((Join-Path $global:EvoFarmPackagingTestState.caseRoot "EvoFarm-Windows-x64.zip"), "previous archive")
    [System.IO.File]::WriteAllText((Join-Path $global:EvoFarmPackagingTestState.caseRoot "SHA256SUMS"), "previous checksums")
    if ($IncludeLicense) {
        [System.IO.File]::WriteAllText((Join-Path $global:EvoFarmPackagingTestState.caseRoot "LICENSE"), "fixture license")
    }
    if ($MissingItem) {
        Remove-Item -LiteralPath (Join-Path $global:EvoFarmPackagingTestState.caseRoot $MissingItem) -Force
    }
    $locationBefore = (Get-Location).Path
    $actualError = ""
    try {
        & (Join-Path $global:EvoFarmPackagingTestState.caseRoot "scripts\build-release.ps1") *> $null
    }
    catch {
        $actualError = $_.Exception.Message
    }

    Assert-True ((Get-Location).Path -eq $locationBefore) "$Name : le dossier courant a change."
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $global:EvoFarmPackagingTestState.caseRoot "EvoFarm-Windows-x64.build.zip"))) "$Name : une archive incomplete subsiste."
    if ($ExpectedError) {
        Assert-True ($actualError -like "*$ExpectedError*") "$Name : erreur inattendue '$actualError'."
        Assert-True ([System.IO.File]::ReadAllText((Join-Path $global:EvoFarmPackagingTestState.caseRoot "EvoFarm-Windows-x64.zip")) -eq "previous archive") "$Name : l'ancienne archive a ete alteree."
        Assert-True ([System.IO.File]::ReadAllText((Join-Path $global:EvoFarmPackagingTestState.caseRoot "SHA256SUMS")) -eq "previous checksums") "$Name : les anciennes empreintes ont ete alterees."
        if ($TarExit -eq 0) {
            Assert-True (-not $global:EvoFarmPackagingTestState.tarWasCalled) "$Name : tar ne devait pas etre lance."
            Assert-True ([System.IO.File]::ReadAllText((Join-Path $global:EvoFarmPackagingTestState.caseRoot "EvoFarm.exe")) -eq "previous executable") "$Name : l'ancien executable a ete altere."
        }
        if ($Running) {
            Assert-True ($global:EvoFarmPackagingTestState.cargoArguments.Count -eq 0) "$Name : Cargo ne devait pas etre lance."
        }
    }
    else {
        Assert-True (-not $actualError) "$Name : $actualError"
        Assert-True ($global:EvoFarmPackagingTestState.cargoArguments -contains "--locked") "$Name : Cargo.lock n'est pas impose."
        Assert-True ($global:EvoFarmPackagingTestState.cargoArguments -contains "--release") "$Name : la compilation n'est pas en release."
        Assert-True ([System.IO.File]::ReadAllText((Join-Path $global:EvoFarmPackagingTestState.caseRoot "EvoFarm.exe")) -eq "new executable") "$Name : l'executable n'a pas ete remplace."
        $entries = @(& $nativeTarPath -tf (Join-Path $global:EvoFarmPackagingTestState.caseRoot "EvoFarm-Windows-x64.zip"))
        Assert-True ($LASTEXITCODE -eq 0) "$Name : l'archive ne peut pas etre lue."
        foreach ($required in @("EvoFarm.exe", "README.txt")) {
            Assert-True ($entries -contains $required) "$Name : $required absent de l'archive."
        }
        $expectedEntryCount = if ($IncludeLicense) { 3 } else { 2 }
        Assert-True ($entries.Count -eq $expectedEntryCount) "$Name : l'archive contient des fichiers exclus."
        if ($IncludeLicense) { Assert-True ($entries -contains "LICENSE") "$Name : licence absente." }
        Assert-True ($global:EvoFarmPackagingTestState.cargoArguments -contains "x86_64-pc-windows-msvc") "$Name : la cible Windows x64 n'est pas imposee."
        $checksumLines = [System.IO.File]::ReadAllLines((Join-Path $global:EvoFarmPackagingTestState.caseRoot "SHA256SUMS"))
        Assert-True ($checksumLines.Count -eq 2) "$Name : deux empreintes sont requises."
        foreach ($artifactName in @("EvoFarm.exe", "EvoFarm-Windows-x64.zip")) {
            $hash = (Get-FileHash -LiteralPath (Join-Path $global:EvoFarmPackagingTestState.caseRoot $artifactName) -Algorithm SHA256).Hash.ToLowerInvariant()
            Assert-True ($checksumLines -contains "$hash  $artifactName") "$Name : empreinte incorrecte pour $artifactName."
        }
        if ($SigningMode -eq "complete") {
            Assert-True (($global:EvoFarmPackagingTestState.signToolCalls -join ",") -eq "sign,verify") "$Name : la signature doit etre verifiee."
        }
    }
    Write-Host "OK : $Name"
}

try {
    Invoke-PackagingCase -Name "compilation-error" -CargoExit 23 -ExpectedError "compilation d'EvoFarm a echoue"
    Invoke-PackagingCase -Name "archive-error" -TarExit 17 -ExpectedError "creation de l'archive a echoue"
    Invoke-PackagingCase -Name "missing-executable" -ProduceExe $false -ExpectedError "executable attendu"
    Invoke-PackagingCase -Name "missing-logo" -MissingItem "logo.png" -ExpectedError "fichier requis"
    Invoke-PackagingCase -Name "running-root-application" -Running $true -ExpectedError "en cours d'utilisation"
    Invoke-PackagingCase -Name "running-release-application" -Running $true -ProcessPath "target\x86_64-pc-windows-msvc\release\..\release\EVOFARM.exe" -ExpectedError "en cours d'utilisation"
    Invoke-PackagingCase -Name "running-debug-application" -Running $true -ProcessPath "target\debug\evofarm.exe"
    Invoke-PackagingCase -Name "running-other-project" -Running $true -ProcessPath "other-project\EvoFarm.exe"
    Invoke-PackagingCase -Name "running-unknown-path" -Running $true -ProcessPath ""
    Invoke-PackagingCase -Name "running-unreadable-path" -Running $true -UnreadablePath $true
    Invoke-PackagingCase -Name "complete-distribution"
    Invoke-PackagingCase -Name "missing-documentation" -MissingItem "README.txt" -ExpectedError "fichier requis"
    Invoke-PackagingCase -Name "relative-target-directory" -TargetDirectory "custom-target"
    Invoke-PackagingCase -Name "absolute-target-directory" -TargetDirectory "absolute"
    Invoke-PackagingCase -Name "portable-license" -IncludeLicense $true
    Invoke-PackagingCase -Name "partial-signing" -SigningMode "partial" -ExpectedError "Signature incomplete"
    Invoke-PackagingCase -Name "signature-error" -SigningMode "complete" -SignExit 9 -ExpectedError "signature Authenticode a echoue"
    Invoke-PackagingCase -Name "signature-verification-error" -SigningMode "complete" -VerifyExit 8 -ExpectedError "verification Authenticode a echoue"
    Invoke-PackagingCase -Name "signed-distribution" -SigningMode "complete"
    Invoke-PackagingCase -Name "Dofus Retro EvoFarm [Dev By Donj63000(Coca)]"
    Write-Host "20 tests de packaging reussis."
}
finally {
    foreach ($variable in $preservedEnvironment.Keys) {
        [Environment]::SetEnvironmentVariable($variable, $preservedEnvironment[$variable], "Process")
    }
    # Je verifie le chemin absolu avant de supprimer uniquement les fixtures de ce test.
    $resolvedFixtureRoot = [System.IO.Path]::GetFullPath($fixtureRoot)
    $requiredPrefix = $tempParent.TrimEnd('\', '/') + [System.IO.Path]::DirectorySeparatorChar
    if (-not $resolvedFixtureRoot.StartsWith($requiredPrefix, [System.StringComparison]::OrdinalIgnoreCase) -or
        [System.IO.Path]::GetFileName($resolvedFixtureRoot) -notlike "EvoFarm-packaging-tests-*") {
        throw "Le dossier de test sort du repertoire temporaire attendu."
    }
    if (Test-Path -LiteralPath $resolvedFixtureRoot) {
        Remove-Item -LiteralPath $resolvedFixtureRoot -Recurse -Force
    }
}
