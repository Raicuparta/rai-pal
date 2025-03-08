import { Box, Button } from "@mantine/core";
import { IconRefresh } from "@tabler/icons-react";
import { useAtomValue } from "jotai";
import { useUpdateData } from "@hooks/use-update-data";
import { loadingTasksAtom } from "@hooks/use-data";
import styles from "./components.module.css";
import { useLocalization } from "@hooks/use-localization";

export function RefreshButton() {
	const loadingTasks = useAtomValue(loadingTasksAtom);
	const updateAppData = useUpdateData();
	const t = useLocalization("refresh");

	return (
		<Box pos="relative">
			<Button
				leftSection={<IconRefresh />}
				loading={loadingTasks.length > 0}
				onClick={updateAppData}
				variant="filled"
				w={200}
			>
				{t("button")}
			</Button>
			{loadingTasks.length > 0 && (
				<div className={styles.refreshProgress}>
					{t("loading", {
						items: loadingTasks.map((task) => task.name).join(", "),
					})}
				</div>
			)}
		</Box>
	);
}
