import { Game, ProviderCommandAction, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { TranslationKey, useGetTranslated } from "@hooks/use-translations";
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

const providerCommandTranslationKey: Record<
	ProviderCommandAction,
	TranslationKey<"providerCommand">
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
	const t = useGetTranslated("providerCommand");
	const IconComponent =
		providerCommandActionIcon[props.action] ?? IconDeviceGamepad;

	return (
		<CommandButton
			leftSection={<IconComponent />}
			onClick={() => commands.runProviderCommand(props.game, props.action)}
		>
			{t(providerCommandTranslationKey[props.action])}
		</CommandButton>
	);
}
