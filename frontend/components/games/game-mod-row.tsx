import {
	DefaultMantineColor,
	Table,
	ThemeIcon,
	Box,
	ButtonGroup,
	Group,
	Stack,
	Tooltip,
} from "@mantine/core";
import { DbGame, RemoteConfigs, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconCheck,
	IconCirclePlus,
	IconDotsVertical,
	IconDownload,
	IconFolderOpen,
	IconMinus,
	IconPlayerPlay,
	IconRefreshAlert,
	IconSettings,
	IconSettingsFilled,
	IconTrash,
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

	const handleInstallClick = async () => {
		if (mod.remote?.runForGame && !mod.local && !mod.remote) {
			return commands.openModFolder(mod.merged.id);
		}

		if (isLocalModOutdated) {
			// TODO figure out if this error would be handled.
			await commands.downloadMod(mod.merged.id);
		} else if (isInstalled && !isInstalledModOutdated) {
			return commands.uninstallMod(game.providerId, game.gameId, mod.merged.id);
		}

		if (mod.remote?.install) {
			await commands.installMod(game.providerId, game.gameId, mod.merged.id);
		}

		if (mod.remote?.runForGame) {
			await commands.runMod(game.providerId, game.gameId, mod.merged.id);
		}

		if (availableRemoteConfig) {
			await commands.downloadRemoteConfig(
				game.providerId,
				game.gameId,
				mod.merged.id,
				availableRemoteConfig.file,
				false,
			);
		}
	};
	const { actionText, actionIcon } = (() => {
		if (isLocalModOutdated || isInstalledModOutdated) {
			return { actionText: t("updateMod"), actionIcon: <IconRefreshAlert /> };
		}

		if (isInstalled) {
			return { actionText: t("uninstallMod"), actionIcon: <IconTrash /> };
		}

		if (mod.remote?.install) {
			return { actionText: t("installMod"), actionIcon: <IconCirclePlus /> };
		}

		if (!mod.remote && !mod.local) {
			return { actionText: t("openModFolder"), actionIcon: <IconFolderOpen /> };
		}

		return { actionText: t("runMod"), actionIcon: <IconPlayerPlay /> };
	})();

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

	const buttonColor = ((): DefaultMantineColor => {
		if (isLocalModOutdated || isInstalledModOutdated) return "orange";
		if (isInstalled) return "red";
		return "violet";
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
							<CommandButton
								color={buttonColor}
								size="xs"
								leftSection={actionIcon}
								variant={isInstalled ? "light" : "default"}
								confirmationText={
									isInstalled
										? undefined
										: // TODO: translate
											"Attention: be careful when installing mods on multiplayer games! Anticheat can detect some mods and get you banned, even if the mods seem harmless."
								}
								confirmationSkipId={
									isInstalled ? undefined : "install-mod-confirm"
								}
								onClick={handleInstallClick}
							>
								<Box style={{ textOverflow: "ellipsis", overflow: "hidden" }}>
									{actionText}
								</Box>
							</CommandButton>
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
