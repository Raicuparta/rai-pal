import { Button, Card, Group, Stack } from "@mantine/core";
import { useMemo, useState } from "react";
import { RefreshButton } from "@components/refresh-button";
import { commands } from "@api/bindings";
import { IconFolderCog } from "@tabler/icons-react";
import { ModModal } from "./mod-modal";
import { useUnifiedMods } from "@hooks/use-unified-mods";
import { useLocalization } from "@hooks/use-localization";
import { ModsTable } from "./mods-table";
import { TableContainer } from "@components/table/table-container";

export function ModsPage() {
	const t = useLocalization("modsPage");
	const [selectedModId, setSelectedId] = useState<string>();

	const mods = useUnifiedMods();
	const filteredMods = useMemo(
		() =>
			Object.values(mods).filter((mod) => {
				if (!mod.local && mod.remote?.deprecated) {
					return false;
				}

				return true;
			}),
		[mods],
	);

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
								mods={filteredMods}
								onClick={(mod) => setSelectedId(mod.merged.id)}
							/>
						</TableContainer>
					</Card>
				</>
			)}
		</Stack>
	);
}
