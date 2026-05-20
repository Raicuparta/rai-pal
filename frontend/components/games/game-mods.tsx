import { Alert, Card, Divider, Stack, Table } from "@mantine/core";
import { EngineVersionRange, DbGame, commands, GameMod } from "@api/bindings";
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

const defaultInstalledMods: Record<string, GameMod> = {};

export function GameMods({ game }: Props) {
	const t = useLocalization("gameModal");
	const mods = useUnifiedMods();
	const getInstalledMods = useCallback(
		() => commands.getInstalledMods(game.providerId, game.gameId),
		[game],
	);
	const [installedMods, updateInstalledMods] = useCommandData(
		getInstalledMods,
		defaultInstalledMods,
		!game?.exePath,
	);
	const getRemoteConfigs = useCallback(
		() => commands.getRemoteConfigs(game.providerId, game.gameId),
		[game],
	);
	const [remoteConfigs] = useCommandData(
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
			updateInstalledMods();
		},
	);

	const { compatibleMods, incompatibleMods } = useMemo(() => {
		const compatibleMods: UnifiedMod[] = [];
		const incompatibleMods: UnifiedMod[] = [];

		for (const mod of Object.values(mods)) {
			const isCompatibleEngine =
				!mod.merged.engine || mod.merged.engine === game.engineBrand;
			const isCompatibleUnityBackend =
				!mod.merged.unityBackend ||
				!game.unityBackend ||
				mod.merged.unityBackend === game.unityBackend;
			const isCompatibleArchitecture =
				!mod.merged.architecture ||
				!game.architecture ||
				mod.merged.architecture === game.architecture;

			if (
				!game ||
				!isCompatibleEngine ||
				!isCompatibleUnityBackend ||
				!isCompatibleArchitecture
			) {
				continue;
			}

			// Deprecated mods only show if they had been previously installed.
			if (mod.remote?.deprecated && !installedMods[mod.merged.id]) {
				continue;
			}

			// Non-actionable mods can be skipped.
			if (!mod.merged.runForGame && !mod.merged.install) {
				continue;
			}

			if (isVersionWithinRange(game, mod.merged.engineVersionRange)) {
				compatibleMods.push(mod);
			} else {
				incompatibleMods.push(mod);
			}
		}

		return {
			compatibleMods,
			incompatibleMods,
		};
	}, [game, installedMods, mods]);

	if (compatibleMods.length + incompatibleMods.length === 0) {
		return null;
	}

	return (
		<>
			<Stack>
				{compatibleMods.length > 0 && (
					<>
						<Divider label={t("gameModsLabel")} />
						{!game.exePath && (
							<Alert color="orange">{t("gameNotInstalledWarning")}</Alert>
						)}
						<Table
							highlightOnHover
							highlightOnHoverColor="dark.7"
						>
							<Table.Tbody>
								{compatibleMods.map((mod) => (
									<GameModRow
										key={mod.merged.id}
										game={game}
										mod={mod}
										remoteConfigs={remoteConfigs}
										installedMod={installedMods[mod.merged.id]}
									/>
								))}
							</Table.Tbody>
						</Table>
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
			</Stack>
			{incompatibleMods.length > 0 && (
				<Stack>
					<Divider label={t("incompatibleGameModsLabel")} />
					<MutedText>{t("incompatibleGameModsDescription")}</MutedText>
					<Table>
						<Table.Tbody>
							{incompatibleMods.map((mod) => (
								<GameModRow
									key={mod.merged.id}
									game={game}
									mod={mod}
									remoteConfigs={remoteConfigs}
									installedMod={installedMods[mod.merged.id]}
									incompatible
								/>
							))}
						</Table.Tbody>
					</Table>
				</Stack>
			)}
		</>
	);
}
