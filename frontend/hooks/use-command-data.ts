import { useCallback, useEffect, useState } from "react";
import { useAsyncCommand } from "./use-async-command";

export function useCommandData<TResultValue, TArgs = void>(
	command: (args: TArgs) => Promise<TResultValue>,
	args: TArgs,
	defaultValue: TResultValue,
	skip = false,
) {
	const [getValue] = useAsyncCommand(command);
	const [value, setValue] = useState<TResultValue>(defaultValue);

	const updateData = useCallback(() => {
		if (skip) return;
		getValue(args).then(setValue);
	}, [args, getValue, skip]);

	useEffect(updateData, [updateData]);

	return [value, updateData] as const;
}
