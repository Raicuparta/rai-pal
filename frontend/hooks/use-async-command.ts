import { useCallback, useEffect, useState } from "react";
import { useTimeout } from "@mantine/hooks";

export function useAsyncCommand<TResult>(command: () => Promise<TResult>) {
	const [isLoading, setIsLoading] = useState(false);
	const [isLongLoading, setIsLongLoading] = useState(false);
	const [error, setError] = useState("");
	const [success, setSuccess] = useState(false);
	const { start: startSuccessTimeout, clear: clearSuccessTimeout } = useTimeout(
		() => {
			setSuccess(false);
		},
		1000,
	);
	const { start: startLoadingTimeout, clear: clearLoadingTimeout } = useTimeout(
		() => {
			if (!isLoading) return;
			setIsLongLoading(true);
		},
		1000,
	);

	const clearError = useCallback(() => setError(""), []);

	const executeCommand = useCallback(async () => {
		clearSuccessTimeout();
		setIsLongLoading(false);
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

	useEffect(() => {
		setIsLongLoading(false);
		if (!isLoading) return;

		startLoadingTimeout();

		return clearLoadingTimeout;
	}, [clearLoadingTimeout, isLoading, startLoadingTimeout]);

	return [executeCommand, isLongLoading, success, error, clearError] as const;
}
