import { useAsyncCommand } from "@hooks/use-async-command";
import { useLongLoading } from "@hooks/use-long-loading";
import {
	Button,
	ButtonProps,
	Checkbox,
	Flex,
	Paper,
	Popover,
	Stack,
} from "@mantine/core";
import { forwardRef, useCallback, useState } from "react";
import { IconArrowBack, IconCheck } from "@tabler/icons-react";
import { usePersistedState } from "@hooks/use-persisted-state";

interface Props<TResult> extends ButtonProps {
	readonly onClick: () => Promise<TResult>;
	readonly onSuccess?: () => void;
	readonly confirmationText?: string;
	readonly confirmationSkipId?: string;
}

function CommandButtonInternal<TResult>(
	{
		onClick,
		onSuccess,
		confirmationText,
		confirmationSkipId,
		children,
		...props
	}: Props<TResult>,
	ref: React.ForwardedRef<HTMLButtonElement>,
) {
	const [isConfirmationOpen, setIsConfirmationOpen] = useState(false);
	const [shouldSkipConfirm, setShouldSkipConfirm] = usePersistedState(
		false,
		`skip-confirm-${confirmationSkipId}`,
	);
	const [dontAskAgain, setDontAskAgain] = useState(false);
	const [executeCommand, isLoading, success] = useAsyncCommand(
		onClick,
		onSuccess,
	);

	const handleClick = useCallback(() => {
		if (confirmationText && !shouldSkipConfirm) {
			setIsConfirmationOpen(true);
		} else {
			executeCommand();
		}
	}, [confirmationText, shouldSkipConfirm, executeCommand]);

	const closeConfirmation = useCallback(() => {
		setIsConfirmationOpen(false);
		setDontAskAgain(false);
	}, []);

	const confirm = useCallback(() => {
		setShouldSkipConfirm(dontAskAgain);
		executeCommand();
		closeConfirmation();
	}, [closeConfirmation, dontAskAgain, executeCommand, setShouldSkipConfirm]);

	const isLongLoading = useLongLoading(isLoading);

	return (
		<Popover
			trapFocus
			withArrow
			shadow="md"
			opened={isConfirmationOpen}
			onClose={closeConfirmation}
			disabled={!confirmationText || shouldSkipConfirm}
		>
			<Popover.Target>
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
			</Popover.Target>
			<Popover.Dropdown
				p={5}
				maw={600}
			>
				<Paper
					color="dark"
					p="xs"
				>
					<Stack align="center">
						{confirmationText}
						<Flex
							gap="md"
							justify="center"
						>
							<Button
								leftSection={<IconArrowBack />}
								onClick={closeConfirmation}
							>
								Cancel
							</Button>
							<Button
								leftSection={<IconCheck />}
								onClick={confirm}
								{...props}
							>
								{children}
							</Button>
						</Flex>
						{confirmationSkipId && (
							<Checkbox
								checked={dontAskAgain}
								onChange={(event) =>
									setDontAskAgain(event.currentTarget.checked)
								}
								label="Don't ask again"
							/>
						)}
					</Stack>
				</Paper>
			</Popover.Dropdown>
		</Popover>
	);
}

export const CommandButton = forwardRef(CommandButtonInternal);
