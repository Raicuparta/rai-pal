import { Alert, Divider, Table, Text } from "@mantine/core";
import { EngineVersionRange, DbGame, commands } from "@api/bindings";
import { useCallback, useMemo } from "react";
import { CommandButton } from "@components/command-button";
import { IconTrash } from "@tabler/icons-react";
import { UnifiedMod, useUnifiedMods } from "@hooks/use-unified-mods";
import { GameModRow } from "./game-mod-row";
import { TableContainer } from "@components/table/table-container";
import { useLocalization } from "@hooks/use-localization";
import { useCommandData } from "@hooks/use-command-data";
import { useAppEvent } from "@hooks/use-app-event";
import { MutedText } from "@components/muted-text";

type Props = {
	readonly game: DbGame;
};

function isVersionWithinRange(
	{
		engineVersionMajor: major,
		engineVersionMinor: minor,
		engineVersionPatch: patch,
	}: DbGame,
	range: EngineVersionRange | null,
) {
	if (!major || !range) return true;

	if (!major) return false;

	const { minimum, maximum } = range;

	if (minimum && minimum.major > major) return false;
	if (maximum && maximum.major < major) return false;
	if (
		minimum &&
		minimum.major === major &&
		minimum.minor != null &&
		minor != null &&
		minimum.minor > minor
	)
		return false;
	if (
		maximum &&
		maximum.major === major &&
		maximum.minor != null &&
		minor != null &&
		maximum.minor < minor
	)
		return false;
	if (
		minimum &&
		minimum.major === major &&
		minimum.minor === minor &&
		minimum.patch != null &&
		patch != null &&
		minimum.patch > patch
	)
		return false;
	if (
		maximum &&
		maximum.major === major &&
		maximum.minor === minor &&
		maximum.patch != null &&
		patch != null &&
		maximum.patch < patch
	)
		return false;

	return true;
}

const defaultInstalledModVersions: Record<string, string> = {};

export function GameMods({ game }: Props) {
	const t = useLocalization("gameModal");
	const mods = useUnifiedMods();
	const getInstalledModVersions = useCallback(
		() => commands.getInstalledModVersions(game.providerId, game.gameId),
		[game],
	);
	const [installedModVersions, updateInstalledModVersions] = useCommandData(
		getInstalledModVersions,
		defaultInstalledModVersions,
		!game?.exePath,
	);
	const getRemoteConfigs = useCallback(
		() => commands.getRemoteConfigs(game.providerId, game.gameId),
		[game],
	);
	const [remoteConfigs, updateRemoteConfigs] = useCommandData(
		getRemoteConfigs,
		null,
		!game?.exePath,
	);

	useAppEvent(
		"refreshGame",
		`installed-mods-${game.providerId}:${game.gameId}`,
		([refreshedProviderId, refreshedGameId]) => {
			if (
				refreshedProviderId !== game.providerId ||
				refreshedGameId !== game.gameId
			)
				return;
			updateInstalledModVersions();
		},
	);

	const { compatibleMods, incompatibleMods } = useMemo(() => {
		const compatibleMods: UnifiedMod[] = [];
		const incompatibleMods: UnifiedMod[] = [];

		for (const mod of Object.values(mods)) {
			const isCompatibleEngine =
				!mod.common.engine || mod.common.engine === game.engineBrand;
			const isCompatibleUnityBackend =
				!mod.common.unityBackend ||
				!game.unityBackend ||
				mod.common.unityBackend === game.unityBackend;

			if (!game || !isCompatibleEngine || !isCompatibleUnityBackend) {
				continue;
			}

			// Deprecated mods only show if they had been previously installed.
			if (mod.remote?.deprecated && !installedModVersions[mod.common.id]) {
				continue;
			}

			if (isVersionWithinRange(game, mod.common.engineVersionRange)) {
				compatibleMods.push(mod);
			} else {
				incompatibleMods.push(mod);
			}
		}

		return {
			compatibleMods,
			incompatibleMods,
		};
	}, [game, installedModVersions, mods]);

	if (compatibleMods.length + incompatibleMods.length === 0) {
		return null;
	}

	return (
		<>
			{compatibleMods.length > 0 && (
				<>
					<Divider label={t("gameModsLabel")} />
					{!game.exePath && (
						<Alert color="orange">{t("gameNotInstalledWarning")}</Alert>
					)}
					<TableContainer bg="dark">
						<Table>
							<Table.Tbody>
								{compatibleMods.map((mod) => (
									<GameModRow
										key={mod.common.id}
										game={game}
										mod={mod}
										remoteConfigs={remoteConfigs}
										installedVersion={installedModVersions[mod.common.id]}
									/>
								))}
							</Table.Tbody>
						</Table>
					</TableContainer>
				</>
			)}
			{incompatibleMods.length > 0 && (
				<>
					<Divider label={t("incompatibleGameModsLabel")} />
					<MutedText>{t("incompatibleGameModsDescription")}</MutedText>
					<TableContainer bg="dark">
						<Table>
							<Table.Tbody>
								{incompatibleMods.map((mod) => (
									<GameModRow
										key={mod.common.id}
										game={game}
										mod={mod}
										remoteConfigs={remoteConfigs}
										installedVersion={installedModVersions[mod.common.id]}
										incompatible
									/>
								))}
							</Table.Tbody>
						</Table>
					</TableContainer>
				</>
			)}
			{game.exePath && (
				<CommandButton
					confirmationText={t("uninstallAllModsConfirmation")}
					onClick={() =>
						commands.uninstallAllMods(game.providerId, game.gameId)
					}
					color="red"
					variant="light"
					leftSection={<IconTrash />}
				>
					{t("uninstallAllModsButton")}
				</CommandButton>
			)}
		</>
	);
}
