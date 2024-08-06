import { useSetAtom, atom } from "jotai";
import { atomWithStorage } from "jotai/utils";
import { useAsyncCommand } from "./use-async-command";
import { useEffect } from "react";
import { type commands, Result } from "@api/bindings";
import { loadingCountAtom } from "./use-data";

export function commandData<TResultValue, TError>(
	id: keyof typeof commands,
	command: () => Promise<Result<TResultValue, TError>>,
	defaultValue: TResultValue,
) {
	const stateAtom = atomWithStorage<TResultValue>(id, defaultValue);

	function useCommandData() {
		const [asyncCommand] = useAsyncCommand(command);
		const setData = useSetAtom(stateAtom);
		const setLoadingCount = useSetAtom(loadingCountAtom);

		useEffect(() => {
			setLoadingCount((count) => count + 1);
			asyncCommand()
				.then((resultData) => {
					if (resultData) {
						setData(resultData);
					}
				})
				.finally(() => {
					setLoadingCount((count) => count - 1);
				});
		}, [asyncCommand, setData, setLoadingCount]);
	}

	return [stateAtom, useCommandData] as const;
}
