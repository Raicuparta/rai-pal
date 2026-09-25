import { Button, Card, CardProps, Group, Stack } from "@mantine/core";
import { useHotkeys } from "@mantine/hooks";
import { useLocalization } from "@hooks/use-localization";
import { IconArrowLeft } from "@tabler/icons-react";
import { Sidebar } from "@components/sidebar";
import { createContext, useContext, useState } from "react";

const PageScrollContext = createContext<HTMLElement | null>(null);

// Lets content that manages its own scrolling (like the virtualized games
// table) reuse the page's single scroll container instead of nesting another.
export function usePageScrollElement() {
	return useContext(PageScrollContext);
}

interface Props extends CardProps {
	readonly onClose?: () => void;
	readonly sidebar?: React.ReactNode;
}

export function Page({ onClose, sidebar, children, ...props }: Props) {
	const { t } = useLocalization("subPage");
	const [scrollElement, setScrollElement] = useState<HTMLElement | null>(null);

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
					ref={setScrollElement}
					flex={1}
					mih={0}
					gap={0}
					style={{ overflowY: "scroll" }}
				>
					<PageScrollContext.Provider value={scrollElement}>
						{children}
					</PageScrollContext.Provider>
				</Stack>
			</Group>
		</Card>
	);
}
