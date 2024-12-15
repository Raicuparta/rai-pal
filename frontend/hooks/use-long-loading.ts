import { useCallback, useEffect, useRef, useState } from "react";

export function useLongLoading(isLoading: boolean) {
	const [isLongLoading, setIsLongLoading] = useState(false);
	const timeout = useRef<number>();

	const timeoutCallback = useCallback(() => {
		if (!isLoading) return;
		setIsLongLoading(true);
	}, [isLoading]);

	useEffect(() => {
		setIsLongLoading(false);
		if (!isLoading) return;

		// eslint-disable-next-line @typescript-eslint/ban-ts-comment
		// @ts-expect-error
		timeout.current = setTimeout(timeoutCallback, 500);

		return () => clearTimeout(timeout.current);
	}, [isLoading, timeoutCallback]);

	return isLongLoading;
}
