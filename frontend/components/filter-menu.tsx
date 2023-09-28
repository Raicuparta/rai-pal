import { Button, Indicator, Popover } from "@mantine/core";
import { IconFilter } from "@tabler/icons-react";

type Props = {
	readonly children: React.ReactNode;
	readonly active: boolean;
};

export function FilterMenu(props: Props) {
	return (
		<Popover>
			<Popover.Target>
				<Indicator
					disabled={!props.active}
					offset={8}
				>
					<Button leftSection={<IconFilter />}>Filter</Button>
				</Indicator>
			</Popover.Target>
			<Popover.Dropdown>{props.children}</Popover.Dropdown>
		</Popover>
	);
}
