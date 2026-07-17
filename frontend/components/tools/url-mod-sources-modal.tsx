import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { ConfirmModSourceModal } from "@components/tools/confirm-mod-source-modal";
import { useLocalization } from "@hooks/use-localization";
import {
	ActionIcon,
	Group,
	Modal,
	Stack,
	Text,
	TextInput,
} from "@mantine/core";
import { IconPlus, IconTrash } from "@tabler/icons-react";
import { useEffect, useState } from "react";

type Props = {
	readonly isOpen: boolean;
	readonly onClose: () => void;
};

export function UrlModSourcesModal(props: Props) {
	const t = useLocalization("urlModSources");
	const [sources, setSources] = useState<string[]>([]);
	const [newUrl, setNewUrl] = useState("");
	const [pendingSourceUrl, setPendingSourceUrl] = useState<string | null>(null);

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
		const trimmed = newUrl.trim();
		if (!trimmed) throw new Error("URL is empty");

		await commands.getModsFromUrlModSource(trimmed);
		setPendingSourceUrl(trimmed);
	};

	const handleConfirmClose = () => {
		setPendingSourceUrl(null);
	};

	const handleConfirmSaved = () => {
		setPendingSourceUrl(null);
		setNewUrl("");
		props.onClose();
	};

	const handleRemove = async (url: string) => {
		await commands.removeUrlModSource(url);
		commands.refreshMods();
		await loadSources();
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
					{sources.length > 0 && (
						<Stack gap="xs">
							{sources.map((url) => (
								<Group
									key={url}
									gap="xs"
									wrap="nowrap"
								>
									<span
										style={{
											flex: 1,
											wordBreak: "break-all",
											fontSize: "0.875rem",
										}}
									>
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

					<Group
						gap="xs"
						align="flex-end"
					>
						<TextInput
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

					<Text
						size="sm"
						c="dimmed"
					>
						{t("addSourceDescription", {
							deepLink: "rai-pal://add-mod-source?url=[URL]",
						})}
					</Text>
				</Stack>
			</Modal>
			<ConfirmModSourceModal
				url={pendingSourceUrl ?? ""}
				isOpen={!!pendingSourceUrl}
				onClose={handleConfirmClose}
				onSaved={handleConfirmSaved}
			/>
		</>
	);
}
