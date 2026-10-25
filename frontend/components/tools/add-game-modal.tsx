import { Channel } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
	commands,
	type DirectoryScanResult,
	type ScanProgress,
} from "@api/bindings";
import {
	ActionIcon,
	Alert,
	Button,
	Code,
	Divider,
	Group,
	Modal,
	ScrollArea,
	Stack,
	Text,
} from "@mantine/core";
import {
	IconAppWindowFilled,
	IconCircleCheck,
	IconDots,
	IconFolderFilled,
	IconInfoCircle,
	IconSearch,
	IconTrash,
} from "@tabler/icons-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { showAppNotification } from "@components/app-notifications";
import { useAsyncCommand } from "@hooks/use-async-command";
import { useLocalization } from "@hooks/use-localization";

type ScanPhase = "idle" | "scanning" | "confirming";

type Props = {
	readonly isOpen: boolean;
	readonly onClose: () => void;
};

export function AddGameModal(props: Props) {
	const { t } = useLocalization("manualGames");
	const [directories, setDirectories] = useState<string[]>([]);

	const [scanPhase, setScanPhase] = useState<ScanPhase>("idle");
	const [scanProgress, setScanProgress] = useState<ScanProgress | null>(null);
	const [scanResult, setScanResult] = useState<DirectoryScanResult | null>(
		null,
	);
	const [selectedPath, setSelectedPath] = useState<string | null>(null);
	const cancelledRef = useRef(false);
	const mountedRef = useRef(true);

	const [executeAddGame] = useAsyncCommand(commands.addGame);
	const [executeAddGameDirectory] = useAsyncCommand(commands.addGameDirectory);

	const loadDirectories = () =>
		commands.getManualGameDirectories().then(setDirectories);

	const resetScanState = useCallback(() => {
		setScanPhase("idle");
		setScanProgress(null);
		setScanResult(null);
		setSelectedPath(null);
		cancelledRef.current = false;
	}, []);

	useEffect(() => {
		mountedRef.current = true;
		return () => {
			mountedRef.current = false;
		};
	}, []);

	useEffect(() => {
		if (props.isOpen) {
			resetScanState();
			loadDirectories();
		}
	}, [props.isOpen, resetScanState]);

	const handleFileClick = async () => {
		const path = await openDialog({
			multiple: false,
			title: "Select the game executable",
			filters: [
				{
					extensions: ["exe"],
					name: "Windows executable",
				},
				{
					extensions: ["*"],
					name: "Other executable",
				},
			],
		});
		if (!path) return;

		await executeAddGame(path).then(() => props.onClose());
	};

	const handleDirectoryClick = async () => {
		const path = await openDialog({
			multiple: false,
			title: "Select a game directory",
			directory: true,
		});
		if (!path) return;

		setSelectedPath(path);
		cancelledRef.current = false;

		const channel = new Channel<ScanProgress>((progress) => {
			setScanProgress(progress);
		});

		setScanPhase("scanning");

		try {
			const result = await commands.scanGameDirectory(path, channel);
			if (cancelledRef.current || !mountedRef.current) return;
			setScanResult(result);
			setScanPhase("confirming");
		} catch (error) {
			showAppNotification(`${error}`, "error");
			if (cancelledRef.current || !mountedRef.current) return;
			setScanPhase("idle");
		}
	};

	const handleCancel = () => {
		cancelledRef.current = true;
		resetScanState();
	};

	const handleConfirm = async () => {
		if (!selectedPath) return;

		try {
			await executeAddGameDirectory(selectedPath);
			if (!mountedRef.current) return;
			loadDirectories();
			resetScanState();
		} catch {
			if (!mountedRef.current) return;
			resetScanState();
		}
	};

	const handleRemoveDirectory = async (path: string) => {
		await commands.removeGameDirectory(path);
		await commands.refreshGames("Manual");
		await loadDirectories();
	};

	return (
		<Modal
			opened={props.isOpen}
			centered
			size="lg"
			onClose={() => {
				handleCancel();
				props.onClose();
			}}
			title={t("title")}
		>
			<Stack>
				<Alert
					p="xs"
					icon={<IconInfoCircle />}
				>
					{t("manualSteamSupportNote")}
				</Alert>

				{scanPhase === "idle" && (
					<>
						<Button
							size="lg"
							leftSection={<IconAppWindowFilled />}
							onClick={handleFileClick}
							rightSection={<IconDots />}
						>
							{t("selectGameExecutable")}
						</Button>
						<Button
							size="lg"
							leftSection={<IconFolderFilled />}
							onClick={handleDirectoryClick}
							rightSection={<IconDots />}
						>
							{t("selectGamesDirectory")}
						</Button>
						<Text>{t("fileDropNote")}</Text>
					</>
				)}

				{scanPhase === "scanning" && (
					<Stack
						align="center"
						gap="md"
					>
						<IconSearch size={48} />
						<Text>{t("scanning", { path: selectedPath ?? "" })}</Text>
						{scanProgress && (
							<Text>
								{t("scanProgress", {
									directories: String(scanProgress.scannedDirs),
									executables: String(scanProgress.executablesFound),
								})}
							</Text>
						)}
						<Button
							color="red"
							variant="light"
							onClick={handleCancel}
						>
							{t("cancel")}
						</Button>
						{scanProgress && (
							<ScrollArea
								w="100%"
								scrollbars="x"
							>
								<Code>
									<pre>{scanProgress.currentPath}</pre>
								</Code>
							</ScrollArea>
						)}
					</Stack>
				)}

				{scanPhase === "confirming" && scanResult && (
					<Stack
						align="center"
						gap="md"
					>
						<IconCircleCheck
							size={48}
							color="green"
						/>
						<Text>
							{t("scanComplete", {
								gamesCount: String(scanResult.games.length),
								duration: scanResult.durationSecs?.toFixed(1) ?? "?",
							})}
						</Text>
						<Group>
							<Button onClick={handleConfirm}>{t("confirmAddFolder")}</Button>
							<Button
								variant="outline"
								onClick={handleCancel}
							>
								{t("cancel")}
							</Button>
						</Group>
					</Stack>
				)}

				{directories.length > 0 && scanPhase === "idle" && (
					<Stack>
						<Divider />
						<Text>{t("savedDirectories")}</Text>
						{directories.map((dir) => (
							<Group key={dir}>
								<Code style={{ flex: 1 }}>{dir}</Code>
								<ActionIcon
									color="red"
									variant="subtle"
									onClick={() => handleRemoveDirectory(dir)}
								>
									<IconTrash />
								</ActionIcon>
							</Group>
						))}
					</Stack>
				)}
			</Stack>
		</Modal>
	);
}
