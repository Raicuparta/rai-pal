import {
	Table,
	ThemeIcon,
	ButtonGroup,
	Group,
	Stack,
	Tooltip,
	Text,
} from "@mantine/core";
import {
	DbGame,
	EngineVersionNumbers,
	EngineVersionRange,
	GameMod,
	GameModInfo,
	RemoteConfigs,
	commands,
} from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconCalendarFilled,
	IconCheck,
	IconDotsVertical,
	IconDownload,
	IconFolderOpen,
	IconGizmo,
	IconMinus,
	IconSettings,
	IconSettingsFilled,
	IconUserFilled,
} from "@tabler/icons-react";
import { OutdatedMarker } from "@components/outdated-marker";
import { CommandDropdown } from "@components/command-dropdown";
import { DeprecatedBadge } from "@components/mods/deprecated-badge";
import { useLocalization } from "@hooks/use-localization";
import { GameModInstallButton } from "./game-mod-install-button";
import { GameModRunButton } from "./game-mod-run-button";
import { GameModUpdateButton } from "./game-mod-update-button";
import { GameModUninstallButton } from "./game-mod-uninstall-button";
import { dateFormatter } from "../../date-formatter";

type Props = {
	readonly game: DbGame;
	readonly mod: GameMod;
	readonly remoteConfigs?: RemoteConfigs | null;
	readonly info?: GameModInfo;
	readonly incompatible?: boolean;
};

function renderVersionRange(range: EngineVersionRange | null) {
	if (!range) return null;

	return [range.minimum, range.maximum]
		.map(renderVersionNumbers)
		.filter(Boolean)
		.join(" - ");
}

function renderVersionNumbers(version: EngineVersionNumbers | null) {
	if (!version) return null;

	return [version.major, version.minor, version.patch]
		.filter(Boolean)
		.join(".");
}

export function GameModRow({
	game,
	mod,
	info,
	remoteConfigs,
	incompatible = false,
}: Props) {
	const { t } = useLocalization("gameModRow");

	const availableRemoteConfig = remoteConfigs?.configs.find(
		(config) => config.modId === (mod.config?.modIdOverride ?? mod.id),
	);

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
				<Stack>
					<Group wrap="nowrap">
						{isModUsable && (
							<ThemeIcon
								color={statusColor}
								size="sm"
							>
								{statusIcon}
							</ThemeIcon>
						)}
						<Stack gap={0}>
							<Group>
								<Text
									fw="bold"
									size="lg"
								>
									{mod.title}
								</Text>
								{availableRemoteConfig && (
									<Tooltip label={t("remoteConfigAvailable")}>
										<IconSettingsFilled fontSize={15} />
									</Tooltip>
								)}
							</Group>
							<Group
								gap={0}
								style={{ columnGap: 10 }}
								fz="xs"
								opacity={0.4}
							>
								{versionText && <span>{versionText}</span>}
								{mod.download?.releaseDate && (
									<Group gap={2}>
										<IconCalendarFilled fontSize={10} />
										<span>
											{dateFormatter.format(new Date(mod.download.releaseDate))}
										</span>
									</Group>
								)}
								<Group gap={2}>
									<IconUserFilled fontSize={10} />
									<span>{mod.author}</span>
								</Group>
								<Group gap={2}>
									<IconGizmo fontSize={10} />
									<span>
										{[
											mod.engine,
											renderVersionRange(mod.engineVersionRange),
											mod.unityBackend,
											mod.gameOs,
											mod.architecture,
										]
											.filter(Boolean)
											.join(" ")}
									</span>
								</Group>
							</Group>

							<Text
								fz="sm"
								opacity={0.75}
							>
								{mod.description}
							</Text>
						</Stack>
						{mod?.deprecated && <DeprecatedBadge size="sm" />}
					</Group>
				</Stack>
			</Table.Td>
			<Table.Td maw={200}>
				<Group justify="right">
					{isModUsable && (
						<ButtonGroup>
							{!isInstalled && !isOutdated && mod.install && (
								<GameModInstallButton
									game={game}
									mod={mod}
									remoteConfigFile={availableRemoteConfig?.file}
								/>
							)}
							{isOutdated && (
								<GameModUpdateButton
									game={game}
									mod={mod}
									remoteConfigFile={availableRemoteConfig?.file}
								/>
							)}
							{mod.runForGame && (!mod.install || isInstalled) && (
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
								{availableRemoteConfig && (
									<CommandButton
										disabled={!isInstalled}
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
