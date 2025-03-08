import { Game, ProviderCommandAction, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { LocalizationKey } from "@localizations/localizations";
import {
	Icon,
	IconDeviceGamepad,
	IconBooks,
	IconBrowser,
	IconDownload,
	IconPlayerPlay,
	IconExternalLink,
} from "@tabler/icons-react";

type Props = {
	readonly game: Game;
	readonly action: ProviderCommandAction;
};

const providerCommandLocalizationKey: Record<
	ProviderCommandAction,
	LocalizationKey<"providerCommand">
> = {
	Install: "installGame",
	ShowInLibrary: "showGameInLibrary",
	ShowInStore: "showGameInStore",
	Start: "startGame",
	OpenInBrowser: "openGamePageInBrowser",
};

const providerCommandActionIcon: Record<ProviderCommandAction, Icon> = {
	Install: IconDownload,
	ShowInLibrary: IconBooks,
	ShowInStore: IconBrowser,
	Start: IconPlayerPlay,
	OpenInBrowser: IconExternalLink,
};

export function ProviderCommandButton(props: Props) {
	const t = useLocalization("providerCommand");
	const IconComponent =
		providerCommandActionIcon[props.action] ?? IconDeviceGamepad;

	return (
		<CommandButton
			leftSection={<IconComponent />}
			onClick={() => commands.runProviderCommand(props.game, props.action)}
		>
			{t(providerCommandLocalizationKey[props.action])}
		</CommandButton>
	);
}
