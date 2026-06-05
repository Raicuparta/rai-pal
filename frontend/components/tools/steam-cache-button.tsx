import { useLocalization } from "@hooks/use-localization";
import { Menu } from "@mantine/core";
import { IconDots, IconSteam } from "@tabler/icons-react";

type Props = {
	readonly onClick: () => void;
};

export function SteamCacheButton(props: Props) {
	const t = useLocalization("steamCache");

	return (
		<Menu.Item
			onClick={() => props.onClick()}
			leftSection={<IconSteam />}
			rightSection={<IconDots />}
		>
			{t("resetSteamCacheButton")}
		</Menu.Item>
	);
}
