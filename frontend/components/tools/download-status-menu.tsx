import { commands, ProgressStatus } from "@api/bindings";
import { Channel } from "@tauri-apps/api/core";
import { Button, Menu } from "@mantine/core";
import { IconClearAll } from "@tabler/icons-react";
import { useEffect, useState } from "react";
import { useLocalization } from "@hooks/use-localization";
import { DownloadProgressRing } from "./download-progress-ring";

type StatusMap = Map<string, ProgressStatus>;

const stepProgress = (status: ProgressStatus) => {
	if (status.total === null || status.current === null) return 0;
	if (status.total === 0) return 0;
	if (status.current >= status.total) return 100;
	const raw = (status.current / status.total) * 100;
	return Number.isFinite(raw) ? raw : 0;
};

export function DownloadStatusMenu() {
	const { t } = useLocalization("downloadStatusMenu");

	const [progresses, setProgresses] = useState<StatusMap>(new Map());
	useEffect(() => {
		commands.listenToDownloadProgress(
			new Channel<ProgressStatus>((status) => {
				setProgresses((previous) => {
					return new Map(previous).set(status.id, status);
				});
			}),
		);
	}, []);

	if (progresses.size === 0) {
		return null;
	}

	const count = progresses.size;
	const totalPercentage = Array.from(progresses.values()).reduce<number>(
		(acc, status) => acc + stepProgress(status) / count,
		0,
	);

	return (
		<Menu
			closeOnItemClick={false}
			keepMounted={true}
			withOverlay={false}
		>
			<Menu.Target>
				<Button
					variant="filled"
					color="dark"
					fz="md"
				>
					<DownloadProgressRing percentage={totalPercentage} />
				</Button>
			</Menu.Target>
			<Menu.Dropdown
				p="xs"
				bg="dark"
			>
				<Menu.Item
					onClick={() => {
						setProgresses((previous) => {
							const newMap: StatusMap = new Map();
							for (const [id, status] of previous.entries()) {
								if (stepProgress(status) < 100) {
									newMap.set(id, status);
								}
							}
							return newMap;
						});
					}}
					leftSection={<IconClearAll />}
				>
					{t("clear")}
				</Menu.Item>
				{[...progresses.entries()].map(([id, status]) => {
					const progress = stepProgress(status);
					return (
						<Menu.Item
							key={id}
							leftSection={<DownloadProgressRing percentage={progress} />}
						>
							{status.name}: {progress.toFixed(2)}%
						</Menu.Item>
					);
				})}
			</Menu.Dropdown>
		</Menu>
	);
}
