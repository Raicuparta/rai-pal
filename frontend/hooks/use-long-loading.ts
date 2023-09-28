import { useTimeout } from "@mantine/hooks";
import { useCallback, useEffect, useState } from "react";

export function useLongLoading(isLoading: boolean) {
	const [isLongLoading, setIsLongLoading] = useState(false);

	const timeoutCallback = useCallback(() => {
		if (!isLoading) return;
		setIsLongLoading(true);
	}, [isLoading]);

	const { start, clear } = useTimeout(timeoutCallback, 100);

	useEffect(() => {
		setIsLongLoading(false);
		if (!isLoading) return;

		start();

		return clear;
	}, [clear, isLoading, start]);

	return isLongLoading;
}
