import { DbGame, ProviderCommand, ProviderCommandAction } from "@api/bindings";
import { CommandDropdown } from "@components/command-dropdown";
import { ProviderIcon } from "@components/providers/provider-icon";
import { ProviderCommandButton } from "./provider-command-button";
import { Button } from "@mantine/core";

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

	const startViaProvider = props.game.providerCommands["StartViaProvider"];
	const startViaExe = props.game.providerCommands["StartViaExe"];
	const primaryStartCommand = startViaProvider ?? startViaExe;

	return (
		primaryStartCommand && (
			<>
				<Button.Group>
					{primaryStartCommand && (
						<ProviderCommandButton
							game={props.game}
							action={startViaProvider ? "StartViaProvider" : "StartViaExe"}
							command={primaryStartCommand}
						/>
					)}

					{startViaExe && (
						<CommandDropdown>
							<ProviderCommandButton
								game={props.game}
								action="StartViaExe"
								command={startViaExe}
							/>
						</CommandDropdown>
					)}
				</Button.Group>
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
			</>
		)
	);
}
