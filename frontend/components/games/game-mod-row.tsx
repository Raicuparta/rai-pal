import { Table, ThemeIcon, ButtonGroup, Group, Tooltip } from "@mantine/core";
import { DbGame, GameMod, GameModInfo, commands } from "@api/bindings";
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
import { OutdatedMarker } from "@components/outdated-marker";
import { CommandDropdown } from "@components/command-dropdown";
import { ModDetails } from "@components/mods/mod-details";
import { useLocalization } from "@hooks/use-localization";
import { GameModInstallButton } from "./game-mod-install-button";
import { GameModRunButton } from "./game-mod-run-button";
import { GameModUpdateButton } from "./game-mod-update-button";
import { GameModUninstallButton } from "./game-mod-uninstall-button";

type Props = {
	readonly game: DbGame;
	readonly mod: GameMod;
	readonly info?: GameModInfo;
	readonly hasRemoteConfig?: boolean;
	readonly incompatible?: boolean;
};

export function GameModRow({
	game,
	mod,
	info,
	hasRemoteConfig = false,
	incompatible = false,
}: Props) {
	const { t } = useLocalization("gameModRow");

	const isOutdated = info?.isOutdated;

	const isInstalled = Boolean(info?.installedHash);

	const { statusIcon, statusColor } = (() => {
		if (isOutdated)
			return {
				statusIcon: <OutdatedMarker />,
				statusColor: "orange",
			};
		if (isInstalled)
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

	const versionText = [
		info?.installedVersion || mod.download?.id,
		isOutdated ? mod.download?.id : null,
	]
		.filter(Boolean)
		.join(" ➔ ");

	return (
		<Table.Tr key={mod.id}>
			<Table.Td>
				<ModDetails
					mod={mod}
					versionText={versionText}
					leading={
						isModUsable ? (
							<ThemeIcon
								color={statusColor}
								size="sm"
							>
								{statusIcon}
							</ThemeIcon>
						) : undefined
					}
					titleAdornment={
						hasRemoteConfig ? (
							<Tooltip label={t("remoteConfigAvailable")}>
								<IconSettingsFilled fontSize={15} />
							</Tooltip>
						) : undefined
					}
				/>
			</Table.Td>
			<Table.Td maw={200}>
				<Group justify="right">
					{isModUsable && (
						<ButtonGroup>
							{!isInstalled && !isOutdated && mod.install && (
								<GameModInstallButton
									game={game}
									mod={mod}
								/>
							)}
							{isOutdated && (
								<GameModUpdateButton
									game={game}
									mod={mod}
								/>
							)}
							{mod.runForGame && (!mod.install || isInstalled) && (
								<GameModRunButton
									game={game}
									mod={mod}
								/>
							)}
							<CommandDropdown icon={<IconDotsVertical />}>
								{(mod.config || hasRemoteConfig) && (
									<ButtonGroup>
										<CommandButton
											flex={1}
											disabled={!isInstalled}
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
												disabled={!isInstalled}
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
									disabled={!isInstalled}
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
								{hasRemoteConfig && (
									<CommandButton
										disabled={!isInstalled}
										leftSection={<IconDownload />}
										onClick={() =>
											commands.downloadModConfig(
												game.providerId,
												game.gameId,
												mod.id,
											)
										}
									>
										{t("downloadRemoteConfig")}
									</CommandButton>
								)}
								{isInstalled && mod.install && (
									<GameModUninstallButton
										game={game}
										mod={mod}
										modInfo={info}
									/>
								)}
							</CommandDropdown>
						</ButtonGroup>
					)}
				</Group>
			</Table.Td>
		</Table.Tr>
	);
}
