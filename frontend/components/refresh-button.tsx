import { Button } from "@mantine/core";
import { IconRefresh } from "@tabler/icons-react";
import { ErrorPopover } from "./error-popover";
import { useAppStore } from "@hooks/use-app-state";

export function RefreshButton() {
	const isLoading = useAppStore((state) => state.isLoading);
	const error = useAppStore((state) => state.error);
	const refresh = useAppStore((store) => store.updateState);
	const clearError = useAppStore((store) => store.clearError);

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
