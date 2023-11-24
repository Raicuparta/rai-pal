import { Result } from "@api/result";
import { notifications } from "@mantine/notifications";
import { useCallback, useRef, useState } from "react";

export function useAsyncCommand<TData, TArgs, TError>(
	command: (args: TArgs) => Promise<Result<TData, TError>>,
	onSuccess?: (result: TData) => void,
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
						throw result.error;
					} else {
						if (onSuccess) {
							onSuccess(result.data);
						}
						setSuccess(true);
						timeout.current = setTimeout(() => {
							setSuccess(false);
						}, 1000);
						return result;
					}
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
