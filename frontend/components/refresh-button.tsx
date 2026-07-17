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
	const { t } = useLocalization("refresh");

	const isLoading = loadingTasks.length > 0;

	return (
		<Box pos="relative">
			<Button
				w={200}
				leftSection={<IconRefresh />}
				loading={isLoading}
				onClick={() => updateAppData()}
				opacity={isLoading ? 0.5 : 1}
				variant="filled"
				flex={1}
			>
				{t("button")}
			</Button>
			{isLoading && (
				<div className={styles.refreshProgress}>
					{t("loading", {
						items: loadingTasks.map((task) => task.name).join(", "),
					})}
				</div>
			)}
		</Box>
	);
}
