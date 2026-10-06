import { Stack, StackProps } from "@mantine/core";
import { ScrollAreaFill } from "@components/scroll-area-fill";

export function Sidebar(props: StackProps) {
	return (
		<ScrollAreaFill
			flex="0 0 auto"
			w={250}
			style={{
				borderRight: "solid 1px var(--mantine-color-dark-4)",
			}}
		>
			<Stack
				flex={1}
				mih={0}
				gap="lg"
				py="xs"
				{...props}
			/>
		</ScrollAreaFill>
	);
}
