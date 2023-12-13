import { ThemeIcon } from "@mantine/core";
import { IconRefreshAlert } from "@tabler/icons-react";

export function OutdatedMarker() {
	return (
		<ThemeIcon
			color="orange"
			radius="xl"
			size="sm"
		>
			<IconRefreshAlert fontSize={15} />
		</ThemeIcon>
	);
}
