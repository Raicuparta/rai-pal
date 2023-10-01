import {
	CloseButton,
	Code,
	Flex,
	Popover,
	Stack,
	ThemeIcon,
} from "@mantine/core";
import { IconExclamationCircle } from "@tabler/icons-react";

type Props = {
	readonly children: React.ReactNode;
	readonly error?: string;
	readonly clearError: () => void;
};

export function ErrorPopover(props: Props) {
	return (
		<Popover
			opened={Boolean(props.error)}
			position="bottom"
			width={400}
		>
			<Popover.Target>
				<span>{props.children}</span>
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
						<CloseButton onClick={props.clearError} />
					</Flex>
					<Code>{props.error}</Code>
				</Stack>
			</Popover.Dropdown>
		</Popover>
	);
}
