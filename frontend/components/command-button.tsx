import { useAsyncCommand } from "@hooks/use-async-command";
import { useLongLoading } from "@hooks/use-long-loading";
import {
	Button,
	ButtonProps,
	Checkbox,
	Group,
	Paper,
	Popover,
	Stack,
} from "@mantine/core";
import { forwardRef, useState } from "react";
import { IconArrowBack, IconCheck } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";
import { useAppSettingSingle } from "@hooks/use-app-setting-single";

interface Props<TResultValue> extends ButtonProps {
	readonly onClick: () => Promise<TResultValue>;
	readonly onSuccess?: (result: TResultValue) => void;
	readonly confirmationText?: string;
	readonly confirmationSkipId?: string;
}

function CommandButtonInternal<TResultValue>(
	{
		onClick,
		onSuccess,
		confirmationText,
		confirmationSkipId,
		children,
		...props
	}: Props<TResultValue>,
	ref: React.ForwardedRef<HTMLButtonElement>,
) {
	const t = useLocalization("commandButton");
	const [isConfirmationOpen, setIsConfirmationOpen] = useState(false);
	const [skipConfirmDialogs, setSkipConfirmDialogs] =
		useAppSettingSingle("skipConfirmDialogs");
	const confirmDialogId = `skip-confirm-${confirmationSkipId}`;
	const shouldSkipConfirm = skipConfirmDialogs.includes(confirmDialogId);

	const [dontAskAgain, setDontAskAgain] = useState(false);
	const [executeCommand, isLoading, success] = useAsyncCommand(onClick);

	const executeCommandWithSuccessCallback = () =>
		executeCommand().then(onSuccess);

	const handleClick = () => {
		if (confirmationText && !shouldSkipConfirm) {
			setIsConfirmationOpen(true);
		} else {
			executeCommandWithSuccessCallback();
		}
	};

	const closeConfirmation = () => {
		setIsConfirmationOpen(false);
		setDontAskAgain(false);
	};

	const confirm = () => {
		if (dontAskAgain) {
			setSkipConfirmDialogs((prev) => {
				if (prev.includes(confirmDialogId)) return prev;
				return [...prev, confirmDialogId];
			});
		}
		executeCommandWithSuccessCallback();
		closeConfirmation();
	};

	const isLongLoading = useLongLoading(isLoading);

	return (
		<Popover
			trapFocus
			opened={isConfirmationOpen}
			onClose={closeConfirmation}
			disabled={!confirmationText || shouldSkipConfirm}
		>
			<Popover.Target>
				<Button
					ref={ref}
					loading={isLongLoading}
					{...props}
					color={success ? "green" : props.color}
					variant={success ? "filled" : props.variant}
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
						<Group>
							<Button
								leftSection={<IconArrowBack />}
								onClick={closeConfirmation}
							>
								{t("cancel")}
							</Button>
							<Button
								leftSection={<IconCheck />}
								onClick={confirm}
							>
								{children}
							</Button>
						</Group>
						{confirmationSkipId && (
							<Checkbox
								checked={dontAskAgain}
								onChange={(event) =>
									setDontAskAgain(event.currentTarget.checked)
								}
								label={t("dontAskAgain")}
							/>
						)}
					</Stack>
				</Paper>
			</Popover.Dropdown>
		</Popover>
	);
}

export const CommandButton = forwardRef(CommandButtonInternal);
