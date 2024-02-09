function CheckEnvVar {
    param (
        [string]$Var
    )

    if (-not (Test-Path Env:$Var)) {
        Write-Error "Environment variable '$Var' is not defined."
        exit 1
    }
}

CheckEnvVar -Var "TAURI_PRIVATE_KEY"
CheckEnvVar -Var "TAURI_KEY_PASSWORD"

pnpm build
