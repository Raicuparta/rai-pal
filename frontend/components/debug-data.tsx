import { useGetTranslated } from "@hooks/use-translations";
import { CodeHighlight } from "@mantine/code-highlight";
import { Divider, Stack } from "@mantine/core";
import { useMemo } from "react";

type Props<TData> = {
	readonly data: TData;
};

export function DebugData<TData>(props: Props<TData>) {
	const t = useGetTranslated("debugData");
	const debugText = useMemo(
		() => JSON.stringify(props.data, null, 2) ?? "",
		[props.data],
	);

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
