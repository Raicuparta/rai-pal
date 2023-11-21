import { GameEngineBrand, ProviderId } from "@api/bindings";
import { SegmentedControlData } from "../components/installed-games/typed-segmented-control";

export const engineFilterOptions: SegmentedControlData<GameEngineBrand>[] = [
	{ label: "Any Engine", value: "" },
	{ label: "Unity", value: "Unity" },
	{ label: "Unreal", value: "Unreal" },
	{ label: "Godot", value: "Godot" },
];

export const providerFilterOptions: SegmentedControlData<ProviderId>[] = [
	{ label: "Any provider", value: "" },
	{ label: "Steam", value: "Steam" },
	{ label: "Manual", value: "Manual" },
];
