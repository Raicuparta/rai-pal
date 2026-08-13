import { commands, UrlModSource } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { ConfirmModSourceModal } from "@components/tools/confirm-mod-source-modal";
import { useLocalization } from "@hooks/use-localization";
import {
	ActionIcon,
	Code,
	Group,
	Input,
	Modal,
	Stack,
	Switch,
	Text,
	TextInput,
	Tooltip,
} from "@mantine/core";
import { IconPlus, IconTrash } from "@tabler/icons-react";
import { startTransition, useEffect, useState } from "react";

type Props = {
	readonly isOpen: boolean;
	readonly onClose: () => void;
};

export function UrlModSourcesModal(props: Props) {
	const { t, T } = useLocalization("urlModSources");
	const [sources, setSources] = useState<UrlModSource[]>([]);
	const [newUrl, setNewUrl] = useState("");
	const [pendingSourceUrl, setPendingSourceUrl] = useState<string | null>(null);
	const [isToggling, setIsToggling] = useState(false);

	const loadSources = () =>
		commands.getUrlModSources().then((result) => setSources(result.sources));

	useEffect(() => {
		if (props.isOpen) {
			loadSources();
			startTransition(() => {
				setNewUrl("");
			});
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

	const handleToggle = async (source: UrlModSource) => {
		setIsToggling(true);
		try {
			await commands.setUrlModSourceEnabled(source.url, !source.enabled);
			await commands.refreshMods();
			await loadSources();
		} finally {
			setIsToggling(false);
		}
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
				size="lg"
			>
				<Stack>
					{sources.length > 0 && (
						<Stack>
							{sources.map((source) => (
								<Group key={source.url}>
									<Switch
										checked={source.enabled}
										onChange={() => handleToggle(source)}
										disabled={isToggling}
										size="xs"
									/>
									<TextInput
										rightSection={
											<ActionIcon
												size="sm"
												color="red"
												variant="subtle"
												disabled={source.isDefault}
												onClick={() => handleRemove(source.url)}
											>
												<IconTrash />
											</ActionIcon>
										}
										readOnly
										value={source.url}
										flex={1}
									/>
								</Group>
							))}
						</Stack>
					)}

					<Group>
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
						<T
							path="addSourceDescription"
							params={{
								deepLink: (
									<Text
										component={Code}
										textWrap="nowrap"
									>
										rai-pal://add-mod-source?url=[URL]
									</Text>
								),
							}}
						/>
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
