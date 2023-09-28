import { useCallback, useState } from "react";
import { useTimeout } from "@mantine/hooks";

export function useAsyncCommand<TResult>(command: () => Promise<TResult>) {
	const [isLoading, setIsLoading] = useState(false);
	const [error, setError] = useState("");
	const [success, setSuccess] = useState(false);
	const { start: startSuccessTimeout, clear: clearSuccessTimeout } = useTimeout(
		() => {
			setSuccess(false);
		},
		1000,
	);

	const clearError = useCallback(() => setError(""), []);

	const executeCommand = useCallback(async () => {
		clearSuccessTimeout();
		setIsLoading(true);
		setSuccess(false);
		setError("");

		return command()
			.then((result) => {
				setSuccess(true);
				startSuccessTimeout();
				return result;
			})
			.catch((error) => setError(`Failed to execute command: ${error}`))
			.finally(() => setIsLoading(false));
	}, [clearSuccessTimeout, command, startSuccessTimeout]);

	return [executeCommand, isLoading, success, error, clearError] as const;
}
