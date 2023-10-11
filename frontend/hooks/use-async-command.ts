import { useCallback, useRef, useState } from "react";

export function useAsyncCommand<TResult>(
	command: () => Promise<TResult>,
	onSuccess?: (result: TResult) => void,
) {
	const [isLoading, setIsLoading] = useState(false);
	const [error, setError] = useState("");
	const [success, setSuccess] = useState(false);
	const timeout = useRef<number>();

	const clearError = useCallback(() => setError(""), []);

	const executeCommand = useCallback(async () => {
		setIsLoading(true);
		setSuccess(false);
		setError("");

		if (timeout.current) {
			clearTimeout(timeout.current);
		}

		return command()
			.then((result) => {
				if (onSuccess) {
					onSuccess(result);
				}
				setSuccess(true);
				timeout.current = setTimeout(() => {
					setSuccess(false);
				}, 1000);
				return result;
			})
			.catch((error) => setError(`Failed to execute command: ${error}`))
			.finally(() => setIsLoading(false));
	}, [command, onSuccess]);

	return [executeCommand, isLoading, success, error, clearError] as const;
}
