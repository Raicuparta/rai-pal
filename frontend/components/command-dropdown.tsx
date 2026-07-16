import { Button, Popover, Stack } from "@mantine/core";
import { IconChevronDown } from "@tabler/icons-react";
import styles from "./components.module.css";

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
				<Stack
					gap="xs"
					className={styles.commandDropdownButtons}
				>
					{props.children}
				</Stack>
			</Popover.Dropdown>
		</Popover>
	);
}
