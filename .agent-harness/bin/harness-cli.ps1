$ErrorActionPreference = "Stop"

$RepoRoot = (Get-Location).Path
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$HarnessRoot = Split-Path -Parent $ScriptDir
$Binary = Join-Path $ScriptDir "harness-cli.exe"

if (!(Test-Path $Binary)) {
    throw "Harness CLI binary missing: $Binary"
}

if ([string]::IsNullOrWhiteSpace($env:HARNESS_DB)) {
    $env:HARNESS_DB = Join-Path $RepoRoot "harness.db"
}

if ($args.Count -gt 0 -and ($args[0] -eq "init" -or $args[0] -eq "migrate")) {
    if ([string]::IsNullOrWhiteSpace($env:HARNESS_REPO_ROOT)) {
        $env:HARNESS_REPO_ROOT = $HarnessRoot
    }
}

& $Binary @args
exit $LASTEXITCODE
