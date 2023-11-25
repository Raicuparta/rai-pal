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

interface Props<TResult> extends ButtonProps {
	readonly onClick: () => Promise<TResult>;
	readonly onSuccess?: () => void;
	readonly confirmationText?: string;
}

function CommandButtonInternal<TResult>(
	{ onClick, onSuccess, confirmationText, children, ...props }: Props<TResult>,
	ref: React.ForwardedRef<HTMLButtonElement>,
) {
	const [isConfirmationOpen, setIsConfirmationOpen] = useState(false);
	const [executeCommand, isLoading, success] = useAsyncCommand(
		onClick,
		onSuccess,
	);

	const handleClick = useCallback(() => {
		if (confirmationText) {
			setIsConfirmationOpen((isOpen) => !isOpen);
		} else {
			executeCommand();
		}
	}, [executeCommand, confirmationText]);

	const closeConfirmation = useCallback(() => {
		setIsConfirmationOpen(false);
	}, []);

	const confirm = useCallback(() => {
		executeCommand();
		setIsConfirmationOpen(false);
	}, [executeCommand]);

	const isLongLoading = useLongLoading(isLoading);

	return (
		<Popover
			trapFocus
			withArrow
			shadow="md"
			opened={isConfirmationOpen}
			onClose={closeConfirmation}
			disabled={!confirmationText}
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
						{/* <Checkbox label="Don't ask again" /> */}
					</Stack>
				</Paper>
			</Popover.Dropdown>
		</Popover>
	);
}

export const CommandButton = forwardRef(CommandButtonInternal);
