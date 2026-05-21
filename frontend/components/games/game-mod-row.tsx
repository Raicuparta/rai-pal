import {
	Table,
	ThemeIcon,
	ButtonGroup,
	Group,
	Stack,
	Tooltip,
} from "@mantine/core";
import { DbGame, GameMod, RemoteConfigs, commands } from "@api/bindings";
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
	readonly installedMod?: GameMod;
	readonly incompatible?: boolean;
};

export function GameModRow({
	game,
	mod,
	installedMod,
	remoteConfigs,
	incompatible = false,
}: Props) {
	const t = useLocalization("gameModRow");

	const availableRemoteConfig = remoteConfigs?.configs.find(
		(config) => config.modId === mod.id,
	);

	const isInstalledModOutdated = getIsOutdated(installedMod, mod.remote);

	const isLocalModOutdated = getIsOutdated(mod.local, mod.remote);

	const isInstalled = Boolean(installedMod);
	const isReadyRunnable = mod.local && mod.runForGame;

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
		<Table.Tr key={mod.id}>
			<Table.Td ta="left">
				<ItemName label={`by ${mod.author}`}>
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
						local={installedMod}
						remote={mod.remote}
					/>
				</ItemName>
				<Stack gap={0}>
					{mod?.deprecated && <DeprecatedBadge mt={5} />}
					{mod?.description && <MutedText>{mod.description}</MutedText>}
				</Stack>
			</Table.Td>
			<Table.Td maw={200}>
				<Group justify="right">
					{isModUsable && (
						<ButtonGroup>
							{!isInstalled && !isInstalledModOutdated && mod.install && (
								<GameModInstallButton
									game={game}
									mod={mod}
								/>
							)}
							{isInstalled && mod.install && (
								<GameModUninstallButton
									game={game}
									mod={mod}
								/>
							)}
							{(isLocalModOutdated || isInstalledModOutdated) && (
								<GameModUpdateButton
									game={game}
									mod={mod}
									isLocalModOutdated={isLocalModOutdated}
								/>
							)}
							{mod.runForGame && (
								<GameModRunButton
									game={game}
									mod={mod}
								/>
							)}
							<CommandDropdown icon={<IconDotsVertical />}>
								{(mod.config || availableRemoteConfig) && (
									<ButtonGroup>
										<CommandButton
											flex={1}
											disabled={!isInstalled && !isReadyRunnable}
											onClick={() =>
												commands.configureMod(
													game.providerId,
													game.gameId,
													mod.id,
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
														mod.id,
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
											mod.id,
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
												mod.id,
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
