import { Box, Button } from "@mantine/core";
import { IconRefresh } from "@tabler/icons-react";
import { useAtomValue } from "jotai";
import { useUpdateData } from "@hooks/use-update-data";
import { refreshTasksAtom } from "@hooks/use-refresh-tasks";
import styles from "./components.module.css";
import { useLocalization } from "@hooks/use-localization";

export function RefreshButton() {
	const refreshTasks = useAtomValue(refreshTasksAtom);
	const updateAppData = useUpdateData();
	const { t } = useLocalization("refresh");

	const isRefreshing = refreshTasks.length > 0;

	return (
		<Box pos="relative">
			<Button
				w="100%"
				leftSection={<IconRefresh />}
				loading={isRefreshing}
				onClick={() => updateAppData()}
				opacity={isRefreshing ? 0.5 : 1}
				variant="filled"
			>
				{t("button")}
			</Button>
			{isRefreshing && (
				<div className={styles.refreshProgress}>
					{t("loading", {
						items: refreshTasks.map((task) => task.name).join(", "),
					})}
				</div>
			)}
		</Box>
	);
}
