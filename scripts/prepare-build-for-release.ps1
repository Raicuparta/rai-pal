# This takes an existing build, and prepares the files for publishing.
# You can run this locally if you want o test the publishing process,
# although you'll usually be missing stuff like the updater signature and changelog.
# 
# Reads from the following environment variables:
#
# - TAURI_SIGNING_PRIVATE_KEY. See https://v2.tauri.app/plugin/updater/
# - TAURI_SIGNING_PRIVATE_KEY_PASSWORD. Password for the key above.
# - RAI_PAL_CHANGELOG. Changelog that gets included in the update json,
# which gets shown to users whenever they get notified of an update.

param(
    [Parameter(Mandatory=$true)]
    [string]$bundleFolder,

    [Parameter(Mandatory=$true)]
    [string]$outputFolder
)

# Get built bundle file name.
$exeName = (Get-ChildItem -Path $bundleFolder -Filter "*.exe" | Select-Object -First 1).Name

# Extract version number from file name.
$version = [regex]::Match($exeName, '.+_(.+)_.+').Groups[1].Value

$signatureFile = Get-ChildItem -Path $bundleFolder -Filter "*zip.sig" -ErrorAction SilentlyContinue | Select-Object -First 1
$signature = if ($signatureFile) { Get-Content -Path $signatureFile.FullName -Raw } else { "[NOT PROVIDED]" }

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

# Recreate publish folder.
Remove-Item $outputFolder -Force -Recurse -ErrorAction Ignore | Out-Null
New-Item -ItemType Directory $outputFolder | Out-Null

# Write updater json to latest.json (that's the name Tauri uses).
$updaterJson | Set-Content -Path "$outputFolder/latest.json"

# Rename files to make it a bit clearer what should be downloaded,
# and remove version numbers frothe download file. This is important
# because we want to be able to use GitHub's magic releases/latest links
# to directly link to the latest release download.
Copy-Item "$bundleFolder/*.exe" "$outputFolder/RaiPal.exe" -Force
Copy-Item "$bundleFolder/*.zip" "$outputFolder/updater.zip" -Force

return $version
