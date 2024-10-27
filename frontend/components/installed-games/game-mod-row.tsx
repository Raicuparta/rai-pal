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
import { ModLoaderData, commands } from "@api/bindings";
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
import { ProcessedInstalledGame } from "@hooks/use-processed-installed-games";
import { useCallback } from "react";
import { ItemName } from "@components/item-name";
import { MutedText } from "@components/muted-text";
import { ModVersionBadge } from "@components/mods/mod-version-badge";
import { getModTitle } from "@util/game-mod";
import { CommandDropdown } from "@components/command-dropdown";
import { DeprecatedBadge } from "@components/mods/deprecated-badge";

const {
	configureMod,
	downloadMod,
	installMod,
	openInstalledModFolder,
	openModFolder,
	uninstallMod,
} = commands;

type Props = {
	readonly game: ProcessedInstalledGame;
	readonly mod: UnifiedMod;
	readonly modLoader: ModLoaderData;
};

export function GameModRow(props: Props) {
	const installedVersion = props.game.installedModVersions[props.mod.common.id];
	const isInstalledModOutdated = getIsOutdated(
		installedVersion,
		props.mod.remote?.latestVersion?.id,
	);
	const isLocalModOutdated = getIsOutdated(
		props.mod.local?.manifest?.version,
		props.mod.remote?.latestVersion?.id,
	);
	const isInstalled = Boolean(installedVersion);
	const isReadyRunnable = props.mod.local && props.modLoader.kind == "Runnable";

	const handleInstallClick = useCallback(async () => {
		if (
			props.modLoader.kind === "Runnable" &&
			!props.mod.local &&
			!props.mod.remote
		) {
			return openModFolder(props.mod.common.id);
		}

		if (isLocalModOutdated) {
			const downloadResult = await downloadMod(props.mod.common.id);
			if (downloadResult.status === "error") {
				return downloadResult;
			}
		} else if (isInstalled && !isInstalledModOutdated) {
			return uninstallMod(props.game, props.mod.common.id);
		}

		return installMod(props.game, props.mod.common.id);
	}, [
		props.modLoader.kind,
		props.mod.local,
		props.mod.remote,
		props.mod.common.id,
		props.game,
		isLocalModOutdated,
		isInstalled,
		isInstalledModOutdated,
	]);

	const handleConfigureClick = useCallback(() => {
		configureMod(props.game, props.mod.common.id);
	}, [props.game, props.mod.common.id]);

	const handleOpenModFolderClick = useCallback(() => {
		openInstalledModFolder(props.game, props.mod.common.id);
	}, [props.game, props.mod.common.id]);

	const { actionText, actionIcon } = (() => {
		if (isLocalModOutdated || isInstalledModOutdated) {
			return { actionText: "Update", actionIcon: <IconRefreshAlert /> };
		}

		if (isInstalled) {
			return { actionText: "Uninstall", actionIcon: <IconTrash /> };
		}

		if (props.modLoader.kind === "Installable") {
			return { actionText: "Install", actionIcon: <IconCirclePlus /> };
		}

		if (!props.mod.remote && !props.mod.local) {
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
		<Table.Tr key={props.mod.common.id}>
			<Table.Td ta="left">
				<ItemName label={`by ${props.mod.remote?.author}`}>
					<ThemeIcon
						color={statusColor}
						size="sm"
					>
						{statusIcon}
					</ThemeIcon>
					{getModTitle(props.mod)}
					<ModVersionBadge
						localVersion={installedVersion}
						remoteVersion={props.mod.remote?.latestVersion?.id}
					/>
				</ItemName>
				<Stack gap={0}>
					{props.mod.remote?.deprecated && <DeprecatedBadge mt={5} />}
					{props.mod.remote?.description && (
						<MutedText>{props.mod.remote.description}</MutedText>
					)}
				</Stack>
			</Table.Td>
			<Table.Td maw={200}>
				<Group justify="right">
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
				</Group>
			</Table.Td>
		</Table.Tr>
	);
}
