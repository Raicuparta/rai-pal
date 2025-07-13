import { Button, ButtonProps, Checkbox, Tooltip } from "@mantine/core";
import { forwardRef } from "react";
import styles from "./components.module.css";

interface Props extends ButtonProps {
	readonly checked: boolean;
	readonly tooltip?: string;
	readonly onChange?: (checked: boolean) => void;
}

function CheckboxButtonInternal(
	{ checked, onChange, tooltip, children, ...props }: Props,
	ref: React.ForwardedRef<HTMLButtonElement>,
) {
	return (
		<Tooltip
			label={tooltip}
			disabled={!tooltip}
		>
			<Button
				ref={ref}
				justify="start"
				leftSection={
					<Checkbox
						className={styles.buttonCheckbox}
						tabIndex={-1}
						readOnly
						checked={checked}
					/>
				}
				onClick={onChange ? () => onChange(!checked) : undefined}
				{...props}
			>
				{children}

				{tooltip && " *"}
			</Button>
		</Tooltip>
	);
}

export const CheckboxButton = forwardRef(CheckboxButtonInternal);
