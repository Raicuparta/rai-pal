import { Box, Button, Card, CardProps, Group, Stack } from "@mantine/core";
import { useHotkeys } from "@mantine/hooks";
import { useLocalization } from "@hooks/use-localization";
import { IconArrowLeft } from "@tabler/icons-react";
import { Sidebar } from "@components/sidebar";
import { ScrollAreaFill } from "@components/scroll-area-fill";

interface Props extends CardProps {
	readonly onClose?: () => void;
	readonly sidebar?: React.ReactNode;
	// Pages whose content scrolls itself (like the games table) can opt out.
	readonly scrollable?: boolean;
}

function PageContent({
	scrollable,
	children,
}: {
	readonly scrollable: boolean;
	readonly children: React.ReactNode;
}) {
	if (!scrollable) {
		return (
			<Stack
				flex={1}
				mih={0}
				gap={0}
			>
				{children}
			</Stack>
		);
	}

	return (
		<ScrollAreaFill
			flex={1}
			mih={0}
		>
			<Stack
				flex={1}
				mih={0}
				gap={0}
			>
				{children}
			</Stack>
		</ScrollAreaFill>
	);
}

export function Page({
	onClose,
	sidebar,
	scrollable = true,
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
						<Box px="xs">
							<Button
								onClick={onClose}
								leftSection={<IconArrowLeft />}
							>
								{t("back")}
							</Button>
						</Box>
					)}
					{sidebar}
				</Sidebar>
				<PageContent scrollable={scrollable}>{children}</PageContent>
			</Group>
		</Card>
	);
}
