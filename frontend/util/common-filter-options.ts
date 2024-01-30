import { GameEngineBrand, ProviderId } from "@api/bindings";
import { FilterOption } from "@components/table/table-head";

export const engineFilterOptions: FilterOption<GameEngineBrand>[] = [
	{ label: "Unity", value: "Unity" },
	{ label: "Unreal", value: "Unreal" },
	{ label: "Godot", value: "Godot" },
	{ label: "GMaker", value: "GMaker" },
];

export const providerFilterOptions: FilterOption<ProviderId>[] = [
	{ label: "Steam", value: "Steam" },
	{ label: "Epic", value: "Epic" },
	{ label: "GOG", value: "Gog" },
	{ label: "Xbox", value: "Xbox" },
	{ label: "Itch", value: "Itch" },
	{ label: "Manual", value: "Manual" },
];
