import { Button, Card, CardProps, Group, Stack } from "@mantine/core";
import { useHotkeys } from "@mantine/hooks";
import { useLocalization } from "@hooks/use-localization";
import { IconArrowLeft } from "@tabler/icons-react";

interface Props extends CardProps {
	readonly onClose: () => void;
}

export function SubPage({ onClose, ...props }: Props) {
	const t = useLocalization("subPage");

	useHotkeys([["Escape", onClose]]);

	return (
		<>
			<Group>
				<Button
					onClick={onClose}
					leftSection={<IconArrowLeft />}
				>
					{t("back")}
				</Button>
			</Group>
			<Card
				flex={1}
				style={{ overflowY: "scroll" }}
				p={0}
				bg="dark"
				{...props}
			/>
		</>
	);
}
