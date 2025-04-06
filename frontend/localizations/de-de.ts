import { Localization } from "./localizations";

export const deDe: Localization = {
	meta: {
		nativeName: "Deutsch (Deutschland)",
	},

	tab: {
		games: "Spiele",
		mods: "Mods",
		thanks: "Dank",
	},

	addGame: {
		button: "Spiel hinzufügen",
		title: "Spiel hinzufügen",
		dropField:
			"Ziehen Sie eine Spielexe hierher oder klicken Sie, um eine Datei auszuwählen.",
		note: "Hinweis: Sie können Spielexe-Dateien überall im Rai Pal-Fenster ablegen, um sie der Liste der installierten Spiele hinzuzufügen, ohne dieses Dialogfeld zu öffnen.",
	},

	refresh: {
		button: "Aktualisieren",
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
	},

	filterValue: {
		unknown: "Unbekannt",
		arch64: "64-bit",
		arch32: "32-bit",
		tagDemo: "Demo",
		tagVr: "Native VR",
		tagUntagged: "Nicht getaggt",
		statusInstalled: "Installiert",
		statusNotInstalled: "Nicht installiert",
		providerManual: "Manuell",
	},

	filterValueNote: {
		providerXboxOnlyInstalledAndSubscription:
			"Zeigt nur PC-Xbox-Spiele an, wenn sie installiert sind oder wenn Sie sie im Rahmen des PC Game Pass-Abonnements besitzen.",
		providerUbisoftOnlySubscription:
			"Zeigt Ubisoft-Spiele nur an, wenn Sie sie im Rahmen des Ubisoft+-Abonnements besitzen.",
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
		startGameExecutable: "Spiel-Exe starten",
		startGameViaProvider: "Spiel starten über {provider}",
		foldersDropdown: "Ordner",
		openGameFilesFolder: "Spiel-Dateien-Ordner öffnen",
		openInstalledModsFolder: "Installierte Mods-Ordner öffnen",
		removeFromRaiPal: "Aus Rai Pal entfernen",
		removeGameConfirmation:
			"Sind Sie sicher, dass Sie dieses Spiel aus Rai Pal entfernen möchten?",
		refreshGame: "Aktualisieren",
		failedToReadGameInfo:
			"Fehler beim Lesen wichtiger Informationen über dieses Spiel. Dies könnte daran liegen, dass die Exe-Datei geschützt ist. Einige Mods könnten fehlschlagen.",
		failedToDetermineEngine:
			"Fehler beim Bestimmen der Engine für dieses Spiel. Einige Mods könnten fehlschlagen.",
		gameModsLabel: "Mods",
		gameNotInstalledWarning:
			"Dieses Spiel ist nicht installiert, daher bin ich mir nicht 100% sicher, welche Mods kompatibel sind. Die unten angezeigten könnten funktionieren. Wenn Sie das Spiel installieren, zeige ich Ihnen genauere Informationen.",
		uninstallAllModsButton: "Alle Mods deinstallieren",
		uninstallAllModsConfirmation:
			"Sind Sie sicher? Dies wird alle Dateien im Mods-Ordner dieses Spiels löschen. Es werden jedoch keine Dateien aus dem eigentlichen Spiel gelöscht.",
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
			"Dieser Mod ist veraltet. Sie sollten ihn deinstallieren und eine neuere Alternative installieren.",
		modOutdated: "Mod veraltet",
	},

	modModal: {
		runMod: "Ausführen",
		openModFolder: "Mod-Ordner öffnen",
		updateMod: "Mod aktualisieren",
		downloadMod: "Mod herunterladen",
		deleteMod: "Mod löschen",
		deleteModConfirmation:
			"Sind Sie sicher? Alle Dateien im Mod-Ordner gehen verloren.",
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

	steamCache: {
		resetSteamCacheButtonOpenModal: "Steam-Cache zurücksetzen...",
		resetSteamCacheModalTitle: "Steam-Cache zurücksetzen",
		resetSteamCacheDescription:
			"Verwenden Sie dies, wenn Rai Pal Spiele anzeigt, die Sie tatsächlich nicht besitzen. Dies wird den Steam-Cache zurücksetzen, und dann müssen Sie Steam neu starten. Sie erhalten einen Fehler, wenn die Datei bereits gelöscht wurde.",
		resetSteamCacheSuccess:
			"Die Cache-Datei wurde gelöscht. Bitte starten Sie Steam neu, warten Sie ein paar Sekunden und drücken Sie dann die Aktualisierungstaste in Rai Pal.",
	},

	debugData: {
		debugDataTitle: "Debug-Daten",
		debugDataCopy: "Debug-Daten kopieren",
	},

	thanksPage: {
		intro:
			"Hallo. Ich habe Rai Pal gemacht. Ich habe auch in der Vergangenheit andere VR-Mods gemacht und arbeite derzeit an einem universellen VR-Mod für Unity-Spiele. Wenn Ihnen gefällt, was ich tue, und Sie mehr sehen möchten, ziehen Sie bitte eine Spende in Betracht! Sie können mich auch unterstützen, indem Sie einen meiner kostenlosen Mods auf itch.io kaufen.",
		starRaiPalOnGitHub: "Rai Pal auf GitHub bewerten",
		otherModdersTitle: "Andere Modder",
		otherModdersDescription:
			"Rai Pal soll Ihnen helfen, Spiele zu modden, und das können wir nicht ohne die Werkzeuge, die andere Entwickler erstellt haben. Einige dieser Personen haben keine Spendenlinks, aber ich bin ihnen für ihre Arbeit sehr dankbar.",
		modderOnWebsite: "{modderName} auf {website}",
		patreonLeaderboard: "Patreon-Bestenliste",
		rankedByPatreonDonationAmount:
			"Nach der gesamten Lebenszeit-Spendensumme geordnet.",
		patreonProfilePrivateNotice:
			"Wenn Sie sich hier nicht sehen, liegt das daran, dass Ihr Patreon-Profil privat ist.",
	},

	commandButton: {
		cancel: "Abbrechen",
		dontAskAgain: "Nicht mehr fragen",
	},
} as const;
