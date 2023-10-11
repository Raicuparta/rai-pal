import { commands } from "@api/bindings";
import { useCallback, useEffect, useState } from "react";

type ApiFunction<TResultData> = (
	ignoreCache: boolean,
) => Promise<Result<TResultData>>;

type Result<T> = { status: "ok"; data: T } | { status: "error"; error: string };

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
				.then((result) => {
					if (result.status == "ok") {
						setData(result.data);
					} else {
						setError(result.error);
					}
				})
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

export const useOwnedGames = () => useBackendData(commands.getOwnedGames, []);
export const useModLoaders = () => useBackendData(commands.getModLoaders, {});
export const useDiscoverGames = () =>
	useBackendData(commands.getDiscoverGames, []);
