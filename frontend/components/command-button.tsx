import { useAsyncCommand } from "@hooks/use-async-command";
import {
	Button,
	ButtonProps,
	CloseButton,
	Code,
	Flex,
	Popover,
	Stack,
	ThemeIcon,
} from "@mantine/core";
import { IconExclamationCircle } from "@tabler/icons-react";

interface Props<TResult> extends ButtonProps {
	readonly onClick: () => Promise<TResult>;
	readonly icon: React.ReactNode;
}

export function CommandButton<TResult>({
	onClick,
	icon,
	...props
}: Props<TResult>) {
	const [executeCommand, isLoading, success, error, clearError] =
		useAsyncCommand(onClick);

	return (
		<Popover
			opened={Boolean(error)}
			position="bottom"
			width={400}
		>
			<Popover.Target>
				<Button
					color={error ? "red" : "green"}
					justify="start"
					loading={isLoading}
					variant={success || error ? "filled" : "default"}
					{...props}
					leftSection={icon}
					onClick={executeCommand}
				/>
			</Popover.Target>
			<Popover.Dropdown>
				<Stack>
					<Flex
						justify="space-between"
						color="red"
					>
						<ThemeIcon
							size="sm"
							radius="xl"
							color="red"
						>
							<IconExclamationCircle />
						</ThemeIcon>
						<CloseButton onClick={clearError} />
					</Flex>
					<Code block>{error}</Code>
				</Stack>
			</Popover.Dropdown>
		</Popover>
	);
}
