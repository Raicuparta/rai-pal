import { TabId } from "@api/bindings";
import { useLocalization } from "@hooks/use-localization";
import { LocalizationKey } from "@localizations/localizations";
import { Tabs, Stack, Text } from "@mantine/core";

export type Page = {
	readonly localizationKey: LocalizationKey<"tab">;
	readonly component: React.ComponentType;
	readonly icon: React.ComponentType;
};

type Props = {
	readonly id: TabId;
	readonly page: Page;
	readonly label?: string;
};

export function PageTab(props: Props) {
	const t = useLocalization("tab");

	return (
		<Tabs.Tab
			value={props.id}
			leftSection={<props.page.icon />}
		>
			<Stack
				gap={0}
				align="center"
			>
				<span>{t(props.page.localizationKey)}</span>
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
