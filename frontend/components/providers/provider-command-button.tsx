import { DbGame, ProviderCommandAction, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { LocalizationKey } from "@localizations/localizations";
import { ButtonProps } from "@mantine/core";
import {
	Icon,
	IconDeviceGamepad,
	IconBooks,
	IconBrowser,
	IconDownload,
	IconPlayerPlay,
	IconExternalLink,
} from "@tabler/icons-react";

interface Props extends ButtonProps {
	readonly game: DbGame;
	readonly action: ProviderCommandAction;
}

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

export function ProviderCommandButton({ action, game, ...props }: Props) {
	const { t } = useLocalization("providerCommand");
	const IconComponent = providerCommandActionIcon[action] ?? IconDeviceGamepad;

	return (
		<CommandButton
			leftSection={<IconComponent />}
			onClick={async () => {
				await commands.runProviderCommand(game.providerId, game.gameId, action);

				commands.sendAnalyticsEvent("ProviderCommand", {
					action: action,
					game: game.displayTitle,
				});
			}}
			{...props}
		>
			{t(providerCommandLocalizationKey[action])}
		</CommandButton>
	);
}
