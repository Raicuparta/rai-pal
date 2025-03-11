import { useLocalization } from "@hooks/use-localization";
import { CodeHighlight } from "@mantine/code-highlight";
import { Divider, Stack } from "@mantine/core";

type Props<TData> = {
	readonly data: TData;
};

export function DebugData<TData>({ data }: Props<TData>) {
	const t = useLocalization("debugData");
	const debugText = JSON.stringify(data, null, 2) ?? "";

	return (
		<Stack gap="xs">
			<Divider label={t("debugDataTitle")} />
			<CodeHighlight
				// Using text as key to force component to remount,
				// seems like there's some bug preventing it from updating.
				key={debugText}
				code={debugText}
				copyLabel={t("debugDataCopy")}
				language="json"
			/>
		</Stack>
	);
}
