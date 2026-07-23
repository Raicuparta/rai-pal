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

	gamesPage: {
		// Message that shows when the filters applied to the game list make it return zero games.
		emptyGamesList:
			"Rai Pal didn't find any games at all. Remember Rai Pal needs to find installed apps from other game providers, like Steam, Epic, etc.",

		// Message that shows when Rai Pal didn't find any games at all.
		emptyFilteredGamesList:
			"Nothing! All of your games are hidden because of the filters you selected. Clear your filters to see your beautiful games again.",

		// Message that shows while games are loading, and no games are available yet.
		emptyGamesLoading: "Finding your games...",
	},

	manualGames: {
		manualSteamSupportNote:
			"Note: you can also add your non-Steam games to Steam, and they will still be detected by Rai Pal.",

		savedDirectories:
			"These directories will be scanned recursively every time Rai Pal refreshes its data. Large folders may make things significantly slower.",

		// Button for adding games manually to Rai Pal.
		button: "Add games",

		// Title of the modal for manually adding games.
		title: "Manually Added Games",

		// Button for selecting a single game executable.
		selectGameExecutable: "Select game executable",

		// Button for adding a game directory to scan for multiple games, with a warning about it being potentially slow.
		selectGamesDirectory: "Select game directory. Can be slow!",

		// Note that explains you can drop files and folders onto Rai Pal's window.
		fileDropNote:
			"You can also just drop game executable files or entire folders anywhere on Rai Pal's window to add them to the library, even without opening this dialog.",

		// Shown while scanning a directory for game executables.
		scanning: "Scanning {path}...",

		// Shows the scan progress. {directories} and {executables} are numbers.
		scanProgress:
			"Scanned {directories} directories, found {executables} executables",

		// Shows the current path being scanned.
		currentPath: "Current: {path}",

		// Shown after scan completes, asking the user to confirm. {gamesCount} is a number, {duration} is in seconds.
		scanComplete:
			"Found {gamesCount} executables in {duration} seconds. Add this folder to Rai Pal?",

		// Button to confirm adding a scanned folder.
		confirmAddFolder: "Add folder",

		// Button to cancel adding the folder (during or after scan).
		cancel: "Cancel",
	},

	refresh: {
		// Button for refreshing games and mods.
		button: "Refresh",

		buttonUpdateRemoteDatabases: "Update remote databases",

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
		unityBackend: "Unity Backend",

		// Game engine (Unity, Unreal, etc)
		engine: "Engine",

		// Game installation status (Installed, Not installed). Might have more statuses in the future.
		status: "Status",

		// Mod compatibility.
		mod: "Mod",
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
		providerXboxOnlyInstalled: "Only shows PC Xbox games if they're installed.",
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
		startGameViaProvider: "Start Game",

		// Start the game via this game's executable.
		startGameViaExe: "Run Game Executable",

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
		openGameFilesFolder: "Game Files",

		// Open the folder where Rai Pal places the mods for this game.
		openInstalledModsFolder: "Installed Mods",

		// Open the folder where the game stores its app data.
		openGameDataFolder: "Game App Data",

		// Open the Wine prefix folder detected for this game.
		openGameWinePrefixFolder: "Wine Prefix",

		// Open the folder that contains the Wine binary used for this game.
		openGameWineBinaryFolder: "Wine Binary",

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

		// Label for the section that lists incompatible mods for this game.
		incompatibleGameModsLabel: "Incompatible Mods",

		incompatibleGameModsDescription:
			"The mods listed here can't be installed because they aren't compatible with this game's engine version.",

		// Button that expands the section showing hidden "mods", which sometimes aren't mods at all so we say "things" here.
		otherThings: "Other things",

		// Description for the section that shows hidden "mods".
		otherThingsDescription:
			"These are mostly dependencies and other stuff that you don't usually need to mess with directly.",
	},

	// Row in the table that shows mods for a specific game in the game modal.
	gameModRow: {
		// Button that directly opens the file (or folder) for the config of the mod installed in this game.
		editModConfig: "Edit Mod Config",

		// Tooltip that shows when hovering the button that opens the folder that contains the config file for the mod installed on this game.
		openModConfigFolderTooltip:
			"Open folder containing this mod's config files",

		// Button for opening the folder where a specific mod is installed.
		openModFolder: "Open Mod Folder",

		// Button for updating the install mod for this game to the latest version.
		updateMod: "Update",

		// Button for installing a mod for this game.
		installMod: "Install",

		// Warning that shows before installing a mod for the first time.
		installModAnticheatWarning:
			"Attention: be careful when installing mods on multiplayer games! Anticheat can detect some mods and get you banned, even if the mods seem harmless.",

		// Button for reinstalling a mod that's already installed.
		reinstallMod: "Reinstall",

		// Button for uninstalling a mod for this game.
		uninstallMod: "Uninstall",

		// For mods that can be executed (like UEVR), this button runs them.
		runMod: "Run",

		// Button for downloading a mod config from the database.
		downloadRemoteConfig: "Download Recommended Config",

		// Tooltip for icon that shows next to mod to indicate there's a downloadable config.
		remoteConfigAvailable:
			"Recommended config available. Will be downloaded if you don't already have a config. You can also force it to download from the three dot menu.",

		// Badge that shows when a new version of a mod is available.
		modOutdated: "Mod outdated",

		// Tooltip that shows when hovering the uninstall button for a mod that has other mods depending on it.
		cantUninstallModWithDependants:
			"Can't uninstall a mod that has dependants. Uninstall the mods that depend on this one first.",
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
		// Button for opening the folder where loads can be loaded from disk without being in the online database.
		openLocalModsFolderButton: "Open Local Mods Folder",

		// Describes what the local mods folder is.
		openLoadlModsFolderTooltip:
			"You can place mods in this folder with a rai-pal-manifest.json, to be loaded directly without going through the online database.",

		// Table column for the name of the mod.
		tableColumnMod: "Mod",

		// Table column for the version of the mod.
		tableColumnVersion: "Version",

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
		// Label for the checkbox that toggles visibility of game images.
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
		clearRaiPalCacheOpenModal: "Clear Rai Pal cache",

		// Tooltip for the button that clears Rai Pal's local game database cache.
		clearRaiPalCacheTooltip: "Clears the game list cache used by Rai Pal.",
	},

	// Menu for Discord account login/logout and account info.
	userMenu: {
		// Fallback label when Rai Pal has no username for the logged in account.
		unknownUser: "Unknown user",

		// Button for logging out of Discord in the user menu.
		logOut: "Log out",

		// Button for starting the Discord OAuth login flow.
		signInWithDiscord: "Sign in with Discord",

		// Description text under the Discord sign-in button.
		discordAccessNote:
			"Mods can use this to access your Discord username, avatar, roles, etc.",
	},

	// Shared header shown in sub-pages and modals.
	subPage: {
		// Button to close the sub-page and go back to the previous view.
		back: "Back",
	},

	// Menu that shows active and completed file downloads.
	downloadStatusMenu: {
		// Button that removes finished downloads from the list.
		clear: "Clear completed",
	},

	// Button and modal for resetting Steam's cache.
	steamCache: {
		// Button for resetting Steam's cache.
		resetSteamCacheButton: "Reset Steam cache",

		// Title of the modal that opens after clicking the reset Steam cache button.
		resetSteamCacheModalTitle: "Reset Steam cache",

		// Description of what resetting Steam's cache does.
		resetSteamCacheDescription:
			"Use this if Rai Pal is showing games you don't actually own on Steam. This will reset Steam's cache, and then you'll have to restart Steam. You'll get an error if the file has already been deleted.",

		// Success message after resetting Steam's cache.
		resetSteamCacheSuccess:
			"The cache file has been deleted. Please restart Steam, wait a few seconds, and then press the refresh button on Rai Pal.",
	},

	// Button and modal for adding Rai Pal to Steam as a shortcut.
	steamShortcut: {
		// Button that opens the add-to-Steam modal.
		addRaiPalSteamShortcutButton: "Add Rai Pal to Steam library",

		// Title of the modal that opens after clicking the add-to-Steam button.
		addRaiPalSteamShortcutModalTitle: "Add Rai Pal to Steam Library",

		// Description of what adding Rai Pal to Steam does.
		addRaiPalSteamShortcutDescription:
			"This is especially useful on Steam Deck, to be able to launch Rai Pal in Game Mode. You'll have to restart Steam after doing this.",

		// Success message after adding Rai Pal to Steam.
		addRaiPalSteamShortcutSuccess:
			"Rai Pal has been added to your Steam library. Restart Steam to see it.",
	},

	// Button and modal for making sure BepInEx loads on Linux with Wine.
	globalWineOverrides: {
		// Button that sets up the Linux environment for BepInEx.
		setUpEnvironmentButton: "Set up Linux environment for BepInEx",

		// Title of the modal that opens after clicking the set up Linux environment button.
		setUpEnvironmentTitle: "Set up Linux environment for BepInEx",

		// Description of what setting up the Linux environment for BepInEx does.
		setUpEnvironmentDescription:
			"When using Proton/Wine on Linux, BepInEx won't load automatically unless some Wine settings are set. This will set the environment variable WINEDLLOVERRIDES to 'winhttp.dll=n,b' globally. If you'd rather do that manually, then don't click this button.",

		// Success message after setting up the Linux environment for BepInEx.
		setUpEnvironmentSuccess:
			"File has been written. You will need to log out and log back in, or restart your computer, for the changes to take effect.",
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

	// Modal for managing custom mod database URLs.
	urlModSources: {
		// Title of the modal.
		title: "Mod Sources",

		add: "Add source",

		// Description under the text input for adding a source URL.
		addSourceDescription:
			"Rai Pal will check for mods in all databases provided here, in addition to checking the default mod database. You can share mod sources via the deep-link {deepLink}.",

		// Title for the confirmation modal when adding a new source.
		confirmModalTitle: "Confirm Mod Source",

		// Label showing how many mods were found in the source.
		modsFound: "{count} mods found in this source",

		// Button to confirm adding the source.
		addSource: "Add Source",

		// Button to cancel adding the source.
		cancel: "Cancel",

		// Text shown while the source URL is being fetched.
		loading: "Loading mods...",
	},
} as const;
