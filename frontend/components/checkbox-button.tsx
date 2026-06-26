import {
	ActionIcon,
	Button,
	ButtonProps,
	Checkbox,
	Group,
	Tooltip,
} from "@mantine/core";
import { forwardRef } from "react";
import styles from "./components.module.css";
import { IconCheck, IconCircleDotFilled } from "@tabler/icons-react";

interface Props extends ButtonProps {
	readonly checked: boolean;
	readonly tooltip?: string;
	readonly onChange: (checked: boolean) => void;
	readonly onExclusiveClick: () => void;
}

function CheckboxButtonInternal(
	{ checked, onChange, tooltip, onExclusiveClick, children, ...props }: Props,
	ref: React.ForwardedRef<HTMLButtonElement>,
) {
	return (
		<Tooltip
			label={tooltip}
			disabled={!tooltip}
		>
			<Group gap={0}>
				<ActionIcon
					size="sm"
					variant={checked ? "filled" : "default"}
					onClick={onChange ? () => onChange(!checked) : undefined}
				>
					{checked && <IconCheck />}
				</ActionIcon>
				<Button
					variant="subtle"
					color="gray"
					ref={ref}
					justify="start"
					flex={1}
					px="xs"
					// leftSection={
					// 	<Checkbox
					// 		className={styles.buttonCheckbox}
					// 		tabIndex={-1}
					// 		readOnly
					// 		checked={checked}
					// 	/>
					// }
					onClick={(e) => {
						e.stopPropagation();
						onExclusiveClick();
					}}
					{...props}
				>
					{children}

					{tooltip && " *"}
				</Button>
			</Group>
		</Tooltip>
	);
}

export const CheckboxButton = forwardRef(CheckboxButtonInternal);
