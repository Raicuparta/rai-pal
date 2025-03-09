// This is the source of truth for translations.
// All other translations need to match the keys in this one, and must have the same arguments too.
// Comments are included here to give context to each string, which is especially useful for evil AI translations.

// When significantly changing the meaning of a string, it's best to also change its localization key.
// This way we can ensure that all translations get updated to reflect the new meaning.

export const enUs = {
	meta: {
		nativeName: "English (US)",
	},
	tab: {
		// Name of the tab that shows the user's games from all providers.
		games: "Games",

		// Name of the tab that shows all the available mods.
		mods: "Mods",

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

	// Menu for filtering games.
	filterMenu: {
		// Text in the filter button.
		button: "Filter",

		// Text in the reset button for each specific filterable property.
		resetButton: "Reset",

		// Placeholder text in the search input.
		searchPlaceholder: "Search...",
	},

	// Different properties that the user can filter games by.
	filterProperty: {
		// Game provider (Steam, Epic, etc)
		provider: "Provider",

		// Game tags (Demo, VR, etc)
		tags: "Tag",

		// Game executable architecture (32-bit, 64-bit)
		architecture: "Architecture",

		// Unity scripting backend (Mono, IL2CPP)
		unityScriptingBackend: "Unity Backend",

		// Game engine (Unity, Unreal, etc)
		engine: "Engine",

		// Game installation status (Installed, Not installed). Might have more statuses in the future.
		status: "Status",
	},

	// Different values for each of the filterable properties.
	filterValue: {
		// This can apply to multiple properties.
		unknown: "Unknown",

		// Game executable architectures
		arch64: "64-bit",
		arch32: "32-bit",

		// Game tags
		tagDemo: "Demo",
		tagVr: "Native VR",
		tagUntagged: "Untagged",

		// Game statuses
		statusInstalled: "Installed",
		statusNotInstalled: "Not installed",

		// Game provider for games that the user added manually to Rai Pal.
		providerManual: "Manual",
	},

	// Some filter values show extra notes when the user hovers that value.
	filterValueNote: {
		providerXboxOnlyInstalledAndSubscription:
			"Only shows PC Xbox games if they're installed, or if you own them as part of the PC Game Pass subscription.",
		providerUbisoftOnlySubscription:
			"Only shows Ubisoft games if you own them as part of the Ubisoft+ subscription.",
		engineGodotNotFullySupported: "Godot games are not fully supported yet.",
		engineGameMakerNotFullySupported:
			"GameMaker games are not fully supported yet.",
	},

	// Commands that can be run on a game, depending on the game's provider and status.
	providerCommand: {
		// Install an owned game.
		installGame: "Install",

		// Show an owned game in the library, on this provider's launcher app.
		showGameInLibrary: "Show In Library",

		// Open the store page for a game, on this provider's launcher app.
		showGameInStore: "Open Store Page",

		// Start the game via this provider's launcher app.
		startGame: "Start Game",

		// Open the game's store page in the user's default browser.
		openGamePageInBrowser: "Open In Browser",
	},

	// This is the modal that opens after clicking a game.
	gameModal: {
		// Button for starting the game. Also has a dropdown for different ways to start it.
		startGameButton: "Start Game",

		// Start the game directly via the detected executable file.
		startGameExecutable: "Start Game Executable",

		// Start the game via the provider's launcher app.
		startGameViaProvider: "Start Game via {provider}",

		// Dropdown menu for folders related to the game.
		foldersDropdown: "Folders",

		// Open the folder where the game's executable is located.
		openGameFilesFolder: "Open Game Files Folder",

		// Open the folder where Rai Pal places the mods for this game.
		openInstalledModsFolder: "Open Installed Mods Folder",

		// Button for removing a manually-added game from Rai Pal.
		removeFromRaiPal: "Remove from Rai Pal",

		// Confirmation dialog for removing a manually-added game from Rai Pal.
		removeGameConfirmation:
			"Are you sure you want to remove this game from Rai Pal?",

		// Button for refreshing the game's information.
		refreshGame: "Refresh",

		// Error message when Rai Pal fails to read enough information about the game.
		failedToReadGameInfo:
			"Failed to read some important information about this game. This could be due to the executable being protected. Some mods might fail to install.",

		// Error message when Rai Pal fails to determine the game's engine.
		failedToDetermineEngine:
			"Failed to determine the engine for this game. Some mods might fail to install.",

		// Label for the section that shows mods for this game.
		gameModsLabel: "Mods",

		// Warning that shows when a game isn't installed.
		gameNotInstalledWarning:
			"This game isn't installed, so I'm not 100% sure which mods are compatible. The ones you see below might work. If you install the game, I'll show you more accurate information.",

		// Button for uninstalling all mods for this game.
		uninstallAllModsButton: "Uninstall all mods",

		// Confirmation dialog for uninstalling all mods for this game.
		uninstallAllModsConfirmation:
			"You sure? This will delete all files in this game's mods folder. It won't delete any files from the actual game though.",
	},

	// Named table columns for the list of games.
	gamesTableColumn: {
		// This column is mostly for the game's name, but includes other information like tags.
		game: "Game",

		// The game's engine (Unity, Unreal, etc), engine version, etc.
		engine: "Engine",

		// The date when the game was released (or added to the provider).
		date: "Date",
	},

	// Page that shows all mods available on Rai Pal
	modsPage: {
		// Button for opening the folder where Rai Pal downloads mods to (before installing them in a game).
		openModsFolderButton: "Open Mods Folder",

		// Table column for the name of the mod.
		tableColumnMod: "Mod",

		// Table column for the version of the mod.
		tableColumnVersion: "Version",

		// Table column for the mod loader used by the mod (bepinex, melon loader, etc)
		tableColumnModLoader: "Loader",

		// Table column for the game engine the mod is for (Unity, Unreal, etc)
		tableColumnGameEngine: "Engine",

		// Table column for the Unity scripting backend the mod is for (Mono, IL2CPP)
		tableColumnUnityBackend: "Backend",

		// Label for the author of the mod. Shows after the mod's name.
		modByAuthor: "by {authorName}",

		// Badge that shows when a mod is deprecated.
		modDeprecated: "Deprecated",

		// Tooltip that shows when hovering over a deprecated mod.
		modDeprecatedTooltip:
			"This mod is deprecated. You should uninstall it and install a newer alternative.",

		// Badge that shows when a new version of a mod is available.
		modOutdated: "Mod outdated",
	},

	// Modal that opens after clicking a mod in the mods page.
	modModal: {
		// For mods that can be executed (like UEVR), this button runs them.
		runMod: "Run",

		// Opens the folder where the mod's files are located.
		openModFolder: "Open mod folder",

		// Button for updating a mod to the latest version.
		updateMod: "Update mod",

		// Button for downloading a mod.
		downloadMod: "Download mod",

		// Button for deleting a mod.
		deleteMod: "Delete mod",

		// Confirmation dialog for deleting a mod.
		deleteModConfirmation:
			"You sure? Any files inside the mod's folder will be lost.",

		// Label for the author of the mod. Shows after the mod's name.
		byAuthor: "by {authorName}",
	},

	// Text in the dropdown menu for tools and settings.
	appDropdownMenu: {
		// Checkbox label that toggles showing thumbnails on the game list.
		showGameThumbnails: "Show game thumbnails on list",

		// Label on the dropdown for changing the app's language.
		language: "Language",

		// Display name for the language option that automatically detects the user's language.
		autoDetectedLanguage: "Auto-detected - {languageName}",

		// Button for resetting Rai Pal's settings.
		resetRaiPalSettingsButton: "Reset Rai Pal settings",

		// Tooltip for the button that resets Rai Pal's settings.
		resetRaiPalSettingsTooltip:
			"Will reset filters, confirmation dialogs, probably other stuff.",

		// Button for opening the folder where Rai Pal stores its debug logs.
		openLogsFolderButton: "Open Logs Folder",

		// Button for clearing Rai Pal's local game database cache. The ellipsis indicates that a modal will open.
		clearRaiPalCacheOpenModal: "Clear Rai Pal cache...",

		// Tooltip for the button that clears Rai Pal's local game database cache.
		clearRaiPalCacheTooltip: "Clears the game list cache used by Rai Pal.",
	},

	// Button and modal for resetting Steam's cache.
	steamCache: {
		// Button for resetting Steam's cache. The ellipsis indicates that a modal will open.
		resetSteamCacheButtonOpenModal: "Reset Steam cache...",

		// Title of the modal that opens after clicking the reset Steam cache button.
		resetSteamCacheModalTitle: "Reset Steam cache",

		// Description of what resetting Steam's cache does.
		resetSteamCacheDescription:
			"Use this if Rai Pal is showing games you don't actually own on Steam. This will reset Steam's cache, and then you'll have to restart Steam. You'll get an error if the file has already been deleted.",

		// Success message after resetting Steam's cache.
		resetSteamCacheSuccess:
			"The cache file has been deleted. Please restart Steam, wait a few seconds, and then press the refresh button on Rai Pal.",
	},

	// Debug data that shows in modals for games and mods.
	// Basically a JSON dump of all the info Rai Pal has on that game / mod.
	debugData: {
		// Title of the debug data section.
		debugDataTitle: "Debug data",

		// Button for copying the debug data to the clipboard.
		debugDataCopy: "Copy debug data",
	},

	// Page that shows credits and donation links.
	thanksPage: {
		// Short intro about Raicuparta.
		intro:
			"Hello. I made Rai Pal. I also made other VR mods in the past, and am currently working on a universal VR mod for Unity games. If you like what I do, and would like to see more, please consider donating! You can also support me by buying one of my free mods on itch.io.",

		// Button that opens the Rai Pal GitHub repository.
		starRaiPalOnGitHub: "Star Rai Pal on GitHub",

		// Title of the section that shows other modders featured in Rai Pal.
		otherModdersTitle: "Other modders",

		// Description of the 'Other modders' section.
		otherModdersDescription:
			"Rai Pal is meant to help you manage game modding, and we can't do that without the tools that other developers have created. Some of these people don't have donation links, but I'm extremely grateful for their work.",

		// Button that opens a modder's profile on a website.
		modderOnWebsite: "{modderName} on {website}",

		// Title of the section that shows top Patreon supporters.
		patreonLeaderboard: "Patreon Leaderboard",

		// Explanation of the Patreon Leaderboard ranking.
		rankedByPatreonDonationAmount: "Ranked by total lifetime donation amount.",

		// Note about private Patreon profiles.
		patreonProfilePrivateNotice:
			"If you don't see yourself here, it's because your Patreon profile is private.",
	},

	// Buttons that run commands that show confirmation dialogues.
	commandButton: {
		// Button for cancelling a command in its confirmation dialogue.
		cancel: "Cancel",

		// Label of the checkbox for skipping future confirmations for a command.
		dontAskAgain: "Don't ask again",
	},
} as const;
