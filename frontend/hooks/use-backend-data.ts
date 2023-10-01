import { getModLoaders, getOwnedGames } from "@api/bindings";
import { useCallback, useEffect, useState } from "react";

type ApiFunction<TResultData> = (ignoreCache: boolean) => Promise<TResultData>;

export function useBackendData<TData>(
	apiFunction: ApiFunction<TData>,
	defaultData: TData,
) {
	const [data, setData] = useState<TData>(defaultData);
	const [isLoading, setIsLoading] = useState(false);
	const [error, setError] = useState("");

	const clearError = useCallback(() => setError(""), []);

	const updateData = useCallback(
		(ignoreCache = false) => {
			setIsLoading(true);
			clearError();

			apiFunction(ignoreCache)
				.then(setData)
				.catch((error) => setError(`Error: ${error}`))
				.finally(() => setIsLoading(false));
		},
		[apiFunction, clearError],
	);

	const refresh = useCallback(() => {
		updateData(true);
	}, [updateData]);

	useEffect(() => {
		updateData();
	}, [updateData]);

	return [data, isLoading, refresh, error, clearError] as const;
}

export const useOwnedGames = () => useBackendData(getOwnedGames, []);
export const useModLoaders = () => useBackendData(getModLoaders, {});
