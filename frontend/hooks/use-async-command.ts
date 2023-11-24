import { notifications } from "@mantine/notifications";
import { useCallback, useRef, useState } from "react";

export function useAsyncCommand<TResult, TArgs = void>(
	command: (args: TArgs) => Promise<TResult>,
	onSuccess?: (result: TResult) => void,
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
					if (onSuccess) {
						onSuccess(result);
					}
					setSuccess(true);
					timeout.current = setTimeout(() => {
						setSuccess(false);
					}, 1000);
					return result;
				})
				.catch((error) =>
					notifications.show({
						message: `Failed to execute command: ${error}`,
						color: "red",
					}),
				)
				.finally(() => setIsLoading(false));
		},
		[command, onSuccess],
	);

	return [executeCommand, isLoading, success] as const;
}
