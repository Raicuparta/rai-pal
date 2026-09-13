import { useLocalization } from "@hooks/use-localization";
import { Menu } from "@mantine/core";
import { IconBrandSteam, IconDots } from "@tabler/icons-react";

type Props = {
	readonly onClick: () => void;
};

export function SteamShortcutButton(props: Props) {
	const { t } = useLocalization("steamShortcut");

	return (
		<Menu.Item
			onClick={() => props.onClick()}
			leftSection={<IconBrandSteam />}
			rightSection={<IconDots />}
		>
			{t("addRaiPalSteamShortcutButton")}
		</Menu.Item>
	);
}
