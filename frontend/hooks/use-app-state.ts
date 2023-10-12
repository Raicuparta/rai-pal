import { FullState, getFullState } from "@api/bindings";
import { atom, useAtom } from "jotai";
import { useCallback, useEffect } from "react";

type State = {
	isReady: boolean;
	data: FullState;
};

const stateAtom = atom<State>({
	isReady: false,
	data: {
		gameMap: {},
		ownedGames: [],
		discoverGames: [],
		modLoaders: {},
	},
});

const isLoadingAtom = atom(false);
const errorAtom = atom("");

export function useAppState() {
	const [appState, setAppState] = useAtom(stateAtom);
	const [isLoading, setIsLoading] = useAtom(isLoadingAtom);
	const [error, setError] = useAtom(errorAtom);

	const clearError = useCallback(() => setError(""), []);

	const updateData = useCallback(
		(ignoreCache = false) => {
			if (!ignoreCache && appState.isReady) return;

			setIsLoading(true);
			clearError();

			getFullState()
				.then((data) => setAppState({ data, isReady: true }))
				.catch((error) => setError(`Error: ${error}`))
				.finally(() => setIsLoading(false));
		},
		[clearError],
	);

	const refresh = useCallback(() => {
		updateData(true);
	}, [updateData]);

	useEffect(() => {
		updateData();
	}, [updateData]);

	return [appState, isLoading, refresh, error, clearError] as const;
}
