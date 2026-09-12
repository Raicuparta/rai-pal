import { AppSettings, GamesQuery } from "@api/bindings";

// A filter value that is missing from `known` (or a `null` `unknown`) means it's
// enabled. Empty groups therefore represent "all values selected", which must
// match the Rust `GamesFilter::default()`.
export const defaultQuery: GamesQuery = {
	sortBy: "Title",
	search: "",
	sortDescending: false,
	filter: {
		architectures: { known: {}, unknown: null },
		engines: { known: {}, unknown: null },
		providers: { known: {}, unknown: null },
		tags: { known: {}, unknown: null },
		unityBackends: { known: {}, unknown: null },
		os: { known: {}, unknown: null },
		installed: { known: {}, unknown: null },
		modFamilies: { known: {}, unknown: null },
	},
};

export const defaultSettings: AppSettings = {
	hideGameThumbnails: false,
	overrideLanguage: null,
	gamesQuery: defaultQuery,
	selectedTab: "Games",
	skipConfirmDialogs: [],
};
