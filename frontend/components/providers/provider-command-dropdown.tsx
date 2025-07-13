import { DbGame, ProviderCommand, ProviderCommandAction } from "@api/bindings";
import { CommandDropdown } from "@components/command-dropdown";
import { ProviderIcon } from "@components/providers/provider-icon";
import { ProviderCommandButton } from "./provider-command-button";
import { Button } from "@mantine/core";

type Props = {
	readonly game: DbGame;
};

export function ProviderCommandButtons(props: Props) {
	const {
		StartViaProvider: startViaProvider,
		StartViaExe: startViaExe,
		Install: install,
		...otherProviderCommands
	} = props.game.providerCommands;

	const providerCommandEntries = Object.entries(otherProviderCommands) as [
		ProviderCommandAction,
		ProviderCommand,
	][];

	const primaryStartCommand = startViaProvider ?? startViaExe;
	const secondaryStartCommand =
		primaryStartCommand === startViaExe ? null : startViaExe;

	return (
		<>
			{primaryStartCommand && (
				<Button.Group>
					{primaryStartCommand && (
						<ProviderCommandButton
							game={props.game}
							action={startViaProvider ? "StartViaProvider" : "StartViaExe"}
							command={primaryStartCommand}
						/>
					)}

					{secondaryStartCommand && (
						<CommandDropdown>
							<ProviderCommandButton
								game={props.game}
								action="StartViaExe"
								command={secondaryStartCommand}
							/>
						</CommandDropdown>
					)}
				</Button.Group>
			)}
			{providerCommandEntries.length > 0 && (
				<CommandDropdown
					label={props.game.providerId}
					icon={<ProviderIcon providerId={props.game.providerId} />}
				>
					{providerCommandEntries.map(([action, command]) => (
						<ProviderCommandButton
							key={action}
							game={props.game}
							action={action}
							command={command}
						/>
					))}
					{props.game.exePath && install && (
						<ProviderCommandButton
							game={props.game}
							action="Install"
							command={install}
						/>
					)}
				</CommandDropdown>
			)}
		</>
	);
}
