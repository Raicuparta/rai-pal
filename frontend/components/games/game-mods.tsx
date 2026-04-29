import {
	Alert,
	ButtonGroup,
	Divider,
	Group,
	Table,
	ThemeIcon,
} from "@mantine/core";
import {
	EngineVersionRange,
	DbGame,
	ModLoaderStatus,
	commands,
} from "@api/bindings";
import { useCallback, useMemo } from "react";
import { CommandButton } from "@components/command-button";
import {
	IconCheck,
	IconCirclePlus,
	IconDotsVertical,
	IconFolderOpen,
	IconMinus,
	IconRefresh,
	IconRefreshAlert,
	IconTrash,
} from "@tabler/icons-react";
import { CommandDropdown } from "@components/command-dropdown";
import { UnifiedMod, useUnifiedMods } from "@hooks/use-unified-mods";
import { GameModRow } from "./game-mod-row";
import { TableContainer } from "@components/table/table-container";
import { useLocalization } from "@hooks/use-localization";
import { useCommandData } from "@hooks/use-command-data";
import { useAppEvent } from "@hooks/use-app-event";
import { MutedText } from "@components/muted-text";
import { getIsOutdated } from "@util/is-outdated";
import { OutdatedMarker } from "@components/outdated-marker";
import { ItemName } from "@components/item-name";
import { ModVersionBadge } from "@components/mods/mod-version-badge";
import { useAsyncCommand } from "@hooks/use-async-command";

type Props = {
	readonly game: DbGame;
};

function isVersionWithinRange(
	{
		engineVersionMajor: major,
		engineVersionMinor: minor,
		engineVersionPatch: patch,
	}: DbGame,
	range: EngineVersionRange | null,
) {
	if (!major || !range) return true;

	if (!major) return false;

	const { minimum, maximum } = range;

	if (minimum && minimum.major > major) return false;
	if (maximum && maximum.major < major) return false;
	if (
		minimum &&
		minimum.major === major &&
		minimum.minor != null &&
		minor != null &&
		minimum.minor > minor
	)
		return false;
	if (
		maximum &&
		maximum.major === major &&
		maximum.minor != null &&
		minor != null &&
		maximum.minor < minor
	)
		return false;
	if (
		minimum &&
		minimum.major === major &&
		minimum.minor === minor &&
		minimum.patch != null &&
		patch != null &&
		minimum.patch > patch
	)
		return false;
	if (
		maximum &&
		maximum.major === major &&
		maximum.minor === minor &&
		maximum.patch != null &&
		patch != null &&
		maximum.patch < patch
	)
		return false;

	return true;
}

const defaultInstalledModVersions: Record<string, string> = {};

type ModLoaderRowProps = {
	readonly modLoaderId: string;
	readonly game: DbGame;
	readonly status: ModLoaderStatus;
};

function ModLoaderRow({ game, modLoaderId, status }: ModLoaderRowProps) {
	const tGameModRow = useLocalization("gameModRow");
	const [runModLoaderAction, isRunningModLoaderAction] = useAsyncCommand(
		(forceReinstall: boolean) =>
			commands.installModLoader(
				game.providerId,
				game.gameId,
				modLoaderId,
				forceReinstall,
			),
	);

	const [runUninstallModLoader, isUninstallingModLoader] = useAsyncCommand(() =>
		commands.uninstallModLoader(game.providerId, game.gameId, modLoaderId),
	);

	const isOutdated = getIsOutdated(
		status.installedVersion,
		status.latestVersion,
	);

	const isInstalled = Boolean(status.installedVersion);

	const { statusIcon, statusColor } = (() => {
		if (isOutdated) {
			return {
				statusIcon: <OutdatedMarker />,
				statusColor: "orange",
			};
		}

		if (isInstalled) {
			return {
				statusIcon: <IconCheck />,
				statusColor: "green",
			};
		}

		return {
			statusIcon: <IconMinus />,
			statusColor: "gray",
		};
	})();

	const handleOpenModLoaderFolder = async () => {
		await commands.openGameModLoaderFolder(
			game.providerId,
			game.gameId,
			modLoaderId,
		);
	};

	const { mainButtonAction, mainButtonIcon, mainButtonColor } = (() => {
		if (!isInstalled) {
			return {
				mainButtonAction: () => runModLoaderAction(false),
				mainButtonIcon: <IconCirclePlus />,
				mainButtonColor: "violet",
			};
		}

		if (isOutdated) {
			return {
				mainButtonAction: () => runModLoaderAction(true),
				mainButtonIcon: <IconRefreshAlert />,
				mainButtonColor: "orange",
			};
		}

		return {
			mainButtonAction: () => runUninstallModLoader(),
			mainButtonIcon: <IconTrash />,
			mainButtonColor: "red",
		};
	})();

	const mainButtonLabel = !isInstalled
		? tGameModRow("installMod")
		: isOutdated
			? tGameModRow("updateMod")
			: "Uninstall";

	return (
		<Table.Tr key={`${modLoaderId}-row`}>
			<Table.Td ta="left">
				<ItemName label="mod loader">
					<ThemeIcon
						color={statusColor}
						size="sm"
					>
						{statusIcon}
					</ThemeIcon>
					{modLoaderId}
					<ModVersionBadge
						localVersion={status.installedVersion ?? undefined}
						remoteVersion={status.latestVersion ?? undefined}
					/>
				</ItemName>
				<MutedText>Required by some mods.</MutedText>
			</Table.Td>
			<Table.Td maw={200}>
				<Group justify="right">
					<ButtonGroup>
						<CommandButton
							size="xs"
							leftSection={mainButtonIcon}
							color={mainButtonColor}
							variant={mainButtonColor === "red" ? "light" : "default"}
							loading={isRunningModLoaderAction || isUninstallingModLoader}
							onClick={mainButtonAction}
						>
							{mainButtonLabel}
						</CommandButton>
						<CommandDropdown icon={<IconDotsVertical />}>
							<CommandButton
								size="xs"
								leftSection={<IconRefresh />}
								disabled={!isInstalled}
								loading={isRunningModLoaderAction}
								onClick={() => runModLoaderAction(true)}
							>
								Reinstall
							</CommandButton>
							<CommandButton
								size="xs"
								leftSection={<IconFolderOpen />}
								onClick={handleOpenModLoaderFolder}
							>
								Open mod loader folder
							</CommandButton>
						</CommandDropdown>
					</ButtonGroup>
				</Group>
			</Table.Td>
		</Table.Tr>
	);
}

