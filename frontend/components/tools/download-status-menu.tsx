import { commands, DownloadStatus } from "@api/bindings";
import { Channel } from "@tauri-apps/api/core";
import { Button, Menu } from "@mantine/core";
import { IconClearAll } from "@tabler/icons-react";
import { useEffect, useState } from "react";
import { useLocalization } from "@hooks/use-localization";
import { DownloadProgressRing } from "./download-progress-ring";

type StatusMap = Map<string, DownloadStatus>;

export function DownloadStatusMenu() {
	const t = useLocalization("downloadStatusMenu");

	const [progresses, setProgresses] = useState<StatusMap>(new Map());
	useEffect(() => {
		commands.listenToDownloadProgress(
			new Channel<DownloadStatus>((status) => {
				setProgresses((previous) => {
					return new Map(previous).set(
						`${status.url}:${status.targetPath}`,
						status,
					);
				});
			}),
		);
	}, []);

	if (progresses.size === 0) {
		return null;
	}

	const totalPercentage = Array.from(progresses.values()).reduce<number>(
		(acc, status) => {
			// Skip if total size is unknown or download finished.
			if (status.total !== null && status.downloaded < status.total) {
				acc += (status.downloaded / status.total) * 100;
			}
			return acc;
		},
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
								if (status.total !== null && status.downloaded < status.total) {
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
					return (
						<Menu.Item
							key={id}
							disabled={status.total === null}
							leftSection={
								<DownloadProgressRing
									percentage={
										status.total !== null
											? (status.downloaded / status.total) * 100
											: 0
									}
								/>
							}
						>
							{status.url.split("/").slice(-1)[0]}:{" "}
							{status.total
								? `${((status.downloaded / status.total) * 100).toFixed(2)}%`
								: "Unknown size"}
						</Menu.Item>
					);
				})}
			</Menu.Dropdown>
		</Menu>
	);
}
