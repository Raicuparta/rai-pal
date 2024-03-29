import { DefaultMantineColor, Table, ThemeIcon, Box } from "@mantine/core";
import {
	ModLoaderData,
	downloadMod,
	installMod,
	openModFolder,
	uninstallMod,
} from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconCheck,
	IconCirclePlus,
	IconFolderOpen,
	IconMinus,
	IconPlayerPlay,
	IconRefreshAlert,
	IconTrash,
} from "@tabler/icons-react";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { getIsOutdated } from "../../util/is-outdated";
import { OutdatedMarker } from "@components/outdated-marker";
import { ProcessedInstalledGame } from "@hooks/use-processed-installed-games";
import { useCallback } from "react";
import { ItemName } from "@components/item-name";
import { MutedText } from "@components/muted-text";
import { ModVersionBadge } from "@components/mods/mod-version-badge";

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

	const handleClick = useCallback(async () => {
		if (
			props.modLoader.kind === "Runnable" &&
			!props.mod.local &&
			!props.mod.remote
		) {
			await openModFolder(props.mod.common.id);
			return;
		}

		if (isLocalModOutdated) {
			await downloadMod(props.mod.common.id);
		} else if (isInstalled && !isInstalledModOutdated) {
			await uninstallMod(props.game.id, props.mod.common.id);
			return;
		}

		await installMod(props.game.id, props.mod.common.id);
	}, [
		props.modLoader.kind,
		props.mod.local,
		props.mod.remote,
		props.mod.common.id,
		props.game.id,
		isLocalModOutdated,
		isInstalled,
		isInstalledModOutdated,
	]);

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
		if (isInstalled || (props.mod.local && props.modLoader.kind == "Runnable"))
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
					{props.mod.remote?.title ?? props.mod.common.id}
					<ModVersionBadge
						localVersion={installedVersion}
						remoteVersion={props.mod.remote?.latestVersion?.id}
					/>
				</ItemName>
				{props.mod.remote?.description && (
					<MutedText>{props.mod.remote.description}</MutedText>
				)}
			</Table.Td>
			<Table.Td maw={200}>
				<CommandButton
					fullWidth
					color={buttonColor}
					size="xs"
					leftSection={actionIcon}
					variant={isInstalled ? "light" : "default"}
					confirmationText={
						isInstalled
							? undefined
							: "Attention: be careful when installing mods on multiplayer games! Anticheat can detect some mods and get you banned, even if the mods seem harmless."
					}
					confirmationSkipId={isInstalled ? undefined : "install-mod-confirm"}
					onClick={handleClick}
				>
					<Box style={{ textOverflow: "ellipsis", overflow: "hidden" }}>
						{actionText}
					</Box>
				</CommandButton>
			</Table.Td>
		</Table.Tr>
	);
}
