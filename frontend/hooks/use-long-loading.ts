import { useCallback, useEffect, useState } from "react";
import { useTimeout } from "./use-timeout";

export function useLongLoading(isLoading: boolean) {
	const [isLongLoading, setIsLongLoading] = useState(false);

	const timeoutCallback = useCallback(() => {
		if (!isLoading) return;
		setIsLongLoading(true);
	}, [isLoading]);

	const { start, clear } = useTimeout(timeoutCallback, 200);

	useEffect(() => {
		setIsLongLoading(false);
		if (!isLoading) return;

		start();

		return clear;
	}, [clear, isLoading, start]);

	useEffect(() => {}, [start]);

	return isLongLoading;
}
