param (
  [Parameter(Mandatory=$true)]
  [string]$Changelog
)

function CheckEnvVar {
    param (
      [Parameter(Mandatory=$true)]
      [string]$Var
    )

    if (-not (Test-Path Env:$Var)) {
        Write-Error "Environment variable '$Var' is not defined."
        exit 1
    }
}

# TODO: uncomment
# CheckEnvVar -Var "TAURI_PRIVATE_KEY"
# CheckEnvVar -Var "TAURI_KEY_PASSWORD"

# TODO: uncomment
# pnpm build

$msiFolder = "./backend/target/release/bundle/msi"

# Get built msi file name.
$msiName = (Get-ChildItem -Path $msiFolder -Filter "*.msi" | Select-Object -First 1).Name

# Extract version number from file name.
$version = [regex]::Match($msiName, '.+_(.+)_.+_.+').Groups[1]

# Read signature from sig file.
$signature = Get-Content -Path "$msiFolder/*.sig" -Raw

# Create json that's used by Rai Pal for checking updates.
$updaterJson = ConvertTo-Json -InputObjec @{
    version = "${version}"
    notes = "${Changelog}"
    platforms = @{
      "windows-x86_64" = @{
        signature = "${signature}"
        url = "https://github.com/Raicuparta/rai-pal/releases/latest/download/updater.zip"
    }
  }
}

# Write modified content to the output file
$updaterJson | Set-Content -Path "$msiFolder/latest.json"

# Rename files to make it a bit clearer what should be downloaded,
# and remove version numbers from the download file. This is important
# because we want to be able to use GitHub's magic releases/latest links
# to directly link to the latest release download.

# Move-Item "$msiFolder/*.msi" "$msiFolder/RaiPal.msi"
# Move-Item "$msiFolder/*.zip" "$msiFolder/updater.zip"
# Move-Item "$msiFolder/*.sig" "$msiFolder/signature.sig"
