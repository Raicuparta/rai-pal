import { Button, Card, Group, Stack, Tooltip } from "@mantine/core";
import { useMemo, useState } from "react";
import { useDisclosure } from "@mantine/hooks";
import { RefreshButton } from "@components/refresh-button";
import { commands } from "@api/bindings";
import { IconFolderCog, IconWorld } from "@tabler/icons-react";
import { ModModal } from "./mod-modal";
import { useLocalization } from "@hooks/use-localization";
import { ModsTable } from "./mods-table";
import { TableContainer } from "@components/table/table-container";
import { useAtomValue } from "jotai";
import { modsAtom } from "@hooks/use-data";
import { UrlModSourcesModal } from "@components/tools/url-mod-sources-modal";

export function ModsPage() {
	const t = useLocalization("modsPage");
	const urlModSourcesT = useLocalization("urlModSources");
	const [selectedModId, setSelectedId] = useState<string>();
	const [
		isUrlModSourcesModalOpen,
		{ open: openUrlModSourcesModal, close: closeUrlModSourcesModal },
	] = useDisclosure(false);

	const mods = useAtomValue(modsAtom);

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
							onClick={openUrlModSourcesModal}
							leftSection={<IconWorld />}
						>
							{urlModSourcesT("title")}
						</Button>
						<Tooltip label={t("openLoadlModsFolderTooltip")}>
							<Button
								onClick={commands.openLocalModsFolder}
								leftSection={<IconFolderCog />}
							>
								{t("openLocalModsFolderButton")}
							</Button>
						</Tooltip>
						<RefreshButton />
					</Group>
					<UrlModSourcesModal
						isOpen={isUrlModSourcesModalOpen}
						onClose={closeUrlModSourcesModal}
					/>
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
