import { showAppNotification } from "@components/app-notifications";
import { useEffect, useRef, useState } from "react";

export function useAsyncCommand<TResultValue, TArgs = void>(
	command: (args: TArgs) => Promise<TResultValue>,
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
