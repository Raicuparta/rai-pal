function Check-EnvVariables {
    param (
        [string[]]$EnvVariableNames
    )

    foreach ($envVariableName in $EnvVariableNames) {
        # Check if the environment variable is defined
        if (-not (Test-Path Env:$envVariableName)) {
            Write-Error "Environment variable '$envVariableName' is not defined."
            exit 1  # Exit the script with an error code
        }

        # Access the value of the environment variable
        $envVariableValue = $env:$envVariableName

        # Use the environment variable value here...
        Write-Host "Environment variable '$envVariableName' is defined with value: $envVariableValue"
    }
}
