import { showAppNotification } from "@components/app-notifications";
import { PrimitiveAtom, useSetAtom } from "jotai";
import { useCallback, useEffect, useRef, useState } from "react";
import { useAsyncCommand } from "./use-async-command";

export function useCommandData<TResultValue>(
	command: () => Promise<TResultValue>,
	defaultValue: TResultValue,
	skip = false,
) {
	const [getValue] = useAsyncCommand(command);
	const [value, setValue] = useState<TResultValue>(defaultValue);

	const updateData = useCallback(() => {
		if (skip) return;
		getValue().then(setValue);
	}, [getValue, skip]);

	useEffect(updateData, [updateData]);

	return [value, updateData] as const;
}

export function useCommandAtomData<TResultValue>(
	command: () => Promise<TResultValue>,
	atom: PrimitiveAtom<TResultValue>,
) {
	const setValue = useSetAtom(atom);
	const totalFetchCount = useRef(0);

	const updateData = useCallback(async () => {
		totalFetchCount.current += 1;
		const thisFetchCount = totalFetchCount.current;

		try {
			const data = await command();
			if (thisFetchCount !== totalFetchCount.current) {
				console.log(
					"Cancelling this fetch since another one happened in the meantime.",
				);
				return;
			}
			setValue(data);
		} catch (error) {
			showAppNotification(`Failed to get app data: ${error}`, "error");
		}
	}, [command, setValue]);

	return updateData;
}
