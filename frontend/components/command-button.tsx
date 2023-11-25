import { useAsyncCommand } from "@hooks/use-async-command";
import { useLongLoading } from "@hooks/use-long-loading";
import { Button, ButtonProps } from "@mantine/core";
import { forwardRef, useCallback } from "react";
import { dialog } from "@tauri-apps/api";

interface Props<TResult> extends ButtonProps {
	readonly onClick: () => Promise<TResult>;
	readonly onSuccess?: () => void;
	readonly confirmationText?: string;
}

function CommandButtonInternal<TResult>(
	{ onClick, onSuccess, confirmationText, children, ...props }: Props<TResult>,
	ref: React.ForwardedRef<HTMLButtonElement>,
) {
	const [executeCommand, isLoading, success] = useAsyncCommand(
		onClick,
		onSuccess,
	);

	const handleClick = useCallback(async () => {
		if (confirmationText && !(await dialog.confirm(confirmationText))) {
			return;
		}
		executeCommand();
	}, [executeCommand, confirmationText]);

	const isLongLoading = useLongLoading(isLoading);

	return (
		<Button
			ref={ref}
			color="green"
			justify="start"
			loading={isLongLoading}
			variant={success ? "filled" : "default"}
			{...props}
			onClick={handleClick}
		>
			{children}
		</Button>
	);
}

export const CommandButton = forwardRef(CommandButtonInternal);
