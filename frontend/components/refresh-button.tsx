import { useLongLoading } from "@hooks/use-long-loading";
import { Button } from "@mantine/core";
import { IconRefresh } from "@tabler/icons-react";

type Props = {
	readonly onClick: () => void;
	readonly loading: boolean;
};

export function RefreshButton(props: Props) {
	const isLoading = useLongLoading(props.loading);

	return (
		<Button
			leftSection={<IconRefresh />}
			loading={isLoading}
			onClick={props.onClick}
			style={{ flex: 1, maxWidth: "10em" }}
			variant="filled"
		>
			{isLoading ? "Loading..." : "Refresh"}
		</Button>
	);
}
