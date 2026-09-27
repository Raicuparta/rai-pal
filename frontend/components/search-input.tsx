import { useLocalization } from "@hooks/use-localization";
import { CloseButton, Input } from "@mantine/core";
import { useDebouncedCallback } from "@mantine/hooks";
import { IconSearch } from "@tabler/icons-react";
import { useState } from "react";

type Props = {
	readonly onChange: (search: string) => void;
	readonly value: string;
};

export function SearchInput(props: Props) {
	const { t } = useLocalization("filterMenu");

	const [prevValueProp, setPrevValueProp] = useState(props.value);
	const [innerValue, setInnerValue] = useState(props.value);

	if (props.value !== prevValueProp) {
		setPrevValueProp(props.value);
		setInnerValue(props.value);
	}

	const debouncedOnChange = useDebouncedCallback(props.onChange, 200);

	const setValue = (value: string) => {
		setInnerValue(value);
		debouncedOnChange(value);
	};

	return (
		<Input
			mx="xs"
			onChange={(event) => {
				setValue(event.currentTarget.value);
			}}
			placeholder={t("searchPlaceholder")}
			value={innerValue}
			rightSectionPointerEvents="all"
			rightSection={
				innerValue ? (
					<CloseButton onClick={() => setValue("")} />
				) : (
					<IconSearch size={16} />
				)
			}
		/>
	);
}
