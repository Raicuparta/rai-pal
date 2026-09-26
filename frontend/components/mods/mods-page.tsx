import { Button, Stack, Tooltip } from "@mantine/core";
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
import { Page } from "@components/page";

export function ModsPage() {
	const { t } = useLocalization("modsPage");
	const { t: urlModSourcesT } = useLocalization("urlModSources");
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
				<Page
					sidebar={
						<Stack p="xs">
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
						</Stack>
					}
				>
					<UrlModSourcesModal
						isOpen={isUrlModSourcesModalOpen}
						onClose={closeUrlModSourcesModal}
					/>
					<TableContainer>
						<ModsTable
							mods={mods}
							onClick={(mod) => setSelectedId(mod.id)}
						/>
					</TableContainer>
				</Page>
			)}
		</Stack>
	);
}
