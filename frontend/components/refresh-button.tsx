import { Button } from "@mantine/core";
import { IconRefresh } from "@tabler/icons-react";
import { ErrorPopover } from "./error-popover";
import { useAtom, useAtomValue } from "jotai";
import { errorAtom, loadingAtom } from "@hooks/use-app-state";
import { useUpdateAppState } from "@hooks/use-update-state";

export function RefreshButton() {
	const isLoading = useAtomValue(loadingAtom);
	const [error, setError] = useAtom(errorAtom);
	const updateAppState = useUpdateAppState();

	return (
		<ErrorPopover
			error={error}
			clearError={() => setError("")}
		>
			<Button
				leftSection={<IconRefresh />}
				loading={isLoading}
				onClick={updateAppState}
				style={{ flex: 1, maxWidth: "10em" }}
				variant="filled"
			>
				Refresh
			</Button>
		</ErrorPopover>
	);
}
