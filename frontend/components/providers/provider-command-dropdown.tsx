import { DbGame, ProviderCommandAction } from "@api/bindings";
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
		...otherProviderCommands
	} = props.game.providerCommands;

	const providerCommandActions = Object.keys(
		otherProviderCommands,
	) as ProviderCommandAction[];

	const [primaryStart, secondaryStart]: readonly ProviderCommandAction[] =
		startViaProvider
			? ["StartViaProvider", "StartViaExe"]
			: startViaExe
				? ["StartViaExe"]
				: [];

	return (
		<>
			{primaryStart && (
				<Button.Group>
					<ProviderCommandButton
						game={props.game}
						action={primaryStart}
					/>

					{secondaryStart && (
						<CommandDropdown>
							<ProviderCommandButton
								game={props.game}
								action={secondaryStart}
							/>
						</CommandDropdown>
					)}
				</Button.Group>
			)}
			{providerCommandActions.length > 0 && (
				<CommandDropdown
					label={props.game.providerId}
					icon={<ProviderIcon providerId={props.game.providerId} />}
				>
					{providerCommandActions.map((action) => (
						<ProviderCommandButton
							key={action}
							game={props.game}
							action={action}
						/>
					))}
				</CommandDropdown>
			)}
		</>
	);
}
