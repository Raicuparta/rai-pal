import { Button } from "@mantine/core";
import { IconRefresh } from "@tabler/icons-react";
import { useAtomValue } from "jotai";
import { useUpdateData } from "@hooks/use-update-data";
import { loadingAtom } from "@hooks/use-data";

export function RefreshButton() {
	const isLoading = useAtomValue(loadingAtom);
	const updateAppData = useUpdateData();

	return (
		<Button
			leftSection={<IconRefresh />}
			loading={isLoading}
			onClick={updateAppData}
			style={{ flex: 1, maxWidth: "10em" }}
			variant="filled"
		>
			Refresh
		</Button>
	);
}
