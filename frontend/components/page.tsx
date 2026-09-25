import { Button, Card, CardProps, Group, Stack } from "@mantine/core";
import { useHotkeys } from "@mantine/hooks";
import { useLocalization } from "@hooks/use-localization";
import { IconArrowLeft } from "@tabler/icons-react";
import { Sidebar } from "@components/sidebar";

interface Props extends CardProps {
	readonly onClose?: () => void;
	readonly sidebar?: React.ReactNode;
	// The content area reserves a scrollbar by default, so that navigation
	// between pages never shifts the layout. Pages whose content scrolls
	// itself (like the games table) can opt out with "hidden".
	readonly contentOverflow?: "auto" | "scroll" | "hidden";
}

export function Page({
	onClose,
	sidebar,
	contentOverflow = "scroll",
	children,
	...props
}: Props) {
	const { t } = useLocalization("subPage");

	useHotkeys([["Escape", () => onClose?.()]]);

	return (
		<Card
			p={0}
			flex={1}
			bg="dark"
			{...props}
		>
			<Group
				flex={1}
				mih={0}
				wrap="nowrap"
				align="stretch"
				gap={0}
			>
				<Sidebar>
					{onClose && (
						<Button
							onClick={onClose}
							leftSection={<IconArrowLeft />}
						>
							{t("back")}
						</Button>
					)}
					{sidebar}
				</Sidebar>
				<Stack
					flex={1}
					mih={0}
					gap={0}
					style={{ overflowY: contentOverflow }}
				>
					{children}
				</Stack>
			</Group>
		</Card>
	);
}
