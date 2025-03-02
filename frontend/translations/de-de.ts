export const deDe = {
	tab: {
		games: "Spiele",
		mods: "Mods",
		tools: "Werkzeuge",
		thanks: "Danke",
	},

	addGame: {
		button: "Spiel hinzufügen",
		title: "Spiel hinzufügen",
		dropField:
			"Ziehe eine Spiel-Executable hierher oder klicke, um eine Datei auszuwählen.",
		note: "Hinweis: Du kannst Spiel-Executables überall im Rai Pal-Fenster ablegen, um sie zur Liste der installierten Spiele hinzuzufügen, ohne dieses Dialogfeld zu öffnen.",
	},

	refresh: {
		button: "Aktualisieren",
		loading: "Lade {items}...",
	},

	filterMenu: {
		button: "Filter",
		resetButton: "Zurücksetzen",
		searchPlaceholder: "Suche...",
	},

	filterProperty: {
		provider: "Anbieter",
		tags: "Tag",
		architecture: "Architektur",
		unityScriptingBackend: "Unity-Backend",
		engine: "Engine",
		status: "Status",
	},

	filterValue: {
		unknown: "Unbekannt",
		arch64: "64-Bit",
		arch32: "32-Bit",
		engineGodot: "Godot",
		engineGameMaker: "GameMaker",
		engineUnity: "Unity",
		engineUnreal: "Unreal",
		unityBackendIl2Cpp: "IL2CPP",
		unityBackendMono: "Mono",
		tagDemo: "Demo",
		tagVr: "Native VR",
		tagUntagged: "Ohne Tag",
		statusInstalled: "Installiert",
		statusNotInstalled: "Nicht installiert",
		providerSteam: "Steam",
		providerGog: "GOG",
		providerEpic: "Epic",
		providerItch: "itch.io",
		providerOrigin: "Origin",
		providerManual: "Manuell",
		providerXbox: "Xbox",
		providerEa: "EA",
		providerUbisoft: "Ubisoft",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Xbox-PC-Spiele werden in Rai Pal nur angezeigt, wenn sie installiert sind.",
		engineGodotNotFullySupported:
			"Godot-Spiele werden noch nicht vollständig unterstützt.",
		engineGameMakerNotFullySupported:
			"GameMaker-Spiele werden noch nicht vollständig unterstützt.",
	},

	providerCommand: {
		installGame: "Installieren",
		showGameInLibrary: "In Bibliothek anzeigen",
		showGameInStore: "Store-Seite öffnen",
		startGame: "Spiel starten",
		openGamePageInBrowser: "Im Browser öffnen",
	},

	gameModal: {
		startGameButton: "Spiel starten",
		startGameExecutable: "Spiel-Executable starten",
		startGameViaProvider: "Spiel über {provider} starten",
		foldersDropdown: "Ordner",
		openGameFilesFolder: "Spieldateien-Ordner öffnen",
		openInstalledModsFolder: "Installierte Mods-Ordner öffnen",
		removeFromRaiPal: "Aus Rai Pal entfernen",
		removeGameConfirmation:
			"Bist du sicher, dass du dieses Spiel aus Rai Pal entfernen möchtest?",
		refreshGame: "Aktualisieren",
		failedToReadGameInfo:
			"Fehlgeschlagen, wichtige Informationen über dieses Spiel zu lesen. Dies könnte an einer geschützten Executable liegen. Einige Mods könnten fehlschlagen.",
		failedToDetermineEngine:
			"Fehlgeschlagen, die Engine dieses Spiels zu bestimmen. Einige Mods könnten fehlschlagen.",
		gameModsLabel: "Mods",
		gameNotInstalledWarning:
			"Dieses Spiel ist nicht installiert, daher bin ich mir nicht zu 100 % sicher, welche Mods kompatibel sind. Die unten angezeigten könnten funktionieren. Falls du das Spiel installierst, kann ich genauere Informationen anzeigen.",
		uninstallAllModsButton: "Alle Mods deinstallieren",
		uninstallAllModsConfirmation:
			"Bist du sicher? Dadurch werden alle Dateien im Mod-Ordner dieses Spiels gelöscht. Die Spieldateien selbst bleiben jedoch unberührt.",
	},

	gamesTableColumn: {
		game: "Spiel",
		engine: "Engine",
		date: "Datum",
	},

	modsPage: {
		openModsFolderButton: "Mods-Ordner öffnen",
		tableColumnMod: "Mod",
		tableColumnVersion: "Version",
		tableColumnModLoader: "Loader",
		tableColumnGameEngine: "Engine",
		tableColumnUnityBackend: "Backend",
		modByAuthor: "von {authorName}",
		modDeprecated: "Veraltet",
		modDeprecatedTooltip:
			"Diese Mod ist veraltet. Du solltest sie deinstallieren und eine neuere Alternative installieren.",
		modOutdated: "Mod veraltet",
	},

	modModal: {
		runMod: "Ausführen",
		openModFolder: "Mod-Ordner öffnen",
		updateMod: "Mod aktualisieren",
		downloadMod: "Mod herunterladen",
		deleteMod: "Mod löschen",
		deleteModConfirmation:
			"Bist du sicher? Alle Dateien im Mod-Ordner gehen verloren.",
		byAuthor: "von {authorName}",
	},

	toolsPage: {
		openLogsFolderButton: "Logs-Ordner öffnen",
		resetRaiPalSettingsButton: "Rai Pal-Einstellungen zurücksetzen",
		resetRaiPalSettingsTooltip:
			"Setzt Filter, Bestätigungsdialoge und wahrscheinlich weitere Dinge zurück.",
		clearCacheButton: "Rai Pal-Cache leeren",
		clearCacheTooltip: "Löscht den von Rai Pal verwendeten Spielelisten-Cache.",
	},

	steamCache: {
		resetSteamCacheButton: "Steam-Cache zurücksetzen",
		resetSteamCacheModalTitle: "Steam-Cache zurücksetzen",
		resetSteamCacheDescription:
			"Nutze dies, falls Rai Pal Spiele anzeigt, die du nicht besitzt. Dies setzt den Steam-Cache zurück, danach musst du Steam neu starten.",
		resetSteamCacheSuccess:
			"Die Cache-Datei wurde gelöscht. Bitte starte Steam neu, warte einige Sekunden und drücke dann den Aktualisieren-Button in Rai Pal.",
	},

	debugData: {
		debugDataTitle: "Debug-Daten",
		debugDataCopy: "Debug-Daten kopieren",
	},

	thanksPage: {
		intro: "Hallo. Ich habe Rai Pal gemacht...",
		starRaiPalOnGitHub: "Rai Pal auf GitHub mit Stern markieren",
		otherModdersTitle: "Andere Modder",
		otherModdersDescription:
			"Rai Pal unterstützt Mods, dank vieler Entwickler...",
		modderOnWebsite: "{modderName} auf {website}",
		patreonLeaderboard: "Patreon-Bestenliste",
	},

	commandButton: {
		cancel: "Abbrechen",
		dontAskAgain: "Nicht erneut fragen",
	},
};
