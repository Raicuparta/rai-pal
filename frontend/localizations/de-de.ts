import { Localization } from "./localizations";

export const deDe: Localization = {
	meta: {
		nativeName: "Deutsch (Deutschland)",
	},

	tab: {
		games: "Spiele",
		mods: "Mods",
		thanks: "Danke",
	},

	gamesPage: {
		emptyGamesList:
			"Rai Pal hat keine einzigen Spiele gefunden. Denk dran, dass Rai Pal installierte Apps von anderen Spieleplattformen wie Steam, Epic usw. finden muss.",
		emptyFilteredGamesList:
			"Nichts! All deine Spiele sind wegen der ausgewählten Filter versteckt. Setz deine Filter zurück, um deine schönen Spiele wiederzusehen.",
		emptyGamesLoading: "Suche nach deinen Spielen...",
	},

	addGame: {
		button: "Spiel hinzufügen",
		title: "Spiel hinzufügen",
		dropField:
			"Zieh eine Spiel-Exe hier rein oder klick, um eine Datei auszuwählen.",
		directoryButton: "Scan a folder recursively for games. Can be slow!",
		note: "Hinweis: Du kannst Spiel-Exe-Dateien überall auf Rai Pal ablegen, ohne dieses Dialogfenster zu öffnen.",
	},

	refresh: {
		button: "Aktualisieren",
		buttonUpdateRemoteDatabases: "Remote-Datenbanken aktualisieren",
		loading: "Lade {items}...",
	},

	filterMenu: {
		button: "Filter",
		resetButton: "Zurücksetzen",
		searchPlaceholder: "Suchen...",
	},

	filterProperty: {
		provider: "Anbieter",
		tags: "Tag",
		architecture: "Architektur",
		unityBackend: "Unity Backend",
		engine: "Engine",
		status: "Status",
		mod: "Mod",
	},

	filterValue: {
		unknown: "Unbekannt",
		arch64: "64-bit",
		arch32: "32-bit",
		tagDemo: "Demo",
		tagVr: "Native VR",
		tagUntagged: "Ohne Tag",
		statusInstalled: "Installiert",
		statusNotInstalled: "Nicht installiert",
		providerManual: "Manuell",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Zeigt nur PC Xbox-Spiele an, wenn sie installiert sind.",
		engineGodotNotFullySupported:
			"Godot-Spiele werden noch nicht vollständig unterstützt.",
		engineGameMakerNotFullySupported:
			"GameMaker-Spiele werden noch nicht vollständig unterstützt.",
	},

	providerCommand: {
		installGame: "Installieren",
		showGameInLibrary: "In der Bibliothek anzeigen",
		showGameInStore: "Store-Seite öffnen",
		startGameViaProvider: "Spiel starten",
		startGameViaExe: "Spiel-Exe ausführen",
		openGamePageInBrowser: "Im Browser öffnen",
	},

	gameModal: {
		startGameButton: "Spiel starten",
		startGameExecutable: "Spiel-Exe starten",
		startGameViaProvider: "Spiel starten über {provider}",
		foldersDropdown: "Ordner",
		openGameFilesFolder: "Spiel-Ordner öffnen",
		openInstalledModsFolder: "Installierte Mods öffnen",
		openGameDataFolder: "App-Daten öffnen",
		openGameWinePrefixFolder: "Wine-Prefix öffnen",
		openGameWineBinaryFolder: "Wine Binary öffnen",
		removeFromRaiPal: "Aus Rai Pal entfernen",
		removeGameConfirmation:
			"Bist du sicher, dass du dieses Spiel aus Rai Pal entfernen willst?",
		refreshGame: "Aktualisieren",
		failedToReadGameInfo:
			"Konnte wichtige Infos zu diesem Spiel nicht lesen. Vielleicht ist die Exe geschützt. Einige Mods könnten dann nicht installiert werden.",
		failedToDetermineEngine:
			"Konnte die Engine für dieses Spiel nicht bestimmen. Einige Mods könnten dann nicht funktionieren.",
		gameModsLabel: "Mods",
		gameNotInstalledWarning:
			"Das Spiel ist nicht installiert, daher bin ich mir nicht 100% sicher, welche Mods kompatibel sind. Die unten angezeigten könnten trotzdem funktionieren. Wenn du das Spiel installierst, zeig ich dir genauere Infos.",
		uninstallAllModsButton: "Alle Mods deinstallieren",
		uninstallAllModsConfirmation:
			"Sicher? Das löscht alle Dateien im Mod-Ordner des Spiels. Dateien aus dem eigentlichen Spiel bleiben aber erhalten.",

		incompatibleGameModsLabel: "Inkompatible Mods",

		incompatibleGameModsDescription:
			"Die hier aufgelisteten Mods können nicht installiert werden, da sie nicht mit der Engine-Version dieses Spiels kompatibel sind.",

		otherThings: "Sonstige Dinge",

		otherThingsDescription:
			"Das sind hauptsächlich Abhängigkeiten und andere Dinge, die du normalerweise nicht direkt bearbeiten musst.",
	},

	gameModRow: {
		editModConfig: "Mod-Konfiguration bearbeiten",
		openModConfigFolderTooltip:
			"Ordner mit den Konfigurationsdateien dieses Mods öffnen",
		openModFolder: "Mod-Ordner öffnen",
		updateMod: "Aktualisieren",
		installMod: "Installieren",
		installModAnticheatWarning:
			"Achtung: Sei vorsichtig beim Installieren von Mods in Mehrspielerspielen! Anticheat kann einige Mods erkennen und dich bannen, auch wenn sie harmlos aussehen.",
		reinstallMod: "Neu installieren",
		uninstallMod: "Deinstallieren",
		runMod: "Ausführen",
		downloadRemoteConfig: "Empfohlene Konfiguration herunterladen",
		remoteConfigAvailable:
			"Empfohlene Konfiguration verfügbar. Wird heruntergeladen, falls du noch keine hast. Du kannst auch über das Drei-Punkte-Menü den Download erzwingen.",
		modOutdated: "Mod veraltet",
		cantUninstallModWithDependants:
			"Kann keinen Mod deinstallieren, von dem andere Mods abhängen. Deinstalliere zuerst die Mods, die davon abhängen.",
	},

	gamesTableColumn: {
		game: "Spiel",
		engine: "Engine",
		date: "Datum",
	},

	modsPage: {
		openLocalModsFolderButton: "Lokale Mods öffnen",
		openLoadlModsFolderTooltip:
			"Du kannst Mods mit einer rai-pal-manifest.json in diesen Ordner legen, um sie direkt zu laden, ohne die Online-Datenbank zu nutzen.",
		tableColumnMod: "Mod",
		tableColumnVersion: "Version",
		tableColumnGameEngine: "Engine",
		tableColumnUnityBackend: "Backend",
		modByAuthor: "von {authorName}",
		modDeprecated: "Veraltet",
		modDeprecatedTooltip:
			"Dieser Mod ist veraltet. Du solltest ihn deinstallieren und eine neuere Alternative installieren.",
	},

	modModal: {
		runMod: "Ausführen",
		openModFolder: "Mod-Ordner öffnen",
		updateMod: "Mod aktualisieren",
		downloadMod: "Mod herunterladen",
		deleteMod: "Mod löschen",
		deleteModConfirmation:
			"Sicher? Alle Dateien im Mod-Ordner werden gelöscht.",
		byAuthor: "von {authorName}",
	},

	appDropdownMenu: {
		showGameThumbnails: "Spiel-Thumbnails in der Liste anzeigen",
		language: "Sprache",
		autoDetectedLanguage: "Automatisch erkannt - {languageName}",
		resetRaiPalSettingsButton: "Rai Pal-Einstellungen zurücksetzen",
		resetRaiPalSettingsTooltip:
			"Setzt Filter, Bestätigungsdialoge und wahrscheinlich andere Dinge zurück.",
		openLogsFolderButton: "Logs-Ordner öffnen",
		clearRaiPalCacheOpenModal: "Rai Pal-Cache leeren",
		clearRaiPalCacheTooltip:
			"Löscht den von Rai Pal verwendeten Spielelisten-Cache.",
	},

	userMenu: {
		unknownUser: "Unbekannter Benutzer",
		logOut: "Abmelden",
		signInWithDiscord: "Mit Discord anmelden",
		discordAccessNote:
			"Mods können das nutzen, um auf deinen Discord-Nutzernamen, Avatar, Rollen usw. zuzugreifen.",
	},

	subPage: {
		back: "Zurück",
	},

	downloadStatusMenu: {
		clear: "Abgeschlossene entfernen",
	},

	steamCache: {
		resetSteamCacheButton: "Steam-Cache zurücksetzen",
		resetSteamCacheModalTitle: "Steam-Cache zurücksetzen",
		resetSteamCacheDescription:
			"Nutze das, wenn Rai Pal Spiele anzeigt, die du eigentlich nicht besitzt. Das setzt den Steam-Cache zurück, und dann musst du Steam neu starten. Du bekommst einen Fehler, wenn die Datei bereits gelöscht wurde.",
		resetSteamCacheSuccess:
			"Die Cache-Datei wurde gelöscht. Bitte starte Steam neu, warte ein paar Sekunden und drück dann auf Aktualisieren in Rai Pal.",
	},

	steamShortcut: {
		addRaiPalSteamShortcutButton: "Rai Pal zur Steam-Bibliothek hinzufügen",
		addRaiPalSteamShortcutModalTitle: "Rai Pal zur Steam-Bibliothek hinzufügen",
		addRaiPalSteamShortcutDescription:
			"Ist besonders auf dem Steam Deck nützlich, damit du Rai Pal im Spielmodus starten kannst. Danach musst du Steam neu starten.",
		addRaiPalSteamShortcutSuccess:
			"Rai Pal wurde zu deiner Steam-Bibliothek hinzugefügt. Starte Steam neu, damit es angezeigt wird.",
	},

	globalWineOverrides: {
		setUpEnvironmentButton: "Linux-Umgebung für BepInEx einrichten",
		setUpEnvironmentTitle: "Linux-Umgebung für BepInEx einrichten",
		setUpEnvironmentDescription:
			"Bei Verwendung von Proton/Wine unter Linux wird BepInEx nicht automatisch geladen, wenn nicht einige Wine-Einstellungen gesetzt sind. Dies setzt die Umgebungsvariable WINEDLLOVERRIDES global auf 'winhttp.dll=n,b'. Wenn du das lieber manuell machen möchtest, klick nicht auf diesen Button.",
		setUpEnvironmentSuccess:
			"Die Datei wurde geschrieben. Du musst dich abmelden und wieder anmelden oder deinen Computer neu starten, damit die Änderungen wirksam werden.",
	},

	debugData: {
		debugDataTitle: "Debug-Daten",
		debugDataCopy: "Debug-Daten kopieren",
	},

	thanksPage: {
		intro:
			"Hallo. Ich habe Rai Pal gemacht. Ich hab auch in der Vergangenheit andere VR-Mods gemacht und arbeite gerade an einem universellen VR-Mod für Unity-Spiele. Wenn dir gefällt, was ich mache, und du mehr sehen willst, denk mal über eine Spende nach! Du kannst mich auch unterstützen, indem du einen meiner kostenlosen Mods auf itch.io kaufst.",
		starRaiPalOnGitHub: "Rai Pal auf GitHub ein Sternchen geben",
		otherModdersTitle: "Andere Modder",
		otherModdersDescription:
			"Rai Pal soll dir helfen, Spiele zu modden, und das schaffen wir nicht ohne die Werkzeuge, die andere Entwickler erstellt haben. Manche dieser Leute haben keine Spendenlinks, aber ich bin für ihre Arbeit extrem dankbar.",
		modderOnWebsite: "{modderName} auf {website}",
		patreonLeaderboard: "Patreon-Bestenliste",
		rankedByPatreonDonationAmount: "Nach Gesamtspenden sortiert.",
		patreonProfilePrivateNotice:
			"Siehst du dich hier nicht? Dann ist dein Patreon-Profil privat.",
	},

	commandButton: {
		cancel: "Abbrechen",
		dontAskAgain: "Nicht mehr fragen",
	},

	urlModSources: {
		title: "Mod-Quellen",
		add: "Quelle hinzufügen",
		addSourceDescription:
			"Rai Pal sucht in allen hier angegebenen Datenbanken nach Mods, zusätzlich zur Standard-Datenbank. Du kannst Mod-Quellen über den Deep-Link {deepLink} teilen.",
		confirmModalTitle: "Mod-Quelle bestätigen",
		modsFound: "{count} Mods in dieser Quelle gefunden",
		addSource: "Quelle hinzufügen",
		cancel: "Abbrechen",
		loading: "Mods werden geladen...",
	},
} as const;
