# This will create a release build with all the artifacts ready for publishing.

function CheckEnvVar {
  param (
    [Parameter(Mandatory = $true)]
    [string]$Var
  )

  if (-not (Test-Path Env:$Var)) {
    Write-Error "Environment variable '$Var' is not defined."
    exit 1
  }
}

CheckEnvVar -Var "TAURI_SIGNING_PRIVATE_KEY"
CheckEnvVar -Var "TAURI_SIGNING_PRIVATE_KEY_PASSWORD"

# Folder where the nsis bundle will end up.
$bundleFolder = Join-Path $PSScriptRoot "../backend/target/release/bundle/nsis"
$outputFolder = Join-Path $PSScriptRoot "../publish"

# Delete everything in the bundle folder first.
Remove-Item $bundleFolder -Force -Recurse -ErrorAction Ignore | Out-Null

npm run build

$version = & (Join-Path $PSScriptRoot "prepare-build-for-release.ps1") -bundleFolder $bundleFolder -outputFolder $outputFolder

return $version
