import {
	DbGame,
	ProviderCommand,
	ProviderCommandAction,
	commands,
} from "@api/bindings";
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
	readonly game: DbGame;
	readonly action: ProviderCommandAction;
	readonly command: ProviderCommand;
};

const providerCommandLocalizationKey: Record<
	ProviderCommandAction,
	LocalizationKey<"providerCommand">
> = {
	Install: "installGame",
	ShowInLibrary: "showGameInLibrary",
	ShowInStore: "showGameInStore",
	StartViaProvider: "startGameViaProvider",
	StartViaExe: "startGameViaExe",
	OpenInBrowser: "openGamePageInBrowser",
};

const providerCommandActionIcon: Record<ProviderCommandAction, Icon> = {
	Install: IconDownload,
	ShowInLibrary: IconBooks,
	ShowInStore: IconBrowser,
	StartViaProvider: IconPlayerPlay,
	StartViaExe: IconPlayerPlay,
	OpenInBrowser: IconExternalLink,
};

export function ProviderCommandButton(props: Props) {
	const t = useLocalization("providerCommand");
	const IconComponent =
		providerCommandActionIcon[props.action] ?? IconDeviceGamepad;

	return (
		<CommandButton
			leftSection={<IconComponent />}
			onClick={() => commands.runProviderCommand(props.command)}
		>
			{t(providerCommandLocalizationKey[props.action])}
		</CommandButton>
	);
}
