import { DbGame, ProviderCommand, ProviderCommandAction } from "@api/bindings";
import { CommandDropdown } from "@components/command-dropdown";
import { ProviderIcon } from "@components/providers/provider-icon";
import { ProviderCommandButton } from "./provider-command-button";

type Props = {
	readonly game: DbGame;
};

export function ProviderCommandButtons(props: Props) {
	let providerCommands = Object.entries(props.game.providerCommands) as [
		ProviderCommandAction,
		ProviderCommand,
	][];
	if (props.game.exePath) {
		providerCommands = providerCommands.filter(
			([action]) => (action as ProviderCommandAction) != "Install",
		);
	}

	if (providerCommands.length == 0) return null;

	return (
		<CommandDropdown
			label={props.game.providerId}
			icon={<ProviderIcon providerId={props.game.providerId} />}
		>
			{providerCommands.map(([action, command]) => (
				<ProviderCommandButton
					key={action}
					game={props.game}
					action={action}
					command={command}
				/>
			))}
		</CommandDropdown>
	);
}
