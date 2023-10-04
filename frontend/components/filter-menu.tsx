import { Button, Indicator, Popover } from "@mantine/core";
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
						p="xs"
					>
						<IconX />
					</Button>
				)}
				<Popover>
					<Popover.Target>
						<Button leftSection={<IconFilter />}>Filter</Button>
					</Popover.Target>
					<Popover.Dropdown>{props.children}</Popover.Dropdown>
				</Popover>
			</Button.Group>
		</Indicator>
	);
}
