import { useLocalization } from "@hooks/use-localization";
import { Menu } from "@mantine/core";
import { IconCat, IconDots } from "@tabler/icons-react";

type Props = {
	onClick: () => void;
};

export function WineBepInExEnvironmentButton(props: Props) {
	const t = useLocalization("wineBepInExEnvironment");

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
