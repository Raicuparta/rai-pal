import { GameSubscription } from "@api/bindings";
import { usePersistedState } from "./use-persisted-state";
import { useCallback } from "react";

// TODO: this can't be like this, gotta send owned subs to backend.
export function useOwnedSubscriptions() {
	const [gameSubscriptionMap, setOwnedSubscriptions] = usePersistedState<
		Partial<Record<GameSubscription, boolean>>
	>({}, "owned-game-subscriptions");

	const toggleSubscription = useCallback(
		(subscription: GameSubscription | null) => {
			if (!subscription) return;

			setOwnedSubscriptions((prev) => ({
				...prev,
				[subscription]: !prev[subscription],
			}));
		},
		[setOwnedSubscriptions],
	);

	return [gameSubscriptionMap, toggleSubscription] as const;
}
