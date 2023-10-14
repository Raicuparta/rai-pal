import { useCallback } from "react";
import { useSetAtom } from "jotai";
import { updateData } from "@api/bindings";
import { errorAtom, loadingAtom } from "./use-data";

export function useUpdateData() {
	const setError = useSetAtom(errorAtom);
	const setIsLoading = useSetAtom(loadingAtom);

	const updateAppData = useCallback(() => {
		setIsLoading(true);
		updateData()
			.catch(setError)
			.finally(() => setIsLoading(false));
	}, [setError, setIsLoading]);

	return updateAppData;
}
