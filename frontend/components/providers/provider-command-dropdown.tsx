import { Game, ProviderCommandAction } from "@api/bindings";
import { CommandDropdown } from "@components/command-dropdown";
import { ProviderIcon } from "@components/providers/provider-icon";
import { ProviderCommandButton } from "./provider-command-button";

type Props = {
	readonly game: Game;
	readonly isInstalled?: boolean;
};

export function ProviderCommandButtons(props: Props) {
	let providerCommandActions = Object.keys(
		props.game.providerCommands,
	) as ProviderCommandAction[];
	if (props.isInstalled) {
		providerCommandActions = providerCommandActions.filter(
			(action) => (action as ProviderCommandAction) != "Install",
		);
	}

	if (providerCommandActions.length == 0) return null;

	return (
		<CommandDropdown
			label={props.game.providerId}
			icon={<ProviderIcon providerId={props.game.providerId} />}
		>
			{providerCommandActions.map((providerCommandAction) => (
				<ProviderCommandButton
					key={providerCommandAction}
					game={props.game}
					action={providerCommandAction}
				/>
			))}
		</CommandDropdown>
	);
}
