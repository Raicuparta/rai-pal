import { Anchor, ButtonGroup, Group, Stack } from "@mantine/core";
import { GameMod, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { DebugData } from "@components/debug-data";
import { useLocalization } from "@hooks/use-localization";
import {
	IconDownload,
	IconExternalLink,
	IconPlayerPlay,
} from "@tabler/icons-react";

type Props = {
	readonly mod: GameMod;
};

export function ModActions({ mod }: Props) {
	const { t } = useLocalization("modModal");

	return (
		<Stack align="end">
			<ButtonGroup>
				{mod.runStandalone && (
					<CommandButton
						leftSection={<IconPlayerPlay />}
						onClick={async () => {
							await commands.runMod(mod.id, null, null);

							commands.sendAnalyticsEvent("RunMod", {
								mod_id: mod.id,
							});
						}}
					>
						{t("runMod")}
					</CommandButton>
				)}
				{mod.runStandalone && mod.install && (
					<CommandButton
						leftSection={<IconDownload />}
						onClick={async () => {
							await commands.installMod(mod.id, null, null);

							commands.sendAnalyticsEvent("InstallMod", {
								mod_id: mod.id,
							});
						}}
					>
						{t("downloadMod")}
					</CommandButton>
				)}
				<DebugData
					data={mod}
					hideLabel
				/>
			</ButtonGroup>
			{mod.sourceCode && (
				<Anchor
					c="gray"
					size="sm"
					href={mod.sourceCode}
					onClick={(event) => {
						event.preventDefault();
						commands.openUrl(mod.sourceCode);
					}}
				>
					<Group gap={2}>
						{t("openModSource")}
						<IconExternalLink />
					</Group>
				</Anchor>
			)}
		</Stack>
	);
}
