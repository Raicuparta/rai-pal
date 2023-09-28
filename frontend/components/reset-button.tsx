import { Button } from "@mantine/core";
import { MdFilterAltOff } from "react-icons/md";

type Props = {
	readonly setFilter: (filter: undefined) => void;
};

export function ResetButton(props: Props) {
	return (
		<Button
			color="pink"
			leftSection={<MdFilterAltOff />}
			onClick={() => props.setFilter(undefined)}
			variant="light"
		>
			Reset
		</Button>
	);
}