export function GameMods({ game }: Props) {
	const t = useLocalization("gameModal");
	const mods = useUnifiedMods();
	const getInstalledModVersions = useCallback(
		() => commands.getInstalledModVersions(game.providerId, game.gameId),
		[game],
	);
	const [installedModVersions, updateInstalledModVersions] = useCommandData(
		getInstalledModVersions,
		defaultInstalledModVersions,
		!game?.exePath,
	);
	const getRemoteConfigs = useCallback(
		() => commands.getRemoteConfigs(game.providerId, game.gameId),
		[game],
	);
	const [remoteConfigs] = useCommandData(
		getRemoteConfigs,
		null,
		!game?.exePath,
	);
	const getModLoaderStatuses = useCallback(
		() => commands.getModLoaderStatuses(game.providerId, game.gameId),
		[game],
	);
	const [modLoaderStatuses, updateModLoaderStatuses] = useCommandData(
		getModLoaderStatuses,
		{},
		!game?.exePath,
	);

	useAppEvent(
		"refreshGame",
		`installed-mods-${game.providerId}:${game.gameId}`,
		([refreshedProviderId, refreshedGameId]) => {
			if (
				refreshedProviderId !== game.providerId ||
				refreshedGameId !== game.gameId
			)
				return;
			updateInstalledModVersions();
			updateModLoaderStatuses();
		},
	);

	const { compatibleMods, incompatibleMods } = useMemo(() => {
		const compatibleMods: UnifiedMod[] = [];
		const incompatibleMods: UnifiedMod[] = [];

		for (const mod of Object.values(mods)) {
			const isCompatibleEngine =
				!mod.common.engine || mod.common.engine === game.engineBrand;
			const isCompatibleUnityBackend =
				!mod.common.unityBackend ||
				!game.unityBackend ||
				mod.common.unityBackend === game.unityBackend;

			if (!game || !isCompatibleEngine || !isCompatibleUnityBackend) {
				continue;
			}

			// Deprecated mods only show if they had been previously installed.
			if (mod.remote?.deprecated && !installedModVersions[mod.common.id]) {
				continue;
			}

			if (isVersionWithinRange(game, mod.common.engineVersionRange)) {
				compatibleMods.push(mod);
			} else {
				incompatibleMods.push(mod);
			}
		}

		return {
			compatibleMods,
			incompatibleMods,
		};
	}, [game, installedModVersions, mods]);

	const modLoaderRows = Object.entries(modLoaderStatuses);

	if (
		compatibleMods.length + incompatibleMods.length === 0 &&
		modLoaderRows.length === 0
	) {
		return null;
	}

	return (
		<>
			{(compatibleMods.length > 0 || modLoaderRows.length > 0) && (
				<>
					<Divider label={t("gameModsLabel")} />
					{!game.exePath && (
						<Alert color="orange">{t("gameNotInstalledWarning")}</Alert>
					)}
					<TableContainer bg="dark">
						<Table>
							<Table.Tbody>
								{modLoaderRows.map(([modLoaderId, status]) => (
									<ModLoaderRow
										key={modLoaderId}
										modLoaderId={modLoaderId}
										game={game}
										status={status}
									/>
								))}
								{compatibleMods.map((mod) => (
									<GameModRow
										key={mod.common.id}
										game={game}
										mod={mod}
										remoteConfigs={remoteConfigs}
										installedVersion={installedModVersions[mod.common.id]}
									/>
								))}
							</Table.Tbody>
						</Table>
					</TableContainer>
				</>
			)}
			{incompatibleMods.length > 0 && (
				<>
					<Divider label={t("incompatibleGameModsLabel")} />
					<MutedText>{t("incompatibleGameModsDescription")}</MutedText>
					<TableContainer bg="dark">
						<Table>
							<Table.Tbody>
								{incompatibleMods.map((mod) => (
									<GameModRow
										key={mod.common.id}
										game={game}
										mod={mod}
										remoteConfigs={remoteConfigs}
										installedVersion={installedModVersions[mod.common.id]}
										incompatible
									/>
								))}
							</Table.Tbody>
						</Table>
					</TableContainer>
				</>
			)}
			{game.exePath && (
				<CommandButton
					confirmationText={t("uninstallAllModsConfirmation")}
					onClick={() =>
						commands.uninstallAllMods(game.providerId, game.gameId)
					}
					color="red"
					variant="light"
					leftSection={<IconTrash />}
				>
					{t("uninstallAllModsButton")}
				</CommandButton>
			)}
		</>
	);
}
