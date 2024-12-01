# This script reads from the following environment variables:
#
# - TAURI_SIGNING_PRIVATE_KEY. See https://v2.tauri.app/plugin/updater/
# - TAURI_SIGNING_PRIVATE_KEY_PASSWORD. Password for the key above.
# - RAI_PAL_CHANGELOG. Changelog that gets included in the update json,
# which gets shown to users whenever they get notified of an update.

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

# Folder where the msi bundle will end up.
$msiFolder = "./backend/target/release/bundle/msi"

# Delete everything in the msi folder first.
Remove-Item $msiFolder -Force -Recurse -ErrorAction Ignore | Out-Null

npm run build

# Get built msi file name.
$msiName = (Get-ChildItem -Path $msiFolder -Filter "*.msi" | Select-Object -First 1).Name

# Extract version number from file name.
$version = [regex]::Match($msiName, '.+_(.+)_.+_.+').Groups[1].Value

# Read signature from sig file. We deleted everything in this folder before,
# so we're pretty sure nothing other the newly created zip.sig should be found.
$signature = Get-Content -Path "$msiFolder/*zip.sig" -Raw

# Read changelog from environment variable.
$changelog = $env:RAI_PAL_CHANGELOG ?? "Someone forgot to include a changelog."

# Create json that's used by Rai Pal for checking updates.
$updaterJson = ConvertTo-Json -InputObjec @{
  version   = "${version}"
  notes     = "${changelog}" -replace '"', '\"'
  platforms = @{
    "windows-x86_64" = @{
      signature = "${signature}"
      url       = "https://github.com/Raicuparta/rai-pal/releases/download/v${version}/updater.zip"
    }
  }
}

$outputFolder = "./publish"

# Recreate publish folder.
Remove-Item $outputFolder -Force -Recurse -ErrorAction Ignore | Out-Null
New-Item -ItemType Directory $outputFolder | Out-Null

# Write updater json to latest.json (that's the name Tauri uses).
$updaterJson | Set-Content -Path "$outputFolder/latest.json"

# Rename files to make it a bit clearer what should be downloaded,
# and remove version numbers frothe download file. This is important
# because we want to be able to use GitHub's magic releases/latest links
# to directly link to the latest release download.
Copy-Item "$msiFolder/*.msi" "$outputFolder/RaiPal.msi" -Force
Copy-Item "$msiFolder/*.zip" "$outputFolder/updater.zip" -Force

return $version
