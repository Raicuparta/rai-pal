import { commands, DbGame, ProviderId } from "@api/bindings";
import { useAppEvent } from "./use-app-event";
import { useCommandData } from "./use-command-data";

export function useGame(providerId: ProviderId, gameId: string) {
	const defaultGame: DbGame = {
		providerId: providerId,
		gameId: gameId,
		displayTitle: "...",
		engineBrand: null,
		engineVersionMajor: null,
		engineVersionMinor: null,
		engineVersionPatch: null,
		engineVersionDisplay: null,
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
		() => commands.getGame(providerId, gameId),
		defaultGame,
	);

	useAppEvent(
		"refreshGame",
		`game-${providerId}:${gameId}`,
		([refreshedProviderId, refreshedGameId]) => {
			if (refreshedProviderId !== providerId || refreshedGameId !== gameId)
				return;
			updateGame();
		},
	);

	return game;
}
