import { Stack, StackProps } from "@mantine/core";

export function Sidebar(props: StackProps) {
	return (
		<Stack
			flex="0 0 auto"
			w={250}
			style={{ overflowY: "auto" }}
			{...props}
		/>
	);
}
