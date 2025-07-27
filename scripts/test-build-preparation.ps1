# Uses an existing build and prepares it for publishing.

# Folder where the nsis bundle will end up.
$bundleFolder = Join-Path $PSScriptRoot "../backend/target/release/bundle/nsis"
$outputFolder = Join-Path $PSScriptRoot "../publish"

$version = & (Join-Path $PSScriptRoot "prepare-build-for-release.ps1") -bundleFolder $bundleFolder -outputFolder $outputFolder

return $version
