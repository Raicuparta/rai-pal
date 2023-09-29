import { useCallback, useState } from "react";
import { useTimeout } from "@mantine/hooks";

export function useAsyncCommand<TResult>(
	command: () => Promise<TResult>,
	onSuccess?: (result: TResult) => void,
) {
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
				if (onSuccess) {
					onSuccess(result);
				}
				setSuccess(true);
				startSuccessTimeout();
				return result;
			})
			.catch((error) => setError(`Failed to execute command: ${error}`))
			.finally(() => setIsLoading(false));
	}, [clearSuccessTimeout, command, startSuccessTimeout, onSuccess]);

	return [executeCommand, isLoading, success, error, clearError] as const;
}
