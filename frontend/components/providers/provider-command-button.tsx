import {
	OwnedGame,
	ProviderCommandAction,
	runProviderCommand,
} from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	Icon,
	IconDeviceGamepad,
	IconBooks,
	IconBrowser,
	IconDownload,
	IconPlayerPlay,
} from "@tabler/icons-react";

type Props = {
	readonly game: OwnedGame;
	readonly action: ProviderCommandAction;
};

const providerCommandActionName: Record<ProviderCommandAction, string> = {
	Install: "Install",
	ShowInLibrary: "Show In Library",
	ShowInStore: "Open Store Page",
	Start: "Start Game",
};

const providerCommandActionIcon: Record<ProviderCommandAction, Icon> = {
	Install: IconDownload,
	ShowInLibrary: IconBooks,
	ShowInStore: IconBrowser,
	Start: IconPlayerPlay,
};

export function ProviderCommandButton(props: Props) {
	const IconComponent =
		providerCommandActionIcon[props.action] ?? IconDeviceGamepad;

	return (
		<CommandButton
			leftSection={<IconComponent />}
			onClick={() => runProviderCommand(props.game.id, props.action)}
		>
			{providerCommandActionName[props.action]}
		</CommandButton>
	);
}
