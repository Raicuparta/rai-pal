import { useAsyncCommand } from "@hooks/use-async-command";
import {
	Box,
	Button,
	ButtonProps,
	CloseButton,
	Code,
	Flex,
	Popover,
	Stack,
} from "@mantine/core";
import { MdError } from "react-icons/md";

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
					<Flex justify="space-between">
						<Box
							c="red"
							component={MdError}
						/>
						<CloseButton onClick={clearError} />
					</Flex>
					<Code block>{error}</Code>
				</Stack>
			</Popover.Dropdown>
		</Popover>
	);
}
