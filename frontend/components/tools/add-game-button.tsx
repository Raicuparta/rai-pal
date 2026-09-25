import { Menu } from "@mantine/core";
import { useLocalization } from "@hooks/use-localization";
import { IconDots, IconPlaylistAdd } from "@tabler/icons-react";

type Props = {
	readonly onClick: () => void;
};

export function AddGameButton(props: Props) {
	const { t } = useLocalization("manualGames");

	return (
		<Menu.Item
			onClick={() => props.onClick()}
			leftSection={<IconPlaylistAdd />}
			rightSection={<IconDots />}
		>
			{t("button")}
		</Menu.Item>
	);
}
