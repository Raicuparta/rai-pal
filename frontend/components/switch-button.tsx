import { Button, ButtonProps, Switch } from "@mantine/core";

interface Props extends ButtonProps {
	readonly value: boolean;
	readonly onChange: (value: boolean) => void;
}

export function SwitchButton({ value, onChange, ...props }: Props) {
	return (
		<Button
			color={value ? "violet" : "dark"}
			justify="start"
			leftSection={
				<Switch
					checked={value}
					style={{ pointerEvents: "none" }}
				/>
			}
			onClick={() => onChange(!value)}
			variant={value ? "light" : "filled"}
			{...props}
		/>
	);
}
