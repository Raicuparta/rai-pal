param (
  [Parameter(Mandatory = $true)]
  [string]$Changelog
)

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

npm run build

$msiFolder = "./backend/target/release/bundle/msi"

# Get built msi file name.
$msiName = (Get-ChildItem -Path $msiFolder -Filter "*.msi" | Select-Object -First 1).Name

# Extract version number from file name.
$version = [regex]::Match($msiName, '.+_(.+)_.+_.+').Groups[1].Value

# Read signature from sig file.
$signature = Get-Content -Path "$msiFolder/*.sig" -Raw

# Create json that's used by Rai Pal for checking updates.
$updaterJson = ConvertTo-Json -InputObjec @{
  version   = "${version}"
  notes     = "${Changelog}" -replace '"', '\"'
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
