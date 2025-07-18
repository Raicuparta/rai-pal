import { Alert, Group, Modal, Stack, Table } from "@mantine/core";
import { commands, ProviderId } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconFolder,
	IconFolderCog,
	IconFolderOpen,
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
import { useGame } from "@hooks/use-game";
import { RemoveGameButton } from "./remove-game-button";
import { GameMods } from "./game-mods";

type Props = {
	readonly providerId: ProviderId;
	readonly gameId: string;
};

export function GameModal({ providerId, gameId }: Props) {
	const t = useLocalization("gameModal");
	const game = useGame(providerId, gameId);
	const setSelectedGame = useSetAtom(selectedGameAtom);

	const close = () => setSelectedGame(null);

	if (!game) {
		return null;
	}

	return (
		<Modal
			centered
			onClose={close}
			opened
			size="xl"
			title={game.displayTitle}
		>
			<Stack>
				<Group align="start">
					<TableContainer>
						<Table>
							<Table.Thead>
								<TableHead columns={gamesColumns} />
							</Table.Thead>
							<Table.Tbody>
								<GameRowInner game={game} />
							</Table.Tbody>
						</Table>
					</TableContainer>
				</Group>
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
		</Modal>
	);
}
