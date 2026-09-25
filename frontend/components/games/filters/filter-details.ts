import { GamesFilter } from "@api/bindings";
import { LocalizationKey } from "@localizations/localizations";

export type FilterKey = keyof GamesFilter;

type ValueDetails = {
	noteLocalizationKey?: LocalizationKey<"filterValueNote">;
	localizationKey?: LocalizationKey<"filterValue">;
	staticDisplayText?: string;
};

export type FilterDetails = {
	localizationKey: LocalizationKey<"filterProperty">;

	// Text that shows for each filter type for the "empty value" option.
	// If not defined, the empty option is hidden from the filter menu.
	emptyLocalizationKey?: LocalizationKey<"filterValue">;

	valueDetails: Record<string, ValueDetails>;
};

export const filterDetails = Object.freeze<{
	[key in FilterKey]: FilterDetails;
}>({
	architectures: {
		localizationKey: "architecture",
		emptyLocalizationKey: "unknown",
		valueDetails: {
			X64: {
				localizationKey: "arch64",
			},
			X86: {
				localizationKey: "arch32",
			},
		},
	},
	engines: {
		localizationKey: "engine",
		emptyLocalizationKey: "unknown",
		valueDetails: {
			Godot: {
				staticDisplayText: "Godot",
			},
			GameMaker: {
				staticDisplayText: "GameMaker",
				noteLocalizationKey: "engineGameMakerNotFullySupported",
			},
			Unity: {
				staticDisplayText: "Unity",
			},
			Unreal: {
				staticDisplayText: "Unreal",
			},
		},
	},
	unityBackends: {
		localizationKey: "unityBackend",
		emptyLocalizationKey: "unknown",
		valueDetails: {
			Il2Cpp: {
				staticDisplayText: "IL2CPP",
			},
			Mono: {
				staticDisplayText: "Mono",
			},
		},
	},
	executableOs: {
		localizationKey: "executableOs",
		emptyLocalizationKey: "unknown",
		valueDetails: {
			Windows: {
				staticDisplayText: "Windows",
			},
			Linux: {
				staticDisplayText: "Linux",
			},
		},
	},
	supportedOs: {
		localizationKey: "supportedOs",
		emptyLocalizationKey: "unknown",
		valueDetails: {
			Windows: {
				staticDisplayText: "Windows",
			},
			Linux: {
				staticDisplayText: "Linux",
			},
		},
	},
	tags: {
		localizationKey: "tags",
		emptyLocalizationKey: "tagUntagged",
		valueDetails: {
			Demo: {
				localizationKey: "tagDemo",
			},
			VR: {
				localizationKey: "tagVr",
			},
		},
	},
	installed: {
		localizationKey: "status",
		valueDetails: {
			Installed: {
				localizationKey: "statusInstalled",
			},
			NotInstalled: {
				localizationKey: "statusNotInstalled",
			},
		},
	},
	providers: {
		localizationKey: "provider",
		valueDetails: {
			Epic: {
				staticDisplayText: "Epic",
			},
			Gog: {
				staticDisplayText: "GOG",
			},
			Itch: {
				staticDisplayText: "itch.io",
			},
			Manual: {
				localizationKey: "providerManual",
			},
			Steam: {
				staticDisplayText: "Steam",
			},
			Xbox: {
				staticDisplayText: "Xbox",
				noteLocalizationKey: "providerXboxOnlyInstalled",
			},
		},
	},
	modFamilies: {
		localizationKey: "mod",
		valueDetails: {},
	},
});
