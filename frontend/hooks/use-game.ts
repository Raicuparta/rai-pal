import { commands, DbGame, GameId } from "@api/bindings";
import { useAppEvent } from "./use-app-event";
import { useCommandData } from "./use-command-data";

export function useGame({ providerId, gameId }: GameId) {
	// const [getGame] = useAsyncCommand(commands.getGame);
	const defaultGame: DbGame = {
		providerId: providerId,
		gameId: gameId,
		displayTitle: "...",
		engineBrand: null,
		engineVersion: null,
		exePath: null,
		externalId: "",
		releaseDate: null,
		thumbnailUrl: null,
		architecture: null,
		unityBackend: null,
		titleDiscriminator: null,
		providerCommands: {},
		tags: [],
	};
	const [game, updateGame] = useCommandData(
		commands.getGame,
		() => ({ args: { providerId, gameId } }),
		defaultGame,
	);

	useAppEvent("foundGame", `${providerId}:${gameId}`, (foundId) => {
		if (foundId.providerId !== providerId || foundId.gameId !== gameId) return;
		updateGame();
	});

	return game;
}
