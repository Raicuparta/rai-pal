# This takes an existing build, and prepares the files for publishing.
# You can run this locally if you want to test the publishing process,
# although you'll usually be missing stuff like the updater signature.
# 
# Reads from the following sources:
#
# - TAURI_SIGNING_PRIVATE_KEY. See https://v2.tauri.app/plugin/updater/
# - TAURI_SIGNING_PRIVATE_KEY_PASSWORD. Password for the key above.
# - Changelog from tracked file: changelogs/v{version}.md

param(
  [Parameter(Mandatory = $true)]
  [string]$bundleFolder,

  [Parameter(Mandatory = $true)]
  [string]$outputFolder
)

$exeName = (Get-ChildItem -Path $bundleFolder -Filter "*.exe" | Select-Object -First 1).Name

$version = [regex]::Match($exeName, '.+_(.+)_.+').Groups[1].Value

$signatureFile = Get-ChildItem -Path $bundleFolder -Filter "*zip.sig" -ErrorAction SilentlyContinue | Select-Object -First 1
$signature = if ($signatureFile) { Get-Content -Path $signatureFile.FullName -Raw } else { "[NOT PROVIDED]" }

$changelogFile = Join-Path $PSScriptRoot "..\changelogs\v${version}.md"
if (Test-Path $changelogFile) {
  $changelog = Get-Content -Path $changelogFile -Raw
  $changelog = $changelog.Trim()
}
else {
  $changelog = "No changelog file found for version ${version}. Expected file: changelogs\v${version}.md"
}

$releaseExeName = "RaiPal.exe"
$releaseUpdaterName = "updater.zip"

# Create json that's used by Rai Pal for checking updates.
$updaterJson = ConvertTo-Json -InputObject @{
  version   = "${version}"
  notes     = "${changelog}" -replace '"', '\"'
  platforms = @{
    "windows-x86_64" = @{
      signature = "${signature}"
      url       = "https://github.com/Raicuparta/rai-pal/releases/download/v${version}/${releaseUpdaterName}"
    }
  }
}

Remove-Item $outputFolder -Force -Recurse -ErrorAction Ignore | Out-Null
New-Item -ItemType Directory $outputFolder | Out-Null

${updaterJson} | Set-Content -Path "${outputFolder}/latest.json"

$releaseBody = @"
$changelog

[![Download Rai Pal for Windows](https://shields.io/badge/-Download_Rai_Pal_for_Windows-8A2BE2?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/Raicuparta/rai-pal/releases/download/v${version}/${releaseExeName})
"@

${releaseBody} | Set-Content -Path "${outputFolder}/ReleaseBody.md"

# Rename files to make it a bit clearer what should be downloaded,
# and remove version numbers from the download file. This is important
# because we want to be able to use GitHub's magic releases/latest links
# to directly link to the latest release download.
Copy-Item "${bundleFolder}/*.exe" "${outputFolder}/${releaseExeName}" -Force
Copy-Item "${bundleFolder}/*.zip" "${outputFolder}/${releaseUpdaterName}" -Force

return $version
