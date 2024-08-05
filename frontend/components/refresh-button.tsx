import { Button } from "@mantine/core";
import { IconRefresh } from "@tabler/icons-react";
import { useAtomValue } from "jotai";
import { useUpdateData } from "@hooks/use-update-data";
import { loadingCountAtom } from "@hooks/use-data";

export function RefreshButton() {
	const loadingCount = useAtomValue(loadingCountAtom);
	const updateAppData = useUpdateData();

	return (
		<Button
			leftSection={<IconRefresh />}
			loading={loadingCount > 0}
			onClick={updateAppData}
			variant="filled"
		>
			Refresh
		</Button>
	);
}
