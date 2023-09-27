import { useCallback, useEffect, useState } from "react";

export function useAsyncCommand<TResult>(command: () => Promise<TResult>) {
  const [isLoading, setIsLoading] = useState(false);
  const [isLongLoading, setIsLongLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState(false);

  const clearError = useCallback(() => setError(""), []);

  const executeCommand = useCallback(async () => {
    setIsLongLoading(false);
    setIsLoading(true);
    setSuccess(false);
    setError("");

    return command()
      .then((result) => {
        setSuccess(true);
        setTimeout(() => setSuccess(false), 1000);
        return result;
      })
      .catch((error) => setError(`Failed to execute command: ${error}`))
      .finally(() => setIsLoading(false));
  }, [command]);

  useEffect(() => {
    setIsLongLoading(false);
    if (!isLoading) return;

    const timeout = setTimeout(() => {
      if (!isLoading) return;
      setIsLongLoading(true);
    }, 100);

    return () => clearTimeout(timeout);
  }, [isLoading]);

  return [executeCommand, isLongLoading, success, error, clearError] as const;
}
