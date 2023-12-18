import { Text, DefaultMantineColor } from "@mantine/core";
import {
	ModLoaderData,
	downloadMod,
	installMod,
	uninstallMod,
} from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconCubeOff, IconCubePlus, IconPlayerPlay } from "@tabler/icons-react";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { getIsOutdated } from "../../util/is-outdated";
import { OutdatedMarker } from "@components/OutdatedMarker";
import { ProcessedInstalledGame } from "@hooks/use-processed-installed-games";
import { useCallback } from "react";

type Props = {
	readonly game: ProcessedInstalledGame;
	readonly mod: UnifiedMod;
	readonly modLoader: ModLoaderData;
};

export function GameModButton(props: Props) {
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
		return "Run";
	}

	function getIcon() {
		if (isInstalledModOutdated) return <OutdatedMarker />;
		if (isInstalled) return <IconCubeOff />;
		if (props.modLoader.kind === "Runnable") return <IconPlayerPlay />;
		return <IconCubePlus />;
	}

	function getColor(): DefaultMantineColor {
		if (isInstalledModOutdated) return "orange";
		if (isInstalled) return "red";
		return "violet";
	}

	return (
		<CommandButton
			leftSection={getIcon()}
			fullWidth
			variant="light"
			color={getColor()}
			confirmationText={
				isInstalled
					? undefined
					: "Attention: be careful when installing mods on multiplayer games! Anticheat can detect some mods and get you banned, even if the mods seem harmless."
			}
			confirmationSkipId={isInstalled ? undefined : "install-mod-confirm"}
			onClick={handleClick}
		>
			<Text>
				{getActionText()} {versionText}
			</Text>
		</CommandButton>
	);
}
