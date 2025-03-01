export const enUs = {
	tabs: {
		// Name of the tab that shows the user's games from all providers.
		games: "Games",

		// Name of the tab that shows all the available mods.
		mods: "Mods",

		// Name of the tab that shows tools and settings.
		tools: "Tools",

		// Name of the tab that shows credits and donation links.
		thanks: "Thanks",
	},

	addGame: {
		// Button for adding a game to Rai Pal.
		button: "Add game",

		// Title of the modal for adding a game.
		title: "Add game",

		// Text inside the file drop area for adding a game.
		dropField:
			"Drag and drop a game executable here, or click to select a file.",

		// Note that shows under the file drop area for adding a game.
		note: "Note: you can drop game executable files anywhere on Rai Pal's window to add them to the installed game list without opening this dialog.",
	},

	refresh: {
		// Button for refreshing games and mods.
		button: "Refresh",

		// Small text that shows inside the refresh button, while stuff is loading. {items} is a comma-separated list.
		loading: "Loading {items}...",
	},
} as const;
