import { Button } from "@mantine/core";
import { IconFilterX } from "@tabler/icons-react";

type Props = {
	readonly setFilter: (filter: undefined) => void;
};

export function FilterResetButton(props: Props) {
	return (
		<Button
			color="pink"
			leftSection={<IconFilterX />}
			onClick={() => props.setFilter(undefined)}
			variant="light"
		>
			Reset
		</Button>
	);
}
