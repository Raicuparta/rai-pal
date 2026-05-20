import { Alert, Box, Group, Stack, Table } from "@mantine/core";
import { commands, DbGame } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconFolder,
	IconFolderCog,
	IconFolderOpen,
	IconGlassFull,
	IconRefresh,
} from "@tabler/icons-react";
import { useSetAtom } from "jotai";
import { DebugData } from "@components/debug-data";
import { TableContainer } from "@components/table/table-container";
import { CommandDropdown } from "@components/command-dropdown";
import { selectedGameAtom } from "./games-state";
import { ProviderCommandButtons } from "@components/providers/provider-command-dropdown";
import { GameRowInner } from "./game-row";
import { TableHead } from "@components/table/table-head";
import { gamesColumns } from "./games-columns";
import { useLocalization } from "@hooks/use-localization";
import { RemoveGameButton } from "./remove-game-button";
import { GameMods } from "./game-mods";
import { platform } from "@tauri-apps/plugin-os";

type Props = {
	readonly game: DbGame;
};

export function GameModal({ game }: Props) {
	const t = useLocalization("gameModal");
	const setSelectedGame = useSetAtom(selectedGameAtom);

	const close = () => setSelectedGame(null);
	const { providerId, gameId } = game;

	return (
		<Stack
			flex={1}
			style={{ overflowY: "scroll" }}
		>
			<Box>
				<TableContainer singleItem>
					<Table highlightOnHover>
						<Table.Thead>
							<TableHead columns={gamesColumns} />
						</Table.Thead>
						<Table.Tbody>
							<GameRowInner
								game={game}
								onClick={close}
							/>
						</Table.Tbody>
					</Table>
				</TableContainer>
			</Box>
			<Stack pr={10}>
				<Group>
					<ProviderCommandButtons game={game} />
					{game.exePath && (
						<CommandDropdown
							label={t("foldersDropdown")}
							icon={<IconFolderOpen />}
						>
							<CommandButton
								leftSection={<IconFolder />}
								onClick={() => commands.openGameFolder(providerId, gameId)}
							>
								{t("openGameFilesFolder")}
							</CommandButton>
							<CommandButton
								leftSection={<IconFolderCog />}
								onClick={() => commands.openGameModsFolder(providerId, gameId)}
							>
								{t("openInstalledModsFolder")}
							</CommandButton>
							{platform() === "linux" && (
								<>
									<CommandButton
										leftSection={<IconGlassFull />}
										onClick={() =>
											commands.openGameWinePrefixFolder(providerId, gameId)
										}
									>
										{t("openGameWinePrefixFolder")}
									</CommandButton>
									<CommandButton
										leftSection={<IconGlassFull />}
										onClick={() =>
											commands.openGameWineBinaryFolder(providerId, gameId)
										}
									>
										{t("openGameWineBinaryFolder")}
									</CommandButton>
								</>
							)}
						</CommandDropdown>
					)}
					{providerId === "Manual" && (
						<RemoveGameButton
							providerId={providerId}
							gameId={gameId}
						/>
					)}
					{game.exePath && (
						<CommandButton
							onClick={() => commands.refreshGame(providerId, gameId)}
							leftSection={<IconRefresh />}
						>
							{t("refreshGame")}
						</CommandButton>
					)}
				</Group>
				{game.exePath && (
					<>
						{game.engineBrand && !game.architecture && (
							<Alert color="red">{t("failedToReadGameInfo")}</Alert>
						)}
						{!game.engineBrand && (
							<Alert color="red">{t("failedToDetermineEngine")}</Alert>
						)}
					</>
				)}
				<GameMods game={game} />
				<DebugData data={game} />
			</Stack>
		</Stack>
	);
}
