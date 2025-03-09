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
import { usePersistedState } from "@hooks/use-persisted-state";
import { Result } from "@api/bindings";
import { useLocalization } from "@hooks/use-localization";

interface Props<TResultValue, TError> extends ButtonProps {
	readonly onClick: () => Promise<Result<TResultValue, TError> | TResultValue>;
	readonly onSuccess?: (result: TResultValue) => void;
	readonly confirmationText?: string;
	readonly confirmationSkipId?: string;
}

function CommandButtonInternal<TResultValue, TError>(
	{
		onClick,
		onSuccess,
		confirmationText,
		confirmationSkipId,
		children,
		...props
	}: Props<TResultValue, TError>,
	ref: React.ForwardedRef<HTMLButtonElement>,
) {
	const t = useLocalization("commandButton");
	const [isConfirmationOpen, setIsConfirmationOpen] = useState(false);
	const [shouldSkipConfirm, setShouldSkipConfirm] = usePersistedState(
		false,
		`skip-confirm-${confirmationSkipId}`,
	);
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
		setShouldSkipConfirm(dontAskAgain);
		executeCommandWithSuccessCallback();
		closeConfirmation();
	};

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
