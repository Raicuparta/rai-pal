import { useCallback, useEffect, useState } from "react";
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
