import { Tabs, Stack, Text } from "@mantine/core";

export type Page = {
	title: string;
	component: () => JSX.Element;
	icon: JSX.Element;
};

type Props = {
	readonly id: string;
	readonly count: number;
	readonly page: Page;
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
				{props.count > -1 && (
					<Text
						size="9px"
						opacity={0.5}
						pos="absolute"
						bottom={1.5}
					>
						({props.count})
					</Text>
				)}
			</Stack>
		</Tabs.Tab>
	);
}
