import { Result } from "@api/result";
import { useAsyncCommand } from "@hooks/use-async-command";
import { useLongLoading } from "@hooks/use-long-loading";
import { Button, ButtonProps } from "@mantine/core";
import { forwardRef } from "react";

interface Props<TData, TError> extends ButtonProps {
	readonly onClick: () => Promise<Result<TData, TError>>;
	readonly onSuccess?: () => void;
}

function CommandButtonInternal<TData, TError>(
	{ onClick, onSuccess, children, ...props }: Props<TData, TError>,
	ref: React.ForwardedRef<HTMLButtonElement>,
) {
	const [executeCommand, isLoading, success] = useAsyncCommand(
		onClick,
		onSuccess,
	);

	const isLongLoading = useLongLoading(isLoading);

	return (
		<Button
			ref={ref}
			color="green"
			justify="start"
			loading={isLongLoading}
			variant={success ? "filled" : "default"}
			{...props}
			onClick={executeCommand}
		>
			{children}
		</Button>
	);
}

export const CommandButton = forwardRef(CommandButtonInternal);
