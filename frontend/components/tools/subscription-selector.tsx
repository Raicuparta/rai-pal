import { GameSubscription } from "@api/bindings";
import { useAppSettings } from "@hooks/use-app-settings";
import { Button } from "@mantine/core";
import { useAtomValue } from "jotai";
import { useLocalization } from "@hooks/use-localization";
import { CheckboxButton } from "@components/checkbox-button";
import { useUpdateData } from "@hooks/use-update-data";
import { loadingTasksAtom } from "@hooks/use-data";
import { useState } from "react";

const subscriptions: GameSubscription[] = [
	"XboxGamePass",
	"EaPlay",
	"UbisoftPremium",
	"UbisoftClassics",
];

export function SubscriptionSelector() {
	const t = useLocalization("appDropdownMenu");
	const [settings, setSettings] = useAppSettings();
	const updateAppData = useUpdateData();
	const loadingTasks = useAtomValue(loadingTasksAtom);
	const [isDirty, setIsDirty] = useState<boolean>(false);

	return (
		<Button.Group orientation="vertical">
			{subscriptions.map((subscription) => (
				<CheckboxButton
					checked={settings.ownedSubscriptions.includes(subscription)}
					key={subscription}
					onChange={(checked) => {
						const newSubscriptions = checked
							? [...settings.ownedSubscriptions, subscription]
							: settings.ownedSubscriptions.filter((s) => s !== subscription);
						setSettings({
							...settings,
							ownedSubscriptions: newSubscriptions,
						}).finally(() => setIsDirty(true));
					}}
				>
					{subscription}
				</CheckboxButton>
			))}
			<Button
				onClick={() => {
					updateAppData().finally(() => {
						setIsDirty(false);
					});
				}}
				disabled={!isDirty || loadingTasks.length > 0}
			>
				{t("applyChangesToSubscriptions")}
			</Button>
		</Button.Group>
	);
}
