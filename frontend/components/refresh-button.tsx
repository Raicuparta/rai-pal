import { Button } from "@mantine/core";
import { IconRefresh } from "@tabler/icons-react";
import { ErrorPopover } from "./error-popover";

export function RefreshButton() {
	const isLoading = false;
	const error = "";
	const refresh = () => {};
	const clearError = () => {};

	return (
		<ErrorPopover
			error={error}
			clearError={clearError}
		>
			<Button
				leftSection={<IconRefresh />}
				loading={isLoading}
				onClick={refresh}
				style={{ flex: 1, maxWidth: "10em" }}
				variant="filled"
			>
				Refresh
			</Button>
		</ErrorPopover>
	);
}
