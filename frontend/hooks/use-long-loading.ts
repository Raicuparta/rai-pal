import { useEffect, useRef, useState } from "react";

export function useLongLoading(isLoading: boolean) {
	const [isLongLoading, setIsLongLoading] = useState(false);
	const timeout = useRef<ReturnType<typeof setTimeout>>(undefined);

	useEffect(() => {
		if (!isLoading) return;

		timeout.current = setTimeout(() => setIsLongLoading(true), 500);

		return () => {
			clearTimeout(timeout.current);
			setIsLongLoading(false);
		};
	}, [isLoading]);

	return isLongLoading;
}
