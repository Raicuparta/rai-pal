import { Tabs, Stack, Text } from "@mantine/core";

export type Page = {
	title: string;
	component: () => React.JSX.Element;
	icon: React.JSX.Element;
};

type Props = {
	readonly id: string;
	readonly page: Page;
	readonly label?: string;
};

export function PageTab(props: Props) {
	return (
		<Tabs.Tab
			value={props.id}
			leftSection={props.page.icon}
		>
			<Stack
				gap={0}
				align="center"
			>
				<span>{props.page.title}</span>
				{props.label && (
					<Text
						size="9px"
						opacity={0.5}
						pos="absolute"
						bottom={1.5}
					>
						{props.label}
					</Text>
				)}
			</Stack>
		</Tabs.Tab>
	);
}
