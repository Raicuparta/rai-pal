import { useSetAtom, atom } from "jotai";
import { useAsyncCommand } from "./use-async-command";
import { useEffect } from "react";
import { Result } from "@api/bindings";

export function commandData<TResultValue, TError>(
	command: () => Promise<Result<TResultValue, TError>>,
	defaultValue: TResultValue,
) {
	const stateAtom = atom<TResultValue>(defaultValue);

	function useCommandData() {
		const [asyncCommand] = useAsyncCommand(command);
		const setData = useSetAtom(stateAtom);

		useEffect(() => {
			asyncCommand().then((resultData) => {
				if (resultData) {
					setData(resultData);
				}
			});
		}, [asyncCommand, setData]);
	}

	return [stateAtom, useCommandData] as const;
}
