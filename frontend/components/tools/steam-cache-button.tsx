import { useLocalization } from "@hooks/use-localization";
import { Menu } from "@mantine/core";
import { IconBrandSteam, IconDots } from "@tabler/icons-react";

type Props = {
	onClick: () => void;
};

export function SteamCacheButton(props: Props) {
	const t = useLocalization("steamCache");

	return (
		<Menu.Item
			onClick={() => props.onClick()}
			leftSection={<IconBrandSteam />}
			rightSection={<IconDots />}
		>
			{t("resetSteamCacheButton")}
		</Menu.Item>
	);
}
