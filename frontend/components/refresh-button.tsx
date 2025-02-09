import { Box, Button } from "@mantine/core";
import { IconRefresh } from "@tabler/icons-react";
import { useAtomValue } from "jotai";
import { useUpdateData } from "@hooks/use-update-data";
import { loadingTasksAtom } from "@hooks/use-data";

export function RefreshButton() {
	const loadingTasks = useAtomValue(loadingTasksAtom);
	const updateAppData = useUpdateData();

	return (
		<Box pos="relative">
			<Button
				leftSection={<IconRefresh />}
				loading={loadingTasks.length > 0}
				onClick={updateAppData}
				variant="filled"
				w={200}
			>
				Refresh
			</Button>
			{loadingTasks.length > 0 && (
				<Box
					pos="absolute"
					bottom={0}
					left={10}
					fz={10}
					opacity={0.5}
				>
					Loading {loadingTasks.map((task) => task.name).join(", ")}...
				</Box>
			)}
		</Box>
	);
}
