import { GameSubscription } from "@api/bindings";
import { FilterButton } from "@components/filters/filter-button";
import { useOwnedSubscriptions } from "@hooks/use-owned-subscriptions";
import { Indicator, Button, Popover, Stack, Text } from "@mantine/core";
import { IconCash } from "@tabler/icons-react";

const subscriptions: Record<GameSubscription, string> = {
	EaPlay: "EA Play",
	XboxGamePass: "Game Pass",
	UbisoftClassics: "Ubisoft Classics",
	UbisoftPremium: "Ubisoft Premium",
} as const;

export function SubscriptionsSelector() {
  const [ownedSubscriptions, toggleSubscription] = useOwnedSubscriptions();

	return (
		<Indicator offset={8} disabled={Object.values(ownedSubscriptions).filter(Boolean).length == 0}>
			<Button.Group>
				<Popover trapFocus>
					<Popover.Target>
						<Button leftSection={<IconCash />}>Subscriptions</Button>
					</Popover.Target>
					<Popover.Dropdown
						bg="dark"
						p="xs"
					>
						<Stack>
							<Text
								maw={300}
								fz="sm"
							>
								Select the subscriptions you have. The games included in those
								subscriptions will be considered as owned in Rai Pal.
							</Text>
							{Object.entries(subscriptions).map(([subscriptionId, subscriptionTitle]) => (
								<FilterButton
									key={subscriptionId}
									filterOption={{
										label: subscriptionTitle,
										value: subscriptionId as GameSubscription,
									}}
									isHidden={!ownedSubscriptions[subscriptionId as GameSubscription] ?? false}
									isUnavailable={false}
									onClick={toggleSubscription}
								/>
							))}
						</Stack>
					</Popover.Dropdown>
				</Popover>
			</Button.Group>
		</Indicator>
	);
}
