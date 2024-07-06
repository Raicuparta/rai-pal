import { Result } from "@api/bindings";
import { showAppNotification } from "@components/app-notifications";
import { useCallback, useRef, useState } from "react";

export function useAsyncCommand<TResultValue, TError, TArgs = void>(
	command: (args: TArgs) => Promise<Result<TResultValue, TError>>,
	onSuccess?: (result: TResultValue) => void,
) {
	const [isLoading, setIsLoading] = useState(false);
	const [success, setSuccess] = useState(false);
	const timeout = useRef<number>();

	const executeCommand = useCallback(
		async (args: TArgs) => {
			setIsLoading(true);
			setSuccess(false);

			if (timeout.current) {
				clearTimeout(timeout.current);
			}

			return command(args)
				.then((result) => {
					if (result.status === "error") {
						throw new Error(JSON.stringify(result.error));
					}

					if (onSuccess) {
						onSuccess(result.data);
					}
					setSuccess(true);

					timeout.current = setTimeout(() => {
						setSuccess(false);
					}, 1000);

					return result.data;
				})
				.catch((error) => {
					showAppNotification(`Failed to execute command: ${error}`, "error");
					return null;
				})
				.finally(() => setIsLoading(false));
		},
		[command, onSuccess],
	);

	return [executeCommand, isLoading, success] as const;
}
