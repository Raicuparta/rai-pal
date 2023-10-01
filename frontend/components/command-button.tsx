import { useAsyncCommand } from "@hooks/use-async-command";
import { useLongLoading } from "@hooks/use-long-loading";
import { Button, ButtonProps } from "@mantine/core";
import { ErrorPopover } from "./error-popover";

interface Props<TResult> extends ButtonProps {
	readonly onClick: () => Promise<TResult>;
	readonly onSuccess?: () => void;
}

export function CommandButton<TResult>({ onClick, ...props }: Props<TResult>) {
	const [executeCommand, isLoading, success, error, clearError] =
		useAsyncCommand(onClick, props.onSuccess);

	const isLongLoading = useLongLoading(isLoading);

	return (
		<ErrorPopover
			error={error}
			clearError={clearError}
		>
			<Button
				color={error ? "red" : "green"}
				loading={isLongLoading}
				variant={success || error ? "filled" : "default"}
				{...props}
				onClick={executeCommand}
			/>
		</ErrorPopover>
	);
}
