import { useSetAtom, atom } from "jotai";
import { useAsyncCommand } from "./use-async-command";
import { useEffect } from "react";
import { Result } from "@api/bindings";
import { loadingCountAtom } from "./use-data";

export function commandData<TResultValue, TError>(
	command: () => Promise<Result<TResultValue, TError>>,
	defaultValue: TResultValue,
) {
	const stateAtom = atom<TResultValue>(defaultValue);

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
