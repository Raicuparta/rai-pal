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

	gamesTableColumn: {
		game: "Game",
		engine: "Engine",
		date: "Date",
	},

	modsPage: {
		openModsFolderButton: "Open Mods Folder",
		tableColumnMod: "Mod",
		tableColumnVersion: "Version",
		tableColumnModLoader: "Loader",
		tableColumnGameEngine: "Engine",
		tableColumnUnityBackend: "Backend",
		modByAuthor: "by {authorName}",
	},

	modModal: {
		runMod: "Run",
		openModFolder: "Open mod folder",
		updateMod: "Update mod",
		downloadMod: "Download mod",
		deleteMod: "Delete mod",
		deleteModConfirmation:
			"You sure? Any files inside the mod's folder will be lost.",
		byAuthor: "by {authorName}",
	},

	toolsPage: {
		openLogsFolderButton: "Open Logs Folder",
		resetRaiPalSettingsButton: "Reset Rai Pal settings",
		clearCacheButton: "Clear Rai Pal cache",
	},

	steamCache: {
		resetSteamCacheButton: "Reset Steam cache",
		resetSteamCacheModalTitle: "Reset Steam cache",
		resetSteamCacheDescription:
			"Use this if Rai Pal is showing games you don't actually own on Steam. This will reset Steam's cache, and then you'll have to restart Steam. You'll get an error if the file has already been deleted.",
		resetSteamCacheSuccess:
			"The cache file has been deleted. Please restart Steam, wait a few seconds, and then press the refresh button on Rai Pal.",
	},

	debugData: {
		debugDataTitle: "Debug data",
		debugDataCopy: "Copy debug data",
	},

	thanksPage: {
		intro:
			"Hello. I made Rai Pal. I also made other VR mods in the past, and am currently working on a universal VR mod for Unity games. If you like what I do, and would like to see more, please consider donating! You can also support me by buying one of my free mods on itch.io.",
		starRaiPalOnGitHub: "Star Rai Pal on GitHub",
		otherModdersTitle: "Other modders",
		otherModdersDescription:
			"Rai Pal is meant to help you manage game modding, and we can't do that without the tools that other developers have created. Some of these people don't have donation links, but I'm extremely grateful for their work.",
		modderOnWebsite: "{modderName} on {website}",
		patreonLeaderboard: "Patreon Leaderboard",
		rankedByPatreonDonationAmount: "Ranked by total lifetime donation amount.",
		patreonProfilePrivateNotice:
			"If you don't see yourself here, it's because your Patreon profile is private.",
	},
} as const;
