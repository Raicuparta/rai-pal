import { Box, Button, ButtonGroup, Popover } from "@mantine/core";
import {
	IconChevronDown,
	IconDownload,
	IconRefresh,
} from "@tabler/icons-react";
import { useAtomValue } from "jotai";
import { useUpdateData } from "@hooks/use-update-data";
import { loadingTasksAtom } from "@hooks/use-data";
import styles from "./components.module.css";
import { useLocalization } from "@hooks/use-localization";

export function RefreshButton() {
	const loadingTasks = useAtomValue(loadingTasksAtom);
	const updateAppData = useUpdateData();
	const t = useLocalization("refresh");

	const isLoading = loadingTasks.length > 0;

	return (
		<Box pos="relative">
			<ButtonGroup
				w={200}
				opacity={isLoading ? 0.5 : 1}
			>
				<Button
					leftSection={<IconRefresh />}
					loading={isLoading}
					onClick={() => updateAppData(false)}
					variant="filled"
					flex={1}
				>
					{t("button")}
				</Button>
				{!isLoading && (
					<Popover position="bottom-end">
						<Popover.Target>
							<Button
								variant="light"
								p={5}
							>
								<IconChevronDown />
							</Button>
						</Popover.Target>
						<Popover.Dropdown>
							<Button
								variant="filled"
								onClick={() => updateAppData(true)}
								disabled={isLoading}
								leftSection={<IconDownload />}
							>
								{t("buttonUpdateRemoteDatabases")}
							</Button>
						</Popover.Dropdown>
					</Popover>
				)}
			</ButtonGroup>
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
