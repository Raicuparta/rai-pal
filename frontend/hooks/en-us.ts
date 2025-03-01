export const enUs = {
	tab: {
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

	filterMenu: {
		button: "Filter",
		resetButton: "Reset",
		searchPlaceholder: "Search...",
	},

	filterProperty: {
		provider: "Provider",
		tags: "Tag",
		architecture: "Architecture",
		unityScriptingBackend: "Unity Backend",
		engine: "Engine",
		status: "Status",
	},

	filterValue: {
		unknown: "Unknown",
		arch64: "64-bit",
		arch32: "32-bit",
		engineGodot: "Godot",
		engineGameMaker: "GameMaker",
		engineUnity: "Unity",
		engineUnreal: "Unreal",
		unityBackendIl2Cpp: "IL2CPP",
		unityBackendMono: "Mono",
		tagDemo: "Demo",
		tagVr: "Native VR",
		tagUntagged: "Untagged",
		statusInstalled: "Installed",
		statusNotInstalled: "Not installed",
		providerSteam: "Steam",
		providerGog: "GOG",
		providerEpic: "Epic",
		providerItch: "itch.io",
		providerOrigin: "Origin",
		providerManual: "Manual",
		providerXbox: "Xbox",
		providerEa: "EA",
		providerUbisoft: "Ubisoft",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Xbox PC games only show on Rai Pal once they're installed.",
		engineGodotNotFullySupported: "Godot games are not fully supported yet.",
		engineGameMakerNotFullySupported:
			"GameMaker games are not fully supported yet.",
	},

	providerCommand: {
		installGame: "Install",
		showGameInLibrary: "Show In Library",
		showGameInStore: "Open Store Page",
		startGame: "Start Game",
		openGamePageInBrowser: "Open In Browser",
	},

	gameModal: {
		startGameButton: "Start Game",
		startGameExecutable: "Start Game Executable",
		startGameViaProvider: "Start Game via {provider}",
		foldersDropdown: "Folders",
		openGameFilesFolder: "Open Game Files Folder",
		openInstalledModsFolder: "Open Installed Mods Folder",
		removeGameConfirmation:
			"Are you sure you want to remove this game from Rai Pal?",
		removeFromRaiPal: "Remove from Rai Pal",
		refreshGame: "Refresh",
		failedToReadGameInfo:
			"Failed to read some important information about this game. This could be due to the executable being protected. Some mods might fail to install.",
		failedToDetermineEngine:
			"Failed to determine the engine for this game. Some mods might fail to install.",
		gameModsLabel: "Mods",
		gameNotInstalledWarning:
			"This game isn't installed, so I'm not 100% sure which mods are compatible. The ones you see below might work. If you install the game, I'll show you more accurate information.",
		uninstallAllModsConfirmation:
			"You sure? This will delete all files in this game's mods folder. It won't delete any files from the actual game though.",
		uninstallAllModsButton: "Uninstall all mods",
	},
} as const;
