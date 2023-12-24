import { DefaultMantineColor, Table, ThemeIcon, Group } from "@mantine/core";
import {
	ModLoaderData,
	downloadMod,
	installMod,
	uninstallMod,
} from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconCheck,
	IconCirclePlus,
	IconMinus,
	IconPlayerPlay,
	IconRefreshAlert,
	IconTrash,
} from "@tabler/icons-react";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { getIsOutdated } from "../../util/is-outdated";
import { OutdatedMarker } from "@components/OutdatedMarker";
import { ProcessedInstalledGame } from "@hooks/use-processed-installed-games";
import { useCallback } from "react";
import { ItemName } from "@components/item-name";
import { MutedText } from "@components/muted-text";

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
		if (isLocalModOutdated) {
			await downloadMod(props.mod.common.id);
		} else if (isInstalled && !isInstalledModOutdated) {
			await uninstallMod(props.game.id, props.mod.common.id);
			return;
		}

		await installMod(props.game.id, props.mod.common.id);
	}, [
		isInstalled,
		isLocalModOutdated,
		isInstalledModOutdated,
		props.game.id,
		props.mod.common.id,
	]);

	const versionText =
		!isInstalled || isInstalledModOutdated
			? props.mod.remote?.latestVersion?.id
			: installedVersion;

	function getActionText() {
		if (isInstalledModOutdated) return "Update to";
		if (isInstalled) return "Uninstall";
		if (props.modLoader.kind === "Installable") return "Install";
		// TODO runnable mod should say "open mod folder" if not installed.
		return "Run";
	}

	function getButtonIcon() {
		if (isInstalledModOutdated) return <IconRefreshAlert />;
		if (isInstalled) return <IconTrash />;
		if (props.modLoader.kind === "Runnable") return <IconPlayerPlay />;
		return <IconCirclePlus />;
	}

	function getStatusIcon() {
		if (isInstalledModOutdated) return <OutdatedMarker />;
		if (isInstalled) return <IconCheck />;
		return <IconMinus />;
	}

	function getButtonColor(): DefaultMantineColor {
		if (isInstalledModOutdated) return "orange";
		if (isInstalled) return "red";
		return "violet";
	}

	function getStatusColor(): DefaultMantineColor {
		if (isInstalledModOutdated) return "orange";
		if (isInstalled) return "green";
		return "gray";
	}

	return (
		<Table.Tr key={props.mod.common.id}>
			<Table.Td ta="left">
				<ItemName label={`by ${props.mod.remote?.author}`}>
					<ThemeIcon
						color={getStatusColor()}
						size="sm"
					>
						{getStatusIcon()}
					</ThemeIcon>
					{props.mod.remote?.title ?? props.mod.common.id}
				</ItemName>
				{props.mod.remote?.description && (
					<MutedText>{props.mod.remote.description}</MutedText>
				)}
			</Table.Td>
			<Table.Td>
				<Group>
					<CommandButton
						fullWidth
						color={getButtonColor()}
						size="xs"
						leftSection={getButtonIcon()}
						variant={isInstalled ? "light" : "default"}
						confirmationText={
							isInstalled
								? undefined
								: "Attention: be careful when installing mods on multiplayer games! Anticheat can detect some mods and get you banned, even if the mods seem harmless."
						}
						confirmationSkipId={isInstalled ? undefined : "install-mod-confirm"}
						onClick={handleClick}
					>
						{getActionText()} {versionText}
					</CommandButton>
				</Group>
			</Table.Td>
		</Table.Tr>
	);
}
