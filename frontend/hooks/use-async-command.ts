import { useCallback, useState } from "react";

export function useAsyncCommand<TResult>(command: () => Promise<TResult>) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState(false);

  const clearError = useCallback(() => setError(""), []);

  const executeCommand = useCallback(async () => {
    setIsLoading(true);
    setSuccess(false);
    setError("");

    return command()
      .then((result) => {
        setSuccess(true);
        setTimeout(() => setSuccess(false), 1500);
        return result;
      })
      .catch((error) => setError(`Failed to execute command: ${error}`))
      .finally(() => setIsLoading(false));
  }, [command]);

  return [executeCommand, isLoading, success, error, clearError] as const;
}
