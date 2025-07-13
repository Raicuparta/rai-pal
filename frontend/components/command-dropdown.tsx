import { Button, Popover, Stack } from "@mantine/core";
import { IconChevronDown } from "@tabler/icons-react";

type Props = {
	readonly children: React.ReactNode;
	readonly icon?: React.ReactNode;
	readonly label?: string;
};

export function CommandDropdown(props: Props) {
	return (
		<Popover>
			<Popover.Target>
				<Button
					px="xs"
					leftSection={props.label ? props.icon : undefined}
					rightSection={props.label ? <IconChevronDown /> : undefined}
				>
					{props.label ?? props.icon ?? <IconChevronDown />}
				</Button>
			</Popover.Target>
			<Popover.Dropdown>
				<Stack gap="xs">{props.children}</Stack>
			</Popover.Dropdown>
		</Popover>
	);
}
