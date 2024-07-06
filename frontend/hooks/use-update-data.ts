import { useCallback } from "react";
import { useSetAtom } from "jotai";
import { commands } from "@api/bindings";
import { loadingAtom } from "./use-data";
import { showAppNotification } from "@components/app-notifications";

export function useUpdateData() {
	const setIsLoading = useSetAtom(loadingAtom);

	const updateAppData = useCallback(() => {
		setIsLoading(true);
		commands
			.updateData()
			.then((result) => {
				if (result.status === "error") {
					showAppNotification(
						`Error while updating data: ${result.error}`,
						"error",
					);
				}
			})
			.catch((error) => {
				showAppNotification(
					`Failed to initialize data update: ${error}`,
					"error",
				);
			})
			.finally(() => setIsLoading(false));
	}, [setIsLoading]);

	return updateAppData;
}
