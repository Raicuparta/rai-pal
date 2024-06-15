import { Button, Group, Indicator, Popover } from "@mantine/core";
import { IconFilter, IconX } from "@tabler/icons-react";
import styles from "./filters.module.css";

type Props = {
	readonly children: React.ReactNode;
	readonly active: boolean;
	readonly setFilter: (filter: undefined) => void;
};

export function FilterMenu(props: Props) {
	return (
		<Indicator
			disabled={!props.active}
			offset={8}
		>
			<Button.Group>
				{props.active && (
					<Button
						onClick={() => props.setFilter(undefined)}
						px={5}
					>
						<IconX />
					</Button>
				)}
				<Popover
					trapFocus
					position="bottom-end"
				>
					<Popover.Target>
						<Button leftSection={<IconFilter />}>Filter</Button>
					</Popover.Target>
					<Popover.Dropdown
						bg="dark"
						p={0}
						className={styles.dropdown}
					>
						<Group
							className={styles.dropdownContent}
							p="xs"
							align="start"
							wrap="nowrap"
						>
							{props.children}
						</Group>
					</Popover.Dropdown>
				</Popover>
			</Button.Group>
		</Indicator>
	);
}
