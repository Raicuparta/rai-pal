import { TranslationKey, useGetTranslated } from "@hooks/use-translations";
import { Tabs, Stack, Text } from "@mantine/core";

export type Page = {
	readonly translationKey: TranslationKey<"tab">;
	readonly component: () => React.JSX.Element;
	readonly icon: React.JSX.Element;
};

type Props = {
	readonly page: Page;
	readonly label?: string;
};

export function PageTab(props: Props) {
	const t = useGetTranslated("tab");

	return (
		<Tabs.Tab
			value={props.page.translationKey}
			leftSection={props.page.icon}
		>
			<Stack
				gap={0}
				align="center"
			>
				<span>{t(props.page.translationKey)}</span>
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
