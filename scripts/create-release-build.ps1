# This will create a release build with all the artifacts ready for publishing.

function Get-RelativePath {
  param (
    [Parameter(Mandatory = $true)]
    [string]$Path
  )
  return Join-Path $PSScriptRoot $Path
}

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
$bundleFolder = Get-RelativePath "../backend/target/release/bundle/nsis"
$outputFolder = Get-RelativePath "../publish"

# Delete everything in the bundle folder first.
Remove-Item $bundleFolder -Force -Recurse -ErrorAction Ignore | Out-Null

npm run build

$version = & (Get-RelativePath "prepare-build-for-release.ps1") -bundleFolder $bundleFolder -outputFolder $outputFolder

return $version
