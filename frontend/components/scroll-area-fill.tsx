import { ScrollArea, ScrollAreaProps } from "@mantine/core";
import { useMergedRef } from "@mantine/hooks";
import { forwardRef, useEffect, useRef } from "react";

export const ScrollAreaFill = forwardRef<HTMLDivElement, ScrollAreaProps>(
	function ScrollAreaFill(
		{ children, viewportRef: forwardedViewportRef, ...props },
		forwardedRef,
	) {
		const viewportRef = useRef<HTMLDivElement>(null);
		const mergedRef = useMergedRef(
			viewportRef,
			forwardedViewportRef,
			forwardedRef,
		);

		useEffect(() => {
			const viewportElement = viewportRef.current;
			if (!(viewportElement instanceof HTMLElement)) return;
			const child = viewportElement.firstElementChild;
			if (!(child instanceof HTMLElement)) return;
			const viewport = viewportElement;
			const content = child;

			let lastScrollHeight = -1;
			let nudgeTimeout: number | undefined;
			let pendingMaxHeight: string | null = null;

			function restoreNudge() {
				window.clearTimeout(nudgeTimeout);
				nudgeTimeout = undefined;
				if (pendingMaxHeight !== null) {
					content.style.maxHeight = pendingMaxHeight;
					pendingMaxHeight = null;
				}
			}

			// These styles keep the content at a fixed size so the inner layout
			// can scroll, which means Mantine never notices when the content
			// grows and hides the scrollbars. Shrinking the content by a pixel for
			// a moment forces Mantine to re-measure.
			function checkOverflow() {
				if (viewport.scrollHeight === lastScrollHeight) return;
				lastScrollHeight = viewport.scrollHeight;
				if (nudgeTimeout !== undefined) return;

				pendingMaxHeight = content.style.maxHeight;
				content.style.maxHeight = `${content.offsetHeight - 1}px`;
				nudgeTimeout = window.setTimeout(restoreNudge, 50);
			}

			const mutationObserver = new MutationObserver((mutations) => {
				const isOwnNudge = mutations.every(
					(mutation) => mutation.target === content,
				);
				if (isOwnNudge) return;
				checkOverflow();
			});
			mutationObserver.observe(content, {
				childList: true,
				subtree: true,
				// Collapsible content animates through inline styles rather than
				// adding or removing nodes.
				attributes: true,
				attributeFilter: ["style"],
			});
			const resizeObserver = new ResizeObserver(checkOverflow);
			resizeObserver.observe(viewport);
			checkOverflow();

			return () => {
				mutationObserver.disconnect();
				resizeObserver.disconnect();
				restoreNudge();
			};
		}, []);

		return (
			<ScrollArea
				{...props}
				styles={{
					root: { display: "flex", flexDirection: "column" },
					viewport: {
						display: "flex",
						flexDirection: "column",
						flex: 1,
						minHeight: 0,
						height: "auto",
					},
					content: {
						display: "flex",
						flexDirection: "column",
						flex: 1,
						minHeight: 0,
					},
				}}
				viewportRef={mergedRef}
			>
				{children}
			</ScrollArea>
		);
	},
);
