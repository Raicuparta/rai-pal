import { Button, Stack, Tooltip } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { RefreshButton } from "@components/refresh-button";
import { commands } from "@api/bindings";
import { IconFolderCog, IconWorld } from "@tabler/icons-react";
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
	const [
		isUrlModSourcesModalOpen,
		{ open: openUrlModSourcesModal, close: closeUrlModSourcesModal },
	] = useDisclosure(false);

	const mods = useAtomValue(modsAtom);

	return (
		<Stack h="100%">
			<Page
				sidebar={
					<Stack
						pr="xs"
						gap="lg"
					>
						<RefreshButton />
						<Tooltip label={t("openLoadlModsFolderTooltip")}>
							<Button
								onClick={commands.openLocalModsFolder}
								leftSection={<IconFolderCog />}
							>
								{t("openLocalModsFolderButton")}
							</Button>
						</Tooltip>
						<Button
							onClick={openUrlModSourcesModal}
							leftSection={<IconWorld />}
						>
							{urlModSourcesT("title")}
						</Button>
					</Stack>
				}
			>
				<UrlModSourcesModal
					isOpen={isUrlModSourcesModalOpen}
					onClose={closeUrlModSourcesModal}
				/>
				<TableContainer>
					<ModsTable mods={mods} />
				</TableContainer>
			</Page>
		</Stack>
	);
}
