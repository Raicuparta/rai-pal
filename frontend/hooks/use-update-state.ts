import { useCallback } from "react";
import { useSetAtom } from "jotai";
import { updateState } from "@api/bindings";
import { errorAtom, loadingAtom } from "./use-app-state";

export function useUpdateAppState() {
	const setError = useSetAtom(errorAtom);
	const setIsLoading = useSetAtom(loadingAtom);

	const updateAppState = useCallback(() => {
		setIsLoading(true);
		updateState()
			.catch(setError)
			.finally(() => setIsLoading(false));
	}, [setError, setIsLoading]);

	return updateAppState;
}
