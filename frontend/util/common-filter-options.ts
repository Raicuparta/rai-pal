import { EngineBrand, GameTag, ProviderId } from "@api/bindings";
import { FilterOption } from "@components/table/table-head";

export const engineFilterOptions: FilterOption<EngineBrand>[] = [
	{ label: "Unity", value: "Unity" },
	{ label: "Unreal", value: "Unreal" },
	{ label: "Godot", value: "Godot" },
	{ label: "Game Maker", value: "GameMaker" },
];

export const providerFilterOptions: FilterOption<ProviderId>[] = [
	{ label: "Steam", value: "Steam" },
	{ label: "Epic", value: "Epic" },
	{ label: "GOG", value: "Gog" },
	{ label: "Xbox", value: "Xbox" },
	{ label: "Itch", value: "Itch" },
	{ label: "Ubisoft", value: "Ubisoft" },
	{ label: "Manual", value: "Manual" },
];

export const tagFilterOptions: FilterOption<GameTag>[] = [
	{ label: "VR", value: "VR" },
	{ label: "Demo", value: "Demo" },
];
