import {
	DefaultMantineColor,
	Table,
	ThemeIcon,
	Box,
	Button,
	ButtonGroup,
	Group,
	Stack,
} from "@mantine/core";
import { Game, ModLoaderData, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconCheck,
	IconCirclePlus,
	IconDotsVertical,
	IconFolderOpen,
	IconMinus,
	IconPlayerPlay,
	IconRefreshAlert,
	IconSettings,
	IconTrash,
} from "@tabler/icons-react";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { getIsOutdated } from "@util/is-outdated";
import { OutdatedMarker } from "@components/outdated-marker";
import { useCallback } from "react";
import { ItemName } from "@components/item-name";
import { MutedText } from "@components/muted-text";
import { ModVersionBadge } from "@components/mods/mod-version-badge";
import { getModTitle } from "@util/game-mod";
import { CommandDropdown } from "@components/command-dropdown";
import { DeprecatedBadge } from "@components/mods/deprecated-badge";

type Props = {
	readonly game: Game;
	readonly mod: UnifiedMod;
	readonly modLoader: ModLoaderData;
};

export function GameModRow({ game, mod, modLoader }: Props) {
	const installedVersion =
		game.installedGame?.installedModVersions[mod.common.id];

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

	const handleInstallClick = useCallback(async () => {
		if (modLoader.kind === "Runnable" && !mod.local && !mod.remote) {
			return commands.openModFolder(mod.common.id);
		}

		if (isLocalModOutdated) {
			const downloadResult = await commands.downloadMod(mod.common.id);
			if (downloadResult.status === "error") {
				return downloadResult;
			}
		} else if (isInstalled && !isInstalledModOutdated) {
			return commands.uninstallMod(game.id, mod.common.id);
		}

		return commands.installMod(game.id, mod.common.id);
	}, [
		modLoader.kind,
		mod.local,
		mod.remote,
		mod.common.id,
		game,
		isLocalModOutdated,
		isInstalled,
		isInstalledModOutdated,
	]);

	const handleConfigureClick = useCallback(() => {
		commands.configureMod(game.id, mod.common.id);
	}, [game, mod.common.id]);

	const handleOpenModFolderClick = useCallback(() => {
		commands.openInstalledModFolder(game.id, mod.common.id);
	}, [game, mod.common.id]);

	const { actionText, actionIcon } = (() => {
		if (isLocalModOutdated || isInstalledModOutdated) {
			return { actionText: "Update", actionIcon: <IconRefreshAlert /> };
		}

		if (isInstalled) {
			return { actionText: "Uninstall", actionIcon: <IconTrash /> };
		}

		if (modLoader.kind === "Installable") {
			return { actionText: "Install", actionIcon: <IconCirclePlus /> };
		}

		if (!mod.remote && !mod.local) {
			return { actionText: "Open mod folder", actionIcon: <IconFolderOpen /> };
		}

		return { actionText: "Run", actionIcon: <IconPlayerPlay /> };
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
					{game.installedGame && (
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
								<Button
									disabled={!isInstalled && !isReadyRunnable}
									onClick={handleConfigureClick}
									leftSection={<IconSettings />}
								>
									Mod Settings
								</Button>
								<Button
									disabled={!isInstalled && !isReadyRunnable}
									onClick={handleOpenModFolderClick}
									leftSection={<IconFolderOpen />}
								>
									Open Mod Folder
								</Button>
							</CommandDropdown>
						</ButtonGroup>
					)}
				</Group>
			</Table.Td>
		</Table.Tr>
	);
}
