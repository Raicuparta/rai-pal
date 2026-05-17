import {
	Table,
	ThemeIcon,
	ButtonGroup,
	Group,
	Stack,
	Tooltip,
} from "@mantine/core";
import { DbGame, RemoteConfigs, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconCheck,
	IconDotsVertical,
	IconDownload,
	IconFolderOpen,
	IconMinus,
	IconSettings,
	IconSettingsFilled,
} from "@tabler/icons-react";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { getIsOutdated } from "@util/is-outdated";
import { OutdatedMarker } from "@components/outdated-marker";
import { ItemName } from "@components/item-name";
import { MutedText } from "@components/muted-text";
import { ModVersionBadge } from "@components/mods/mod-version-badge";
import { getModTitle } from "@util/game-mod";
import { CommandDropdown } from "@components/command-dropdown";
import { DeprecatedBadge } from "@components/mods/deprecated-badge";
import { useLocalization } from "@hooks/use-localization";
import { GameModInstallButton } from "./game-mod-install-button";
import { GameModUpdateButton } from "./game-mod-update-button";
import { GameModRunButton } from "./game-mod-run-button";
import { GameModUninstallButton } from "./game-mod-uninstall-button";

type Props = {
	readonly game: DbGame;
	readonly mod: UnifiedMod;
	readonly remoteConfigs?: RemoteConfigs | null;
	readonly installedVersion?: string;
	readonly incompatible?: boolean;
};

export function GameModRow({
	game,
	mod,
	installedVersion,
	remoteConfigs,
	incompatible = false,
}: Props) {
	const t = useLocalization("gameModRow");

	const availableRemoteConfig = remoteConfigs?.configs.find(
		(config) => config.modId === mod.merged.id,
	);
	const localConfig = mod.local?.config;

	const isInstalledModOutdated = getIsOutdated(
		installedVersion,
		mod.remote?.latestVersion.id,
	);

	const isLocalModOutdated = getIsOutdated(
		mod.local?.latestVersion.id,
		mod.remote?.latestVersion.id,
	);

	const isInstalled = Boolean(installedVersion);
	const isReadyRunnable = mod.local && mod.remote?.runForGame;

	const { statusIcon, statusColor } = (() => {
		if (isLocalModOutdated || isInstalledModOutdated)
			return {
				statusIcon: <OutdatedMarker />,
				statusColor: "orange",
			};
		if (isInstalled || isReadyRunnable)
			return {
				statusIcon: <IconCheck />,
				statusColor: "green",
			};
		return {
			statusIcon: <IconMinus />,
			statusColor: "gray",
		};
	})();

	const isModUsable = !incompatible && game.exePath;

	return (
		<Table.Tr key={mod.merged.id}>
			<Table.Td ta="left">
				<ItemName label={`by ${mod.remote?.author}`}>
					{isModUsable && (
						<ThemeIcon
							color={statusColor}
							size="sm"
						>
							{statusIcon}
						</ThemeIcon>
					)}
					{getModTitle(mod)}
					{availableRemoteConfig && (
						<Tooltip label={t("remoteConfigAvailable")}>
							<IconSettingsFilled fontSize="15" />
						</Tooltip>
					)}
					<ModVersionBadge
						localVersion={installedVersion}
						remoteVersion={mod.remote?.latestVersion.id}
					/>
				</ItemName>
				<Stack gap={0}>
					{mod.remote?.deprecated && <DeprecatedBadge mt={5} />}
					{mod.remote?.description && (
						<MutedText>{mod.remote.description}</MutedText>
					)}
				</Stack>
			</Table.Td>
			<Table.Td maw={200}>
				<Group justify="right">
					{isModUsable && (
						<ButtonGroup>
							{!isInstalled &&
								!isInstalledModOutdated &&
								mod.merged.install && (
									<GameModInstallButton
										game={game}
										mod={mod}
									/>
								)}
							{isInstalled && mod.merged.install && (
								<GameModUninstallButton
									game={game}
									mod={mod}
								/>
							)}
							{(isLocalModOutdated || isInstalledModOutdated) && (
								<GameModUpdateButton
									game={game}
									mod={mod}
								/>
							)}
							{mod.merged.runForGame && (
								<GameModRunButton
									game={game}
									mod={mod}
								/>
							)}
							<CommandDropdown icon={<IconDotsVertical />}>
								{(localConfig || availableRemoteConfig) && (
									<ButtonGroup>
										<CommandButton
											flex={1}
											disabled={!isInstalled && !isReadyRunnable}
											onClick={() =>
												commands.configureMod(
													game.providerId,
													game.gameId,
													mod.merged.id,
													false,
												)
											}
											leftSection={<IconSettings />}
										>
											{t("editModConfig")}
										</CommandButton>
										<Tooltip
											label={t("openModConfigFolderTooltip")}
											position="top-end"
										>
											<CommandButton
												disabled={!isInstalled && !isReadyRunnable}
												onClick={() =>
													commands.configureMod(
														game.providerId,
														game.gameId,
														mod.merged.id,
														true,
													)
												}
											>
												<IconFolderOpen />
											</CommandButton>
										</Tooltip>
									</ButtonGroup>
								)}
								<CommandButton
									disabled={!isInstalled && !isReadyRunnable}
									onClick={() =>
										commands.openInstalledModFolder(
											game.providerId,
											game.gameId,
											mod.merged.id,
										)
									}
									leftSection={<IconFolderOpen />}
								>
									{t("openModFolder")}
								</CommandButton>
								{availableRemoteConfig && (
									<CommandButton
										disabled={!isInstalled && !isReadyRunnable}
										leftSection={<IconDownload />}
										onClick={() =>
											commands.downloadRemoteConfig(
												game.providerId,
												game.gameId,
												mod.merged.id,
												availableRemoteConfig.file,
												true,
											)
										}
									>
										{t("downloadRemoteConfig")}
									</CommandButton>
								)}
							</CommandDropdown>
						</ButtonGroup>
					)}
				</Group>
			</Table.Td>
		</Table.Tr>
	);
}
