param([string]$GitHubOwner = 'arvinai')
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath (Split-Path $PSScriptRoot -Parent)
function Run-Step {
    param([string]$Program, [string[]]$StepArguments)
    & $Program @StepArguments
    if ($LASTEXITCODE -ne 0) { throw ($Program + ' failed: ' + ($StepArguments -join ' ')) }
}
& gh auth status -h github.com
if ($LASTEXITCODE -ne 0) {
    Run-Step -Program gh -StepArguments @('auth', 'login', '-h', 'github.com')
}
$account = & gh api user --jq .login
if ($LASTEXITCODE -ne 0 -or $account.Trim() -ne $GitHubOwner) {
    throw ('Expected GitHub account: ' + $GitHubOwner + '. Select it with gh auth switch before publishing.')
}
Run-Step -Program gh -StepArguments @('auth', 'setup-git')
$branch = & git branch --show-current
if ($LASTEXITCODE -ne 0 -or $branch.Trim() -ne 'fix/windows-pen-barrel-v070') {
    throw 'Publish from fix/windows-pen-barrel-v070 only.'
}
# With an explicit repository argument, gh supports --clone=false but rejects
# --remote. Configure the remotes below after verifying the fork's parent.
Run-Step -Program gh -StepArguments @('repo', 'fork', 'storytold/photocraft', '--clone=false')
$forkName = $GitHubOwner + '/photocraft'
$forkParent = & gh api ('repos/' + $forkName) --jq .parent.full_name
if ($LASTEXITCODE -ne 0 -or $forkParent.Trim() -ne 'storytold/photocraft') {
    throw 'The destination must be a fork of storytold/photocraft.'
}
$remotes = @(& git remote)
if ($remotes -contains 'upstream') {
    $upstreamUrl = & git remote get-url upstream
    if ($upstreamUrl.Trim() -ne 'https://github.com/storytold/photocraft.git') { throw 'Unexpected upstream remote.' }
} else {
    Run-Step -Program git -StepArguments @('remote', 'add', 'upstream', 'https://github.com/storytold/photocraft.git')
}
$forkUrl = 'https://github.com/' + $forkName + '.git'
if ($remotes -contains 'origin') {
    $originUrl = & git remote get-url origin
    if ($originUrl.Trim() -notin @('https://github.com/storytold/photocraft.git', $forkUrl)) { throw 'Unexpected origin remote.' }
    Run-Step -Program git -StepArguments @('remote', 'set-url', 'origin', $forkUrl)
} else {
    Run-Step -Program git -StepArguments @('remote', 'add', 'origin', $forkUrl)
}
Run-Step -Program git -StepArguments @('push', '-u', 'origin', 'fix/windows-pen-barrel-v070')
$localSha = (& git rev-parse HEAD).Trim()
$remoteSha = & gh api ('repos/' + $forkName + '/git/ref/heads/fix/windows-pen-barrel-v070') --jq .object.sha
if ($LASTEXITCODE -ne 0 -or $remoteSha.Trim() -ne $localSha) { throw 'Remote commit verification failed.' }
Write-Host ('PUBLISHED AND VERIFIED: https://github.com/' + $forkName + '/tree/fix/windows-pen-barrel-v070')
Write-Host 'This publishes the tested v0.7.0 reference. It does not open an overlapping upstream PR.'
