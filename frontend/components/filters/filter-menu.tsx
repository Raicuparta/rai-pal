import { Button, Group, Indicator, Popover } from "@mantine/core";
import { IconFilter, IconX } from "@tabler/icons-react";

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
				<Popover trapFocus>
					<Popover.Target>
						<Button leftSection={<IconFilter />}>Filter</Button>
					</Popover.Target>
					<Popover.Dropdown bg="dark">
						<Group align="start">{props.children}</Group>
					</Popover.Dropdown>
				</Popover>
			</Button.Group>
		</Indicator>
	);
}
