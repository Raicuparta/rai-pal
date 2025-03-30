import { useCallback, useEffect, useRef, useState } from "react";
import { useAsyncCommand } from "./use-async-command";

export function useCommandData<TResultValue, TArgs = void>(
	command: (args: TArgs) => Promise<TResultValue>,
	getArgs: () => { args: TArgs; skip?: false } | { args?: unknown; skip: true },
	defaultValue: TResultValue,
) {
	const [getValue] = useAsyncCommand(command);
	const [value, setValue] = useState<TResultValue>(defaultValue);

	const getArgsRef = useRef(getArgs);
	useEffect(() => {
		getArgsRef.current = getArgs;
	}, [getArgs]);

	const updateData = useCallback(() => {
		const { args, skip } = getArgsRef.current();
		if (skip) return;
		getValue(args).then(setValue);
	}, [getValue]);

	useEffect(updateData, [updateData]);

	return [value, updateData] as const;
}
