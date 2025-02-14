import { css, cx } from "@styled-system/css";
import { PropsStylableWithChildren } from "@util/style-types";

export function TableContainer({
	className,
	...props
}: PropsStylableWithChildren) {
	return (
		<div
			className={cx(
				css({
					flex: 1,
					borderRadius: "md",
					overflow: "hidden",
					backgroundColor: "dark.700",
				}),
				className,
			)}
			{...props}
		/>
	);
}
