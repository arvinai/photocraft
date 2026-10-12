param([switch]$BuildOnly)
$ErrorActionPreference = 'Stop'
$sourceRoot = Split-Path $PSScriptRoot -Parent
Set-Location -LiteralPath $sourceRoot
$logRoot = Join-Path $sourceRoot 'log'
New-Item -ItemType Directory -Path $logRoot -Force | Out-Null
$installLog = Join-Path $logRoot 'build-install-pen-fix.log'
Start-Transcript -LiteralPath $installLog -Force | Out-Null
trap {
    Write-Host ('BUILD OR INSTALLATION FAILED: ' + $_.Exception.Message) -ForegroundColor Red
    Write-Host ('Diagnostic log: ' + $installLog)
    Stop-Transcript -ErrorAction SilentlyContinue | Out-Null
    exit 1
}
$env:PATH = (Join-Path $env:USERPROFILE '.cargo\bin') + ';' + $env:PATH
$env:CARGO_HOME = Join-Path $sourceRoot '.build-cargo'
$env:CARGO_BUILD_JOBS = '2'
$env:CARGO_PROFILE_TEST_OPT_LEVEL = '0'
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vsPath = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vsPath) { throw 'Visual Studio C++ x64 tools and a Windows SDK are required.' }
$vcvars = Join-Path $vsPath 'VC\Auxiliary\Build\vcvars64.bat'
& cmd.exe /d /s /c ('"' + $vcvars + '" >nul && set') | ForEach-Object {
    if ($_ -match '^([^=]+)=(.*)$') { Set-Item -Path ('Env:' + $matches[1]) -Value $matches[2] }
}
if (-not $env:HTTPS_PROXY) {
    $url = [uri]'https://index.crates.io/config.json'
    $proxy = [System.Net.WebRequest]::DefaultWebProxy.GetProxy($url)
    if ($proxy -ne $url) { $env:HTTPS_PROXY = $proxy.AbsoluteUri; $env:HTTP_PROXY = $proxy.AbsoluteUri }
}
if (Test-Path -LiteralPath (Join-Path $sourceRoot 'craft-fonts')) {
    $env:CRAFT_FONTS_DIR = Join-Path $sourceRoot 'craft-fonts'
}
function Run-Cargo {
    param([Parameter(Mandatory = $true)][string[]]$CargoArguments)
    # Pass Cargo's literal "--" through Windows PowerShell's function parser.
    # Stream native stderr to the transcript as text, then check the exit code.
    $ErrorActionPreference = 'Continue'
    & cargo @CargoArguments 2>&1 | ForEach-Object { Write-Host $_ }
    $cargoExitCode = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    if ($cargoExitCode -ne 0) { throw ('Cargo failed: ' + ($CargoArguments -join ' ')) }
}
New-Item -ItemType Directory -Path (Join-Path $sourceRoot 'target') -Force | Out-Null
& rustc --test scripts/pen-button-mapping-tests.rs -o target/pen-button-mapping-tests.exe
if ($LASTEXITCODE -ne 0) { throw 'Pen mapping tests did not compile.' }
& .\target\pen-button-mapping-tests.exe
if ($LASTEXITCODE -ne 0) { throw 'Pen mapping tests failed.' }
Run-Cargo -CargoArguments @('fmt', '--all', '--', '--check')
Run-Cargo -CargoArguments @('test', '-p', 'photocraft-ui-egui', '--lib')
Run-Cargo -CargoArguments @('test', '-p', 'photocraft', '--bin', 'photocraft')
Run-Cargo -CargoArguments @('clippy', '-p', 'photocraft-ui-egui', '-p', 'photocraft', '--all-targets', '--', '-D', 'warnings')
Run-Cargo -CargoArguments @('xtask', 'layers')
Run-Cargo -CargoArguments @('xtask', 'wasm')
Run-Cargo -CargoArguments @('build', '--release', '-p', 'photocraft', '--features', 'photocraft/heif')
if ($BuildOnly) { Write-Host 'Build finished: target\release\photocraft.exe'; Stop-Transcript | Out-Null; exit 0 }

$officialRoot = 'D:\PSfree\app\photocraft-0.7.0-windows-x64-portable'
$installRoot = 'D:\PSfree\app\photocraft-0.7.0-pen-button-fix-20261012'
if (-not (Test-Path -LiteralPath (Join-Path $officialRoot 'photocraft.exe'))) { throw 'Official v0.7.0 install was not found.' }
# A fresh directory avoids overwriting a running application or its settings.
if (Test-Path -LiteralPath $installRoot) { throw ('Install directory already exists; inspect it before reuse: ' + $installRoot) }
New-Item -ItemType Directory -Path $installRoot | Out-Null
Get-ChildItem -LiteralPath $officialRoot -File | Where-Object { $_.Name -ne 'photocraft.exe' } | Copy-Item -Destination $installRoot
Copy-Item -LiteralPath (Join-Path $sourceRoot 'target\release\photocraft.exe') -Destination $installRoot
$oldData = Join-Path $officialRoot 'PhotoCraftData'
$newData = Join-Path $installRoot 'PhotoCraftData'
if (Test-Path -LiteralPath $oldData) {
    & robocopy $oldData $newData /E /XD Recovery /NFL /NDL /NJH /NJS /NP
    if ($LASTEXITCODE -gt 7) { throw 'Settings copy failed.' }
    $prefs = 'preferences.json'
    if (Test-Path -LiteralPath (Join-Path $oldData $prefs)) {
        if ((Get-FileHash -LiteralPath (Join-Path $oldData $prefs)).Hash -ne (Get-FileHash -LiteralPath (Join-Path $newData $prefs)).Hash) {
            throw 'Settings verification failed.'
        }
    }
}
Copy-Item -LiteralPath (Join-Path $sourceRoot 'vendor\winit\LICENSE') -Destination (Join-Path $installRoot 'LICENSE-winit.txt')
Copy-Item -LiteralPath (Join-Path $sourceRoot 'docs\windows-pen-button-fix.md') -Destination $installRoot
if ((Get-FileHash -LiteralPath (Join-Path $sourceRoot 'target\release\photocraft.exe')).Hash -ne (Get-FileHash -LiteralPath (Join-Path $installRoot 'photocraft.exe')).Hash) {
    throw 'Installed binary verification failed.'
}
$ws = New-Object -ComObject WScript.Shell
$link = $ws.CreateShortcut('D:\PSfree\启动PhotoCraft.lnk')
$link.TargetPath = Join-Path $installRoot 'photocraft.exe'
$link.WorkingDirectory = $installRoot
$link.IconLocation = $link.TargetPath + ',0'
$link.Description = 'PhotoCraft v0.7.0 Windows pen barrel-button fix'
$link.Save()
Write-Host ('Installed: ' + $installRoot)
Write-Host 'Save and close the old app, then use D:\PSfree\启动PhotoCraft.lnk.'
Write-Host 'Test the Wacom barrel button hovering and touching; then test a normal pressure stroke.'

Stop-Transcript | Out-Null
