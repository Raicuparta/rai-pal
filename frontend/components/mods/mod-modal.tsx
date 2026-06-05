import { Box, Stack } from "@mantine/core";
import { commands, GameMod } from "@api/bindings";
import { DebugData } from "@components/debug-data";
import { TableContainer } from "@components/table/table-container";
import { useMemo } from "react";
import { ModsTable } from "./mods-table";
import { SubPage } from "@components/sub-page";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { IconPlayerPlay } from "@tabler/icons-react";

type Props = {
	readonly mod: GameMod;
	readonly onClose: () => void;
};

export function ModModal(props: Props) {
	const t = useLocalization("modModal");
	const wrappedMod = useMemo(
		() => ({ [props.mod.id]: props.mod }),
		[props.mod],
	);

	return (
		<SubPage onClose={props.onClose}>
			<Box>
				<TableContainer>
					<ModsTable
						mods={wrappedMod}
						onClick={props.onClose}
					/>
				</TableContainer>
			</Box>
			<Stack
				p="xs"
				gap="xl"
			>
				<Stack>
					<DebugData data={props.mod} />
					{props.mod.runStandalone && (
						<CommandButton
							leftSection={<IconPlayerPlay />}
							onClick={async () => {
								await commands.runMod(props.mod.id, null, null);

								commands.sendAnalyticsEvent("run_mod_standalone", {
									game: null,
									param: props.mod.id,
								});
							}}
						>
							{t("runMod")}
						</CommandButton>
					)}
				</Stack>
			</Stack>
		</SubPage>
	);
}
