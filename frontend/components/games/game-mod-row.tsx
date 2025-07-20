import {
	DefaultMantineColor,
	Table,
	ThemeIcon,
	Box,
	Button,
	ButtonGroup,
	Group,
	Stack,
	Tooltip,
} from "@mantine/core";
import { DbGame, ModLoaderData, RemoteConfigs, commands } from "@api/bindings";
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
	readonly modLoader: ModLoaderData;
	readonly remoteConfigs?: RemoteConfigs | null;
	readonly installedVersion?: string;
};

export function GameModRow({
	game,
	mod,
	modLoader,
	installedVersion,
	remoteConfigs,
}: Props) {
	const t = useLocalization("gameModRow");

	const remoteConfig = remoteConfigs?.configs.find(
		(config) => config.mod === mod.common.id && config.loader === modLoader.id,
	);

	const isInstalledModOutdated = getIsOutdated(
		installedVersion,
		mod.remote?.latestVersion?.id,
	);

	const isLocalModOutdated = getIsOutdated(
		mod.local?.manifest?.version,
		mod.remote?.latestVersion?.id,
	);

	const isInstalled = Boolean(installedVersion);
	const isReadyRunnable = mod.local && modLoader.kind == "Runnable";

	const handleInstallClick = async () => {
		if (modLoader.kind === "Runnable" && !mod.local && !mod.remote) {
			return commands.openModFolder(mod.common.id);
		}

		if (isLocalModOutdated) {
			// TODO figure out if this error would be handled.
			await commands.downloadMod(mod.common.id);
		} else if (isInstalled && !isInstalledModOutdated) {
			return commands.uninstallMod(game.providerId, game.gameId, mod.common.id);
		}

		await commands.installMod(game.providerId, game.gameId, mod.common.id);

		if (remoteConfig) {
			await commands.downloadRemoteConfig(
				game.providerId,
				game.gameId,
				mod.common.id,
				remoteConfig.file,
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

		if (modLoader.kind === "Installable") {
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

	return (
		<Table.Tr key={mod.common.id}>
			<Table.Td ta="left">
				<ItemName label={`by ${mod.remote?.author}`}>
					<ThemeIcon
						color={statusColor}
						size="sm"
					>
						{statusIcon}
					</ThemeIcon>
					{getModTitle(mod)}
					{remoteConfig && (
						<Tooltip label={t("remoteConfigAvailable")}>
							<IconSettingsFilled fontSize="15" />
						</Tooltip>
					)}
					<ModVersionBadge
						localVersion={installedVersion}
						remoteVersion={mod.remote?.latestVersion?.id}
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
					{game.exePath && (
						<ButtonGroup>
							<CommandButton
								color={buttonColor}
								size="xs"
								leftSection={actionIcon}
								variant={isInstalled ? "light" : "default"}
								confirmationText={
									isInstalled
										? undefined
										: "Attention: be careful when installing mods on multiplayer games! Anticheat can detect some mods and get you banned, even if the mods seem harmless."
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
								{remoteConfig && (
									<Button
										disabled={!isInstalled && !isReadyRunnable}
										onClick={() =>
											commands.configureMod(
												game.providerId,
												game.gameId,
												mod.common.id,
											)
										}
										leftSection={<IconSettings />}
									>
										{t("modSettings")}
									</Button>
								)}
								<Button
									disabled={!isInstalled && !isReadyRunnable}
									onClick={() =>
										commands.openInstalledModFolder(
											game.providerId,
											game.gameId,
											mod.common.id,
										)
									}
									leftSection={<IconFolderOpen />}
								>
									{t("openModFolder")}
								</Button>
								{remoteConfig && (
									<CommandButton
										disabled={!isInstalled}
										leftSection={<IconDownload />}
										onClick={() =>
											commands.downloadRemoteConfig(
												game.providerId,
												game.gameId,
												mod.common.id,
												remoteConfig.file,
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
