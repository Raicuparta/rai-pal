import { Result } from "@api/bindings";
import { showAppNotification } from "@components/app-notifications";
import { useEffect, useRef, useState } from "react";

// This hook helps dealing with async commands given by the backend,
// parsing the Result types that the backend uses.
// It can also be used with any async function that doesn't return a
// Result type, as a wrapper that adds loading and success states.
export function useAsyncCommand<TResultValue, TError, TArgs = void>(
	command: (
		args: TArgs,
	) => Promise<Result<TResultValue, TError> | TResultValue>,
) {
	const [isLoading, setIsLoading] = useState(false);
	const [success, setSuccess] = useState(false);
	const timeout = useRef<number>(undefined);

	const executeCommand: (args: TArgs) => Promise<TResultValue> = async (
		args,
	) => {
		setIsLoading(true);
		setSuccess(false);

		clearTimeout(timeout.current);

		return command(args)
			.then((result) => {
				if (result && typeof result === "object" && "status" in result) {
					if (result.status === "error") {
						throw new Error(JSON.stringify(result.error));
					}

					return result.data;
				}

				return result;
			})
			.then((result) => {
				setSuccess(true);

				timeout.current = setTimeout(() => {
					setSuccess(false);
				}, 1000) as unknown as number;

				return result;
			})
			.catch((error) => {
				showAppNotification(`Failed to execute command: ${error}`, "error");
				throw error;
			})
			.finally(() => setIsLoading(false));
	};

	useEffect(() => {
		return () => {
			clearTimeout(timeout.current);
		};
	}, []);

	return [executeCommand, isLoading, success] as const;
}
