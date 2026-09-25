import { Button, Card, CardProps, Group } from "@mantine/core";
import { useHotkeys } from "@mantine/hooks";
import { useLocalization } from "@hooks/use-localization";
import { IconArrowLeft } from "@tabler/icons-react";
import { Sidebar } from "@components/sidebar";

interface Props extends CardProps {
	readonly onClose: () => void;
	readonly sidebar?: React.ReactNode;
}

export function SubPage({ onClose, sidebar, children, ...props }: Props) {
	const { t } = useLocalization("subPage");

	useHotkeys([["Escape", onClose]]);

	return (
		<Group
			flex={1}
			mih={0}
			wrap="nowrap"
			align="stretch"
			gap={0}
		>
			<Sidebar>
				<Button
					onClick={onClose}
					leftSection={<IconArrowLeft />}
				>
					{t("back")}
				</Button>
				{sidebar}
			</Sidebar>
			<Card
				flex={1}
				style={{ overflowY: "scroll" }}
				p={0}
				bg="dark"
				{...props}
			>
				{children}
			</Card>
		</Group>
	);
}
