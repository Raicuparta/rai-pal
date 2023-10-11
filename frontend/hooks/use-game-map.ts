import { Game, commands } from "@api/bindings";
import { useCallback, useEffect, useState } from "react";

const defaultData = {};

export const useGameMap = () => {
	const [data, setData] = useState<Record<string, Game>>(defaultData);
	const [isLoading, setIsLoading] = useState(false);
	const [error, setError] = useState("");

	const clearError = useCallback(() => setError(""), []);

	const updateData = useCallback(
		(ignoreCache = false) => {
			setIsLoading(true);
			clearError();

			commands
				.getGameMap(ignoreCache)
				.then(setData)
				.catch((error) => setError(`Error: ${error}`))
				.finally(() => setIsLoading(false));
		},
		[clearError],
	);

	const refresh = useCallback(() => {
		updateData(true);
	}, [updateData]);

	const refreshGame = useCallback((gameId: string) => {
		setIsLoading(true);
		setError("");

		commands
			.updateGameInfo(gameId)
			.then(setData)
			.catch((error) => setError(`Error: ${error}`))
			.finally(() => setIsLoading(false));
	}, []);

	useEffect(() => {
		updateData();
	}, [updateData]);

	return [data, isLoading, refresh, refreshGame, error, clearError] as const;
};
