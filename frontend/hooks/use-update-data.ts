import { useCallback } from "react";
import { useSetAtom } from "jotai";
import { updateData } from "@api/bindings";
import { loadingAtom } from "./use-data";
import { notifications } from "@mantine/notifications";

export function useUpdateData() {
	const setIsLoading = useSetAtom(loadingAtom);

	const updateAppData = useCallback(() => {
		setIsLoading(true);
		updateData()
			.catch((error) => {
				notifications.show({
					message: `Error finding games: ${error}`,
					color: "red",
				});
			})
			.finally(() => setIsLoading(false));
	}, [setIsLoading]);

	return updateAppData;
}
