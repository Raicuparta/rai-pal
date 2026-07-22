import { commands, ProgressStatus } from "@api/bindings";
import { Channel } from "@tauri-apps/api/core";
import { Button, DefaultMantineColor, Menu, RingProgress } from "@mantine/core";
import { IconClearAll } from "@tabler/icons-react";
import { useEffect, useState } from "react";
import { useLocalization } from "@hooks/use-localization";

export type ProgressItem = {
	name: string;
	progress: number;
	phase: ProgressStatus["phase"];
	error?: string;
};

const colorMap: Record<ProgressStatus["phase"], DefaultMantineColor> = {
	failed: "red",
	inProgress: "blue",
	finished: "green",
	pending: "gray",
};

export function DownloadStatusMenu() {
	const { t } = useLocalization("downloadStatusMenu");

	const [items, setItems] = useState<Map<string, ProgressItem>>(new Map());

	useEffect(() => {
		commands.listenToDownloadProgress(
			new Channel<ProgressStatus>((status) => {
				setItems((prev) => {
					if (status.phase === "pending") {
						const allFinished = [...prev.values()].every(
							(item) => item.phase === "finished",
						);
						const next = allFinished ? new Map() : new Map(prev);
						next.set(status.id, {
							name: status.name,
							progress: 0,
							phase: "pending",
						});
						return next;
					}

					const next = new Map(prev);
					switch (status.phase) {
						case "inProgress":
							next.set(status.id, {
								name: prev.get(status.id)?.name ?? status.id,
								progress: (status.progress ?? 0) * 100,
								phase: "inProgress",
							});
							break;
						case "finished":
							next.set(status.id, {
								name: prev.get(status.id)?.name ?? status.id,
								progress: 100,
								phase: "finished",
							});
							break;
						case "failed":
							next.set(status.id, {
								name: prev.get(status.id)?.name ?? status.id,
								progress: prev.get(status.id)?.progress ?? 0,
								error: status.error,
								phase: "failed",
							});
							break;
					}
					return next;
				});
			}),
		);
	}, []);

	if (items.size === 0) {
		return null;
	}

	const entries = [...items.entries()];

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
					<RingProgress
						size={40}
						sections={entries.map(([, item]) => ({
							color: colorMap[item.phase],
							value:
								(item.phase === "inProgress" ? item.progress : 100) /
								entries.length,
						}))}
					/>
				</Button>
			</Menu.Target>
			<Menu.Dropdown
				p="xs"
				bg="dark"
			>
				<Menu.Item
					onClick={() => {
						setItems((prev) => {
							const next = new Map(prev);
							for (const [id, item] of next) {
								if (item.phase === "failed" || item.phase === "finished") {
									next.delete(id);
								}
							}
							return next;
						});
					}}
					leftSection={<IconClearAll />}
				>
					{t("clear")}
				</Menu.Item>
				{entries.map(([id, item]) => {
					const progress = item.progress;
					return (
						<Menu.Item
							key={id}
							leftSection={
								<RingProgress
									size={40}
									sections={[
										{
											value: item.phase === "inProgress" ? item.progress : 100,
											color: colorMap[item.phase],
										},
									]}
								/>
							}
						>
							{item.name}: {progress.toFixed(2)}%
						</Menu.Item>
					);
				})}
			</Menu.Dropdown>
		</Menu>
	);
}
