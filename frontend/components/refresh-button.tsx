import { Button } from "@mantine/core";
import { MdRefresh } from "react-icons/md";

type Props = {
	readonly onClick: () => void;
	readonly loading: boolean;
};

export function RefreshButton(props: Props) {
	return (
		<Button
			leftSection={<MdRefresh />}
			loading={props.loading}
			onClick={props.onClick}
			style={{ flex: 1, maxWidth: 200 }}
			variant="filled"
		>
			{props.loading ? "Loading..." : "Refresh"}
		</Button>
	);
}
