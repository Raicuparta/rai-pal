import { useCallback } from "react";
import { useSetAtom } from "jotai";
import { updateData } from "@api/bindings";
import { loadingAtom } from "./use-data";
import { showAppNotification } from "@components/app-notifications";

export function useUpdateData() {
	const setIsLoading = useSetAtom(loadingAtom);

	const updateAppData = useCallback(() => {
		setIsLoading(true);
		updateData()
			.catch((error) => {
				showAppNotification(`Error finding games: ${error}`, "error");
			})
			.finally(() => setIsLoading(false));
	}, [setIsLoading]);

	return updateAppData;
}
