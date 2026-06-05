import { useLocalization } from "@hooks/use-localization";
import { Menu } from "@mantine/core";
import { IconCat, IconDots } from "@tabler/icons-react";

type Props = {
	readonly onClick: () => void;
};

export function GlobalWineOverridesButton(props: Props) {
	const t = useLocalization("globalWineOverrides");

	return (
		<Menu.Item
			onClick={props.onClick}
			leftSection={<IconCat />}
			rightSection={<IconDots />}
		>
			{t("setUpEnvironmentButton")}
		</Menu.Item>
	);
}
