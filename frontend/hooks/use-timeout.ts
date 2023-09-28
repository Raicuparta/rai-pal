import { useRef, useEffect, useCallback } from "react";

// Copied mostly from mantine's useTimeout hook,
// fixed issues with references changing more than they needed.

export function useTimeout(
	callback: (...callbackParams: unknown[]) => void,
	delay: number,
	options: { autoInvoke: boolean } = { autoInvoke: false },
) {
	const timeoutRef = useRef<number | null>(null);

	const start = useCallback(
		(...callbackParams: unknown[]) => {
			if (!timeoutRef.current) {
				timeoutRef.current = window.setTimeout(() => {
					callback(callbackParams);
					timeoutRef.current = null;
				}, delay);
			}
		},
		[callback, delay],
	);

	const clear = useCallback(() => {
		if (timeoutRef.current) {
			window.clearTimeout(timeoutRef.current);
			timeoutRef.current = null;
		}
	}, []);

	useEffect(() => {
		if (options.autoInvoke) {
			start();
		}

		return clear;
	}, [clear, delay, options.autoInvoke, start]);

	return { start, clear };
}
