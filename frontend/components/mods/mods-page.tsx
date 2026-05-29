import { Button, Card, Group, Stack } from "@mantine/core";
import { useMemo, useState } from "react";
import { RefreshButton } from "@components/refresh-button";
import { commands } from "@api/bindings";
import { IconFolderCog } from "@tabler/icons-react";
import { ModModal } from "./mod-modal";
import { useLocalization } from "@hooks/use-localization";
import { ModsTable } from "./mods-table";
import { TableContainer } from "@components/table/table-container";
import { useAtomValue } from "jotai";
import { remoteModsAtom } from "@hooks/use-data";

export function ModsPage() {
	const t = useLocalization("modsPage");
	const [selectedModId, setSelectedId] = useState<string>();

	const mods = useAtomValue(remoteModsAtom);

	const selectedMod = useMemo(() => {
		const result = selectedModId ? mods[selectedModId] : undefined;

		return result;
	}, [selectedModId, mods]);

	return (
		<Stack h="100%">
			{selectedMod && (
				<ModModal
					onClose={() => setSelectedId(undefined)}
					mod={selectedMod}
				/>
			)}
			{!selectedMod && (
				<>
					<Group justify="end">
						<Button
							onClick={commands.openModsFolder}
							leftSection={<IconFolderCog />}
						>
							{t("openModsFolderButton")}
						</Button>
						<RefreshButton />
					</Group>
					<Card
						p={0}
						flex={1}
						bg="dark"
					>
						<TableContainer style={{ overflowY: "scroll" }}>
							<ModsTable
								mods={mods}
								onClick={(mod) => setSelectedId(mod.id)}
							/>
						</TableContainer>
					</Card>
				</>
			)}
		</Stack>
	);
}
