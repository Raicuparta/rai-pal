import { Menu, MenuItemProps, Switch } from "@mantine/core";

interface Props extends MenuItemProps {
	readonly value: boolean;
	readonly onChange: (value: boolean) => void;
}

export function SwitchButton({ value, onChange, ...props }: Props) {
	return (
		<Menu.Item
			closeMenuOnClick={false}
			leftSection={
				<Switch
					checked={value}
					style={{ pointerEvents: "none" }}
				/>
			}
			onClick={() => onChange(!value)}
			{...props}
		/>
	);
}
