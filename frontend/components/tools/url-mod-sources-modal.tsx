import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { ActionIcon, Group, Modal, Stack, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconPlus, IconTrash, IconWorld } from "@tabler/icons-react";
import { useEffect, useState } from "react";
import { ConfirmModSourceModal } from "./confirm-mod-source-modal";

type Props = {
	readonly isOpen: boolean;
	readonly onClose: () => void;
};

const DEFAULT_URL = "https://raicuparta.github.io/rai-pal-db/mod-db/1/mods.json";

export function UrlModSourcesModal(props: Props) {
	const t = useLocalization("urlModSources");
	const [sources, setSources] = useState<string[]>([]);
	const [newUrl, setNewUrl] = useState("");
	const [
		isConfirmModalOpen,
		{ open: openConfirmModal, close: closeConfirmModal },
	] = useDisclosure(false);

	const loadSources = () =>
		commands
			.getUrlModSources()
			.then((result) => setSources(result.additionalUrls));

	useEffect(() => {
		if (props.isOpen) {
			loadSources();
			setNewUrl("");
		}
	}, [props.isOpen]);

	const handleAdd = async () => {
		if (!newUrl.trim()) return;
		openConfirmModal();
	};

	const handleRemove = async (url: string) => {
		await commands.removeUrlModSource(url);
		await loadSources();
	};

	const handleSourceSaved = () => {
		setNewUrl("");
		loadSources();
	};

	return (
		<>
			<Modal
				centered
				opened={props.isOpen}
				onClose={props.onClose}
				title={t("title")}
			>
				<Stack>
					<Group gap="xs">
						<IconWorld size={16} />
						<span style={{ fontWeight: 500 }}>{t("defaultSource")}</span>
					</Group>
					<span style={{ opacity: 0.7, wordBreak: "break-all" }}>{DEFAULT_URL}</span>

					{sources.length > 0 && (
						<Stack gap="xs">
							<span style={{ fontWeight: 500 }}>{t("customSources")}</span>
							{sources.map((url) => (
								<Group key={url} gap="xs" wrap="nowrap">
									<span style={{ flex: 1, wordBreak: "break-all", fontSize: "0.875rem" }}>
										{url}
									</span>
									<ActionIcon
										color="red"
										variant="subtle"
										onClick={() => handleRemove(url)}
									>
										<IconTrash size={16} />
									</ActionIcon>
								</Group>
							))}
						</Stack>
					)}

					<Group gap="xs" align="flex-end">
						<TextInput
							label={t("addCustomSource")}
							placeholder="https://example.com/mods.json"
							value={newUrl}
							onChange={(event) => setNewUrl(event.currentTarget.value)}
							onKeyDown={(event) => {
								if (event.key === "Enter") {
									handleAdd();
								}
							}}
							style={{ flex: 1 }}
						/>
						<CommandButton
							onClick={handleAdd}
							leftSection={<IconPlus size={16} />}
						>
							{t("add")}
						</CommandButton>
					</Group>
				</Stack>
			</Modal>
			<ConfirmModSourceModal
				url={newUrl.trim()}
				isOpen={isConfirmModalOpen}
				onClose={closeConfirmModal}
				onSaved={handleSourceSaved}
			/>
		</>
	);
}
