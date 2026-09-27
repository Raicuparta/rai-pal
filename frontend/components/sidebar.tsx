import { Stack, StackProps } from "@mantine/core";

export function Sidebar(props: StackProps) {
	return (
		<Stack
			flex="0 0 auto"
			gap="lg"
			py="xs"
			w={250}
			style={{
				overflowY: "auto",
				borderRight: "solid 1px var(--mantine-color-dark-4)",
			}}
			{...props}
		/>
	);
}
