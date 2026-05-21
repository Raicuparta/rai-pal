import { useLocalization } from "@hooks/use-localization";
import {
	ActionIcon,
	Box,
	Card,
	CopyButton,
	Divider,
	Stack,
	Tooltip,
} from "@mantine/core";
import { IconCheck, IconCopy } from "@tabler/icons-react";

type Props<TData> = {
	readonly data: TData;
};

export function DebugData<TData>({ data }: Props<TData>) {
	const t = useLocalization("debugData");
	const debugText = JSON.stringify(data, null, 2) ?? "";

	return (
		<Stack gap="xs">
			<Divider label={t("debugDataTitle")} />
			<Card
				component="pre"
				pos="relative"
				fz="xs"
			>
				<Box
					pos="absolute"
					right={10}
					top={10}
				>
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
				</Box>
				<Box
					component="code"
					style={{ overflowX: "auto", overflowY: "hidden" }}
				>
					{debugText}
				</Box>
			</Card>
		</Stack>
	);
}
