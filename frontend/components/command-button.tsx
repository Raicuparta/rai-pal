import { useAsyncCommand } from "@hooks/use-async-command";
import { useLongLoading } from "@hooks/use-long-loading";
import { Button, ButtonProps } from "@mantine/core";
import { forwardRef } from "react";

interface Props<TResult> extends ButtonProps {
	readonly onClick: () => Promise<TResult>;
	readonly onSuccess?: () => void;
}

function CommandButtonInternal<TResult>(
	{ onClick, onSuccess, children, ...props }: Props<TResult>,
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
