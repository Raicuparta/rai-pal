import { showAppNotification } from "@components/app-notifications";
import { atom, useSetAtom } from "jotai";
import { useCallback } from "react";

export type RefreshTask = {
	id: number;
	name: string;
};

export const refreshTasksAtom = atom<RefreshTask[]>([]);

export const isRefreshingAtom = atom((get) => get(refreshTasksAtom).length > 0);

let refreshTaskCounter = 0;

function generateRefreshTaskId() {
	refreshTaskCounter += 1;
	return refreshTaskCounter;
}

export function useRefreshTask() {
	const setRefreshTasks = useSetAtom(refreshTasksAtom);

	return useCallback(
		(name: string, promise: Promise<unknown>) => {
			const id = generateRefreshTaskId();
			setRefreshTasks((tasks) => [...tasks, { id, name }]);
			return promise
				.catch((error) => {
					showAppNotification(`Failed to refresh ${name}: ${error}`, "error");
				})
				.finally(() => {
					setRefreshTasks((tasks) => tasks.filter((task) => task.id !== id));
				});
		},
		[setRefreshTasks],
	);
}
