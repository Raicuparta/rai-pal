import { CodeHighlight } from "@mantine/code-highlight";
import { CloseButton, Flex, Popover, Stack, ThemeIcon } from "@mantine/core";
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
			width={600}
			withinPortal={false}
		>
			<Popover.Target>{props.children}</Popover.Target>
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
					<CodeHighlight
						language="md"
						code={props.error ?? ""}
						styles={{ code: { whiteSpace: "break-spaces" } }}
					/>
				</Stack>
			</Popover.Dropdown>
		</Popover>
	);
}
