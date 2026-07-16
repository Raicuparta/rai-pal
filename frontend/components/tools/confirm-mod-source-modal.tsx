import { commands, GameMod } from "@api/bindings";
import { showAppNotification } from "@components/app-notifications";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import {
	Button,
	Group,
	Modal,
	Stack,
	Text,
	ScrollArea,
} from "@mantine/core";
import { IconPlus } from "@tabler/icons-react";
import { useEffect, useState } from "react";

type Props = {
	readonly url: string;
	readonly isOpen: boolean;
	readonly onClose: () => void;
	readonly onSaved: () => void;
};

export function ConfirmModSourceModal(props: Props) {
	const t = useLocalization("urlModSources");
	const [mods, setMods] = useState<GameMod[]>();
	const [isLoading, setIsLoading] = useState(false);
	const [isSaving, setIsSaving] = useState(false);
	const [error, setError] = useState<string>();

	useEffect(() => {
		if (!props.isOpen) return;

		setMods(undefined);
		setError(undefined);
		setIsLoading(true);

		commands
			.getModsFromUrlModSource(props.url)
			.then((result) => setMods(result))
			.catch((err) => setError(String(err)))
			.finally(() => setIsLoading(false));
	}, [props.isOpen, props.url]);

	const handleConfirm = async () => {
		setIsSaving(true);
		try {
			await commands.addUrlModSource(props.url);
			props.onSaved();
			props.onClose();
		} catch (error) {
			showAppNotification(
				`Failed to add mod source: ${String(error)}`,
				"error",
			);
		} finally {
			setIsSaving(false);
		}
	};

	return (
		<Modal
			centered
			opened={props.isOpen}
			onClose={props.onClose}
			title={t("confirmModalTitle")}
		>
			<Stack>
				<Text
					style={{ wordBreak: "break-all" }}
					size="sm"
					opacity={0.7}
				>
					{props.url}
				</Text>

				{isLoading && <Text>{t("loading")}</Text>}
				{error && (
					<Text c="red" size="sm">
						{error}
					</Text>
				)}
				{mods && (
					<>
						<Text size="sm">
							{t("modsFound", { count: String(mods.length) })}
						</Text>
						<ScrollArea.Autosize mah={300}>
							<Stack gap="xs">
								{mods.map((mod) => (
									<Group
										key={mod.id}
										gap="xs"
										wrap="nowrap"
									>
										<Text
											size="sm"
											style={{ flex: 1 }}
											lineClamp={1}
										>
											{mod.title}
										</Text>
										<Text
											size="xs"
											opacity={0.6}
											lineClamp={1}
										>
											{mod.author}
										</Text>
									</Group>
								))}
							</Stack>
						</ScrollArea.Autosize>
					</>
				)}

				<Group justify="end" gap="xs">
					<Button
						variant="default"
						onClick={props.onClose}
						disabled={isSaving}
					>
						{t("cancel")}
					</Button>
					<CommandButton
						onClick={handleConfirm}
						loading={isSaving}
						disabled={!mods || isLoading || !!error}
						leftSection={<IconPlus size={16} />}
					>
						{t("addSource")}
					</CommandButton>
				</Group>
			</Stack>
		</Modal>
	);
}
