import { useLocalization } from "@hooks/use-localization";
import {
	ActionIcon,
	Box,
	Button,
	Card,
	CopyButton,
	Divider,
	Group,
	Modal,
	Stack,
	Tooltip,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconBug, IconCheck, IconCopy } from "@tabler/icons-react";

type Props<TData> = {
	readonly data: TData;
};

export function DebugData<TData>({ data }: Props<TData>) {
	const t = useLocalization("debugData");
	const debugText = JSON.stringify(data, null, 2) ?? "";
	const [opened, { open, close }] = useDisclosure(false);

	return (
		<>
			<Button
				leftSection={<IconBug />}
				onClick={open}
			>
				{t("debugDataTitle")}
			</Button>
			<Modal
				size="100%"
				opened={opened}
				onClose={close}
				title={
					<Group>
						<span>{t("debugDataTitle")}</span>
						<CopyButton
							value={debugText}
							timeout={2000}
						>
							{({ copied, copy }) => (
								<Tooltip
									label={t("debugDataCopy")}
									withArrow
									position="right"
								>
									<ActionIcon
										color={copied ? "green" : "gray"}
										variant="subtle"
										onClick={copy}
									>
										{copied ? <IconCheck size={16} /> : <IconCopy size={16} />}
									</ActionIcon>
								</Tooltip>
							)}
						</CopyButton>
					</Group>
				}
			>
				<Box
					component="pre"
					fz="xs"
				>
					<Box
						component="code"
						style={{ overflowX: "auto", overflowY: "hidden" }}
					>
						{debugText}
					</Box>
				</Box>
			</Modal>
		</>
	);
}
